// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"encoding/json"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/cmd/extract_deps/depparse"
	"cros.local/bazel/ebuild/private/cmd/extract_deps/srcparse"
	"cros.local/bazel/ebuild/private/common/depdata"
	"cros.local/bazel/ebuild/private/common/portage"
	"cros.local/bazel/ebuild/private/common/runfiles"
	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/packages"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

const workspaceDirInChroot = "/mnt/host/source/src"

// HACK: Hard-code several package info.
// TODO: Remove these hacks.
var (
	forceDepsPackages = map[string][]string{
		"virtual/chromeos-bootcomplete": {"chromeos-base/bootcomplete-login"},
		"virtual/editor":                {"app-editors/vim"},
		"virtual/logger":                {"app-admin/rsyslog"},
		"virtual/update-policy":         {"chromeos-base/update-policy-chromeos"},
	}

	invalidEbuilds = map[string]struct{}{
		// The 9999 ebuild isn't actually functional.
		"chromeos-lacros-9999.ebuild": {},
	}
)

// HACK: Hard-code several USE flags.
// TODO: Support USE_EXPAND and remove this hack.
var forceUse = []string{
	"elibc_glibc",
	"input_devices_evdev",
	"kernel_linux",
}

// HACK: Hard-code several packages not to be installed.
var forceProvided = []string{
	// This package was used to force rust binary packages to rebuild.
	// We no longer need this workaround with bazel.
	"virtual/rust-binaries",

	// This is really a BDEPEND and there is no need to declare it as a
	// RDEPEND.
	"virtual/rust",
}

func unique(list []string) []string {
	sort.Strings(list)

	var uniqueRuntimeDeps []string
	var previousDep string
	for _, dep := range list {
		if dep == previousDep {
			continue
		}
		previousDep = dep
		uniqueRuntimeDeps = append(uniqueRuntimeDeps, dep)
	}

	return uniqueRuntimeDeps
}

func filterOutSymlinks(list []*packages.Package) ([]*packages.Package, error) {
	var realFiles []*packages.Package
	for _, pkg := range list {
		info, err := os.Lstat(pkg.Path())
		if err != nil {
			return nil, err
		}
		if info.Mode()&fs.ModeSymlink == 0 {
			realFiles = append(realFiles, pkg)
		}
	}
	return realFiles, nil
}

func isCrosWorkonPackage(pkg *packages.Package) bool {
	return pkg.UsesEclass("cros-workon")
}

func isRustPackage(pkg *packages.Package) bool {
	return pkg.UsesEclass("cros-rust")
}

func isRustSrcPackage(pkg *packages.Package) bool {
	return isRustPackage(pkg) && !isCrosWorkonPackage(pkg) && pkg.Metadata()["HAS_SRC_COMPILE"] == "0"
}

func selectBestPackage(resolver *portage.Resolver, atom *dependency.Atom) (*packages.Package, error) {
	candidates, err := resolver.Packages(atom)
	if err != nil {
		return nil, err
	}

	stabilityTargets := []packages.Stability{packages.StabilityTesting, packages.StabilityStable}

	for _, stabilityTarget := range stabilityTargets {
		for _, candidate := range candidates {
			ebuildFileName := filepath.Base(candidate.Path())
			if _, ok := invalidEbuilds[ebuildFileName]; ok {
				continue
			}

			if candidate.Stability() == stabilityTarget {
				return candidate, nil
			}
		}
	}
	return nil, fmt.Errorf("no package satisfies %s", atom.String())
}

func extractDeps(depType string, pkg *packages.Package, resolver *portage.Resolver) ([]*dependency.Atom, error) {
	if forceDeps, ok := forceDepsPackages[pkg.Name()]; ok {
		if depType != "RDEPEND" {
			return nil, nil
		}

		var atoms []*dependency.Atom
		for _, s := range forceDeps {
			atom, err := dependency.ParseAtom(s)
			if err != nil {
				return nil, err
			}
			atoms = append(atoms, atom)
		}
		return atoms, nil
	}

	metadata := pkg.Metadata()
	deps, err := dependency.Parse(metadata[depType])
	if err != nil {
		return nil, err
	}
	return depparse.Parse(deps, pkg, resolver)
}

func computeDepsInfo(resolver *portage.Resolver, startPackageNames []string) (depdata.PackageInfoMap, error) {
	infoMap := make(depdata.PackageInfoMap)

	var dfs func(atom *dependency.Atom) (*depdata.PackageInfo, error) // for recursive calls
	dfs = func(atom *dependency.Atom) (*depdata.PackageInfo, error) {
		log.Printf("%s", atom.String())

		pkg, err := selectBestPackage(resolver, atom)
		if err != nil {
			return nil, err
		}

		path := pkg.Path()
		if !strings.HasPrefix(path, workspaceDirInChroot+"/") {
			return nil, fmt.Errorf("%s is not under %s", path, workspaceDirInChroot)
		}
		relPath := path[len(workspaceDirInChroot)+1:]
		label := fmt.Sprintf("//%s:%s", filepath.Dir(relPath), pkg.MainSlot())

		if info, ok := infoMap[label]; ok {
			if info == nil {
				return nil, fmt.Errorf("circular dependencies involving %s detected", label)
			}
			if info.Version != pkg.Version().String() {
				return nil, fmt.Errorf("inconsistent package selection for %s: got %s, want %s", label, pkg.Version().String(), info.Version)
			}
			return info, nil
		}

		// Temporarily set nil to detect circular dependencies and avoid infinite
		// recursion.
		infoMap[label] = nil

		rawRuntimeDeps, err := extractDeps("RDEPEND", pkg, resolver)
		if err != nil {
			return nil, err
		}

		rawBuildTimeDeps, err := extractDeps("DEPEND", pkg, resolver)
		if err != nil {
			return nil, err
		}

		// Some rust src packages have their dependencies only listed as DEPEND.
		// They also need to be listed as RDPEND so they get pulled in as
		// transitive deps.
		if isRustSrcPackage(pkg) {
			rawRuntimeDeps = append(rawRuntimeDeps, rawBuildTimeDeps...)
		}

		resolveDeps := func(rawDeps []*dependency.Atom) ([]string, error) {
			depSet := make(map[string]struct{})
			for _, rawDep := range rawDeps {
				info, err := dfs(rawDep)
				if err != nil {
					return nil, err
				}
				l := fmt.Sprintf("//%s:%s", filepath.Dir(info.EBuildPath), info.MainSlot)
				depSet[l] = struct{}{}
			}

			var deps []string
			for dep := range depSet {
				deps = append(deps, dep)
			}
			sort.Strings(deps)
			return deps, nil
		}

		runtimeDeps, err := resolveDeps(rawRuntimeDeps)
		if err != nil {
			return nil, err
		}
		if len(runtimeDeps) > 0 {
			log.Printf("  R: %s", strings.Join(runtimeDeps, ", "))
		}

		buildTimeDeps, err := resolveDeps(rawBuildTimeDeps)
		if err != nil {
			return nil, err
		}
		if len(buildTimeDeps) > 0 {
			log.Printf("  B: %s", strings.Join(buildTimeDeps, ", "))
		}

		srcDeps, err := srcparse.ExtractLocalPackages(pkg)
		if err != nil {
			return nil, err
		}
		if len(srcDeps) > 0 {
			log.Printf("  S: %s", srcDeps)
		}

		srcURIs, err := srcparse.ExtractURIs(pkg)
		if err != nil {
			return nil, err
		}

		info := &depdata.PackageInfo{
			Name:        pkg.Name(),
			MainSlot:    pkg.MainSlot(),
			EBuildPath:  relPath,
			Version:     pkg.Version().String(),
			BuildDeps:   buildTimeDeps,
			LocalSrc:    srcDeps,
			RuntimeDeps: runtimeDeps,
			SrcUris:     srcURIs,
		}
		infoMap[label] = info
		return info, nil
	}

	for _, name := range startPackageNames {
		if _, err := dfs(dependency.NewAtom(name, dependency.OpNone, nil, false, "", nil)); err != nil {
			return nil, err
		}
	}

	return infoMap, nil
}

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagStart = &cli.StringSliceFlag{
	Name:     "start",
	Required: true,
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagBoard,
		flagStart,
	},
	Action: func(c *cli.Context) error {
		board := c.String(flagBoard.Name)
		startPackageNames := c.StringSlice(flagStart.Name)

		rootDir := filepath.Join("/build", board)

		var providedPackages []*config.TargetPackage
		for _, name := range forceProvided {
			providedPackages = append(providedPackages, &config.TargetPackage{
				Name:    name,
				Version: &version.Version{Main: []string{"0"}}, // assume version 0
			})
		}
		hackSource := config.NewHackSource(strings.Join(forceUse, " "), providedPackages)

		resolver, err := portage.NewResolver(rootDir, hackSource)
		if err != nil {
			return err
		}

		infoMap, err := computeDepsInfo(resolver, startPackageNames)
		if err != nil {
			return err
		}

		srcparse.FixupLocalSource(infoMap)

		infoMap.FixupForJSON()

		encoder := json.NewEncoder(os.Stdout)
		encoder.SetIndent("", "  ")
		encoder.SetEscapeHTML(false)
		return encoder.Encode(infoMap)
	},
}

func main() {
	runfiles.FixEnv()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
