// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/common/portage/makeconf"
	"cros.local/bazel/ebuild/private/common/portage/portagevars"
	"cros.local/bazel/ebuild/private/common/portage/repository"
	"cros.local/bazel/ebuild/private/common/runfiles"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/ebuild"
	"cros.local/bazel/ebuild/private/common/standard/useflags"
)

// HACK: Hard-code several package info.
// TODO: Remove these hacks.
var (
	knownInstalledPackages = map[string]struct{}{
		"sys-libs/glibc": {},
	}
	knownMissingPackages = map[string]struct{}{
		"app-crypt/heimdal":       {},
		"app-misc/realpath":       {},
		"media-libs/jpeg":         {},
		"net-firewall/nftables":   {},
		"sys-auth/openpam":        {},
		"sys-freebsd/freebsd-bin": {},
		"sys-freebsd/freebsd-lib": {},
		"sys-fs/eudev":            {},
		"sys-libs/e2fsprogs-libs": {},
	}
	forceDepsPackages = map[string][]string{
		"sys-libs/ncurses":              {},
		"virtual/chromeos-bootcomplete": {"chromeos-base/bootcomplete-login"},
		"virtual/editor":                {},
		"virtual/libgudev":              {"sys-fs/udev"},
		"virtual/logger":                {"app-admin/rsyslog"},
		"virtual/mta":                   {},
		"virtual/pkgconfig":             {"dev-util/pkgconfig"},
		"virtual/update-policy":         {"chromeos-base/update-policy-chromeos"},
	}
)

func simplifyDeps(deps *dependency.Deps, use map[string]struct{}, packageName string) *dependency.Deps {
	isRust := strings.HasPrefix(packageName, "dev-rust/")

	deps = dependency.ResolveUse(deps, use)

	// Rewrite package atoms.
	deps = dependency.Simplify(dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return expr
		}

		packageName := pkg.Atom().PackageName()

		// Remove blocks.
		if pkg.Blocks() > 0 {
			return dependency.ConstTrue
		}

		// Rewrite known packages.
		if _, installed := knownInstalledPackages[packageName]; installed {
			return dependency.ConstTrue
		}
		if _, missing := knownMissingPackages[packageName]; missing {
			return dependency.ConstFalse
		}

		// Heuristic: ~ deps in Rust packages can be dropped.
		if isRust && pkg.Atom().VersionOperator() == dependency.OpRoughEqual {
			return dependency.ConstTrue
		}

		// Strip modifiers.
		atom := dependency.NewAtom(packageName, dependency.OpNone, nil, false, "", nil)
		return dependency.NewPackage(atom, 0)
	}))

	// Unify AnyOf whose children refer to the same package.
	deps = dependency.Simplify(dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		anyOf, ok := expr.(*dependency.AnyOf)
		if !ok {
			return expr
		}

		children := anyOf.Children()
		if len(children) == 0 {
			return expr
		}

		pkg0, ok := children[0].(*dependency.Package)
		if !ok {
			return expr
		}

		same := true
		for _, child := range children {
			pkg, ok := child.(*dependency.Package)
			if ok && pkg.Atom().PackageName() == pkg0.Atom().PackageName() {
				continue
			}
			same = false
			break
		}
		if !same {
			return expr
		}
		return pkg0
	}))

	// Deduplicate occurrences of the same package atom.
	var alwaysPkgs []dependency.Expr
	alwaysSet := make(map[string]struct{})
	for _, expr := range deps.Expr().Children() {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			continue
		}
		alwaysPkgs = append(alwaysPkgs, pkg)
		alwaysSet[pkg.String()] = struct{}{}
	}
	deps = dependency.Simplify(dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return expr
		}
		if _, ok := alwaysSet[pkg.Atom().PackageName()]; ok {
			return dependency.ConstTrue
		}
		return pkg
	}))
	deps = dependency.Simplify(dependency.NewDeps(dependency.NewAllOf(append(alwaysPkgs, deps.Expr()))))

	// Remove trivial AnyOf.
	deps = dependency.Simplify(dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		anyOf, ok := expr.(*dependency.AnyOf)
		if !ok {
			return expr
		}
		log.Print(anyOf.String())
		children := anyOf.Children()
		if len(children) == 0 {
			return expr
		}
		pkg0, ok := children[0].(*dependency.Package)
		if !ok {
			return expr
		}
		for _, child := range children {
			pkg, ok := child.(*dependency.Package)
			if !ok {
				return expr
			}
			if pkg.Atom().PackageName() != pkg0.Atom().PackageName() {
				return expr
			}
		}
		return pkg0
	}))

	return deps
}

func parseSimpleDeps(deps *dependency.Deps) ([]string, bool) {
	nameSet := make(map[string]struct{})
	for _, expr := range deps.Expr().Children() {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return nil, false
		}
		nameSet[pkg.Atom().PackageName()] = struct{}{}
	}

	names := make([]string, 0, len(nameSet))
	for name := range nameSet {
		names = append(names, name)
	}
	sort.Strings(names)

	return names, true
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

func computeRuntimeDeps(repoSet *repository.RepoSet, processor *ebuild.CachedProcessor, useContext *useflags.Context, startPackageNames []string) (map[string][]string, error) {
	depsMap := make(map[string][]string)
	if err := genericBFS(startPackageNames, func(packageName string) ([]string, error) {
		atom, err := dependency.ParseAtom(packageName)
		if err != nil {
			return nil, err
		}

		pkg, err := repoSet.BestPackage(atom, processor, useContext)
		if err != nil {
			return nil, err
		}

		vars := pkg.Vars()

		rawDeps, err := dependency.Parse(vars["RDEPEND"])
		if err != nil {
			return nil, err
		}

		simpleDeps := simplifyDeps(rawDeps, pkg.Uses(), packageName)

		parsedDeps, ok := forceDepsPackages[packageName]
		if !ok {
			parsedDeps, ok = parseSimpleDeps(simpleDeps)
			if !ok {
				return nil, fmt.Errorf("cannot simplify deps: %s", simpleDeps.String())
			}
		}

		log.Printf("R: %s => %s", packageName, strings.Join(parsedDeps, ", "))
		depsMap[packageName] = parsedDeps
		return parsedDeps, nil
	}); err != nil {
		return nil, err
	}
	return depsMap, nil
}

func computeBuildDeps(repoSet *repository.RepoSet, processor *ebuild.CachedProcessor, useContext *useflags.Context, installPackageNames []string) (map[string][]string, error) {
	depsMap := make(map[string][]string)
	if err := genericBFS(installPackageNames, func(packageName string) ([]string, error) {
		atom, err := dependency.ParseAtom(packageName)
		if err != nil {
			return nil, err
		}

		pkg, err := repoSet.BestPackage(atom, processor, useContext)
		if err != nil {
			return nil, err
		}

		vars := pkg.Vars()

		rawDeps, err := dependency.Parse(vars["DEPEND"])
		if err != nil {
			return nil, err
		}

		simpleDeps := simplifyDeps(rawDeps, pkg.Uses(), packageName)

		var parsedDeps []string
		if _, ok := forceDepsPackages[packageName]; ok {
			parsedDeps = nil
		} else {
			parsedDeps, ok = parseSimpleDeps(simpleDeps)
			if !ok {
				return nil, fmt.Errorf("cannot simplify deps: %s", simpleDeps.String())
			}
		}

		log.Printf("B: %s => %s", packageName, strings.Join(parsedDeps, ", "))
		depsMap[packageName] = parsedDeps
		return parsedDeps, nil
	}); err != nil {
		return nil, err
	}
	return depsMap, nil
}

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagStart = &cli.StringSliceFlag{
	Name:     "start",
	Required: true,
}

type packageInfo struct {
	BuildDeps   []string `json:"buildDeps"`
	RuntimeDeps []string `json:"runtimeDeps"`
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

		makeConfVars, err := makeconf.ParseDefaults(rootDir)
		if err != nil {
			return err
		}

		// HACK: Set some USE variables since we don't support USE_EXPAND yet.
		// TODO: Remove this hack.
		makeConfVars["USE"] += " elibc_glibc"

		overlays := portagevars.Overlays(makeConfVars)

		repoSet, err := repository.NewRepoSet(overlays)
		if err != nil {
			return err
		}

		profilePath, err := os.Readlink(filepath.Join(rootDir, "etc/portage/make.profile"))
		if err != nil {
			return err
		}

		rawProfile, err := repoSet.ProfileByPath(profilePath)
		if err != nil {
			return err
		}

		profile, err := rawProfile.Parse()
		if err != nil {
			return err
		}

		processor := ebuild.NewCachedProcessor(ebuild.NewProcessor(profile.Vars(), repoSet.EClassDirs()))

		useContext := useflags.NewContext(makeConfVars, profile)

		runtimeDeps, err := computeRuntimeDeps(repoSet, processor, useContext, startPackageNames)
		if err != nil {
			return err
		}

		installPackageNames := make([]string, 0, len(runtimeDeps))
		for packageName := range runtimeDeps {
			installPackageNames = append(installPackageNames, packageName)
		}
		sort.Strings(installPackageNames)

		buildDeps, err := computeBuildDeps(repoSet, processor, useContext, installPackageNames)
		if err != nil {
			return err
		}

		nonNil := func(deps []string) []string {
			if deps == nil {
				deps = []string{}
			}
			return deps
		}

		infoMap := make(map[string]*packageInfo)
		for packageName := range buildDeps {
			infoMap[packageName] = &packageInfo{
				RuntimeDeps: nonNil(runtimeDeps[packageName]),
				BuildDeps:   nonNil(buildDeps[packageName]),
			}
		}

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
