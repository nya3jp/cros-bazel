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

// HACK: Hard-code several package info.
// TODO: Remove these hacks.
var (
	invalidEbuilds = map[string]struct{}{
		// The 9999 ebuild isn't actually functional.
		"chromeos-lacros-9999.ebuild": {},
		// Some type of transitional ebuild
		"ncurses-5.9-r99.ebuild": {},
	}

	// These packages depend on newer versions of them selves (sigh).
	// In order to avoid a circular dependency we need to specify the
	// exact version.
	ebuildDepOverride = map[string]map[string]string{
		"cortex-m-0.6.7.ebuild": {
			"dev-rust/cortex-m": "=dev-rust/cortex-m-0.7.3",
		},
		"nb-0.1.3.ebuild": {
			"dev-rust/nb": "=dev-rust/nb-1.0.0",
		},
	}
)

// HACK: Hard-code several USE flags.
// TODO: Support USE_EXPAND and remove this hack.
var forceUse = []string{
	"elibc_glibc",
	"kernel_linux",
}

// HACK: Hard-code several packages not to be installed.
var forceProvided = []string{
	// TODO: Parse /etc/portage/profile/package.provided and obtain these packages.
	"sys-devel/gcc",
	"sys-libs/glibc",
	"dev-lang/go",

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

func genericBFS[Key comparable](start []Key, visitor func(key Key) ([]Key, error)) error {
	queue := make([]Key, len(start))
	for i, key := range start {
		queue[i] = key
	}

	seen := make(map[Key]struct{})
	for _, key := range start {
		seen[key] = struct{}{}
	}

	for len(queue) > 0 {
		current := queue[0]
		queue = queue[1:]

		nexts, err := visitor(current)
		if err != nil {
			return fmt.Errorf("%v: %w", current, err)
		}

		for _, next := range nexts {
			if _, ok := seen[next]; ok {
				continue
			}
			queue = append(queue, next)
			seen[next] = struct{}{}
		}
	}
	return nil
}

func selectBestPackagesUsingStability(resolver *portage.Resolver, atom *dependency.Atom) ([]*packages.Package, error) {
	candidates, err := resolver.Packages(atom)
	if err != nil {
		return nil, err
	}

	stabilityTargets := []packages.Stability{packages.StabilityTesting, packages.StabilityStable}

	for _, stabilityTarget := range stabilityTargets {
		var matchingPackages []*packages.Package
		for _, candidate := range candidates {
			ebuildFileName := filepath.Base(candidate.Path())
			if _, ok := invalidEbuilds[ebuildFileName]; ok {
				continue
			}

			if candidate.Stability() == stabilityTarget {
				matchingPackages = append(matchingPackages, candidate)
			}
		}
		if len(matchingPackages) > 0 {
			return matchingPackages, nil
		}
	}
	return nil, fmt.Errorf("no package satisfies %s", atom.String())
}

func selectBestPackages(resolver *portage.Resolver, atom *dependency.Atom) ([]*packages.Package, error) {
	candidates, err := selectBestPackagesUsingStability(resolver, atom)
	if err != nil {
		return nil, err
	}

	var pkgs []*packages.Package
	if atom.PackageCategory() == "dev-rust" {
		pkgs, err = filterOutSymlinks(candidates)
		if err != nil {
			return nil, err
		}
	} else {
		// TODO: Is this sorted so we always pick the greatest version?
		pkgs = []*packages.Package{candidates[0]}
	}

	return pkgs, nil
}

func extractDeps(depType string, pkg *packages.Package, resolver *portage.Resolver) ([]string, error) {
	metadata := pkg.Metadata()
	deps, err := dependency.Parse(metadata[depType])
	if err != nil {
		return nil, err
	}
	return depparse.Parse(deps, pkg, resolver)
}

func applyDepOverrides(pkg *packages.Package, deps []string) []string {
	overrideMap, ok := ebuildDepOverride[filepath.Base(pkg.Path())]
	if !ok {
		return deps
	}

	for i, packageName := range deps {
		if override, ok := overrideMap[packageName]; ok {
			deps[i] = override
			break
		}
	}

	return deps
}

func computeDepsInfo(resolver *portage.Resolver, startPackageNames []string) (depdata.PackageInfoMap, error) {
	depsInfo := make(depdata.PackageInfoMap)
	if err := genericBFS(startPackageNames, func(packageName string) ([]string, error) {
		atom, err := dependency.ParseAtom(packageName)
		if err != nil {
			return nil, err
		}

		pkgs, err := selectBestPackages(resolver, atom)
		if err != nil {
			return nil, err
		}

		var allPkgDeps []string

		for _, pkg := range pkgs {
			log.Printf("%s-%s:", pkg.Name(), pkg.Version())

			runtimeDeps, err := extractDeps("RDEPEND", pkg, resolver)
			if err != nil {
				return nil, err
			}
			if len(runtimeDeps) > 0 {
				log.Printf("  R: %s", strings.Join(runtimeDeps, ", "))
			}

			buildTimeDeps, err := extractDeps("DEPEND", pkg, resolver)
			if err != nil {
				return nil, err
			}
			if len(buildTimeDeps) > 0 {
				log.Printf("  B: %s", strings.Join(buildTimeDeps, ", "))
			}

			// Some rust src packages have their dependencies only listed as DEPEND.
			// They also need to be listed as RDPEND so they get pulled in as
			// transitive deps.
			if isRustSrcPackage(pkg) {
				runtimeDeps = append(runtimeDeps, buildTimeDeps...)
				runtimeDeps = unique(runtimeDeps)
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

			allPkgDeps = append(allPkgDeps, buildTimeDeps...)
			allPkgDeps = append(allPkgDeps, runtimeDeps...)

			buildTimeDeps = applyDepOverrides(pkg, buildTimeDeps)
			runtimeDeps = applyDepOverrides(pkg, runtimeDeps)

			depsInfo[pkg.Name()] = append(depsInfo[pkg.Name()], &depdata.PackageInfo{
				Version:     pkg.Version().String(),
				BuildDeps:   buildTimeDeps,
				LocalSrc:    srcDeps,
				RuntimeDeps: runtimeDeps,
				SrcUris:     srcURIs,
			})
		}

		return unique(allPkgDeps), nil
	}); err != nil {
		return nil, err
	}
	return depsInfo, nil
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
