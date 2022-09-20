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
		"virtual/tmpfiles":              {"sys-apps/systemd-tmpfiles"},
		"virtual/update-policy":         {"chromeos-base/update-policy-chromeos"},
	}
)

// HACK: Hard-code several USE flags.
// TODO: Support USE_EXPAND and remove this hack.
var forceUse = []string{
	"elibc_glibc",
	"kernel_linux",
}

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

func computeSrcPackages(category string, project string, localName string, subtree string) ([]string, error) {

	// The parser will return | concat arrays, so undo that here.
	projects := strings.Split(project, "|")

	// Not a cros-workon package
	if len(projects) == 0 {
		return nil, nil
	}

	var localNames []string
	if localName == "" {
		localNames = []string{}
	} else {
		localNames = strings.Split(localName, "|")
	}
	if len(localNames) > 0 && len(localNames) != len(projects) {
		return nil, fmt.Errorf("Number of elements in LOCAL_NAME (%d) and PROJECT (%d) don't match.", len(localNames), len(projects))
	}

	var subTrees []string
	if subtree == "" {
		subTrees = []string{}
	} else {
		subTrees = strings.Split(subtree, "|")
	}

	if len(subTrees) > 0 && len(subTrees) != len(projects) {
		return nil, fmt.Errorf("Number of elements in SUBTREE (%d) and PROJECT (%d) don't match.", len(subTrees), len(projects))
	}

	var allPaths []string

	for i, project := range projects {
		var localName string
		var subtree string
		if len(localNames) > i {
			localName = localNames[i]
		}
		if len(subTrees) > i {
			subtree = subTrees[i]
		}

		if project == "chromiumos/platform/chromiumos-assets" && localName == "chromiumos-assets" {
			// ebuild is incorrect
			localName = "platform/chromiumos-assets"
		}

		var paths []string

		// If there is no local name, then we need to compute it
		if localName == "" {
			if strings.HasPrefix(project, "chromiumos/") {
				paths = []string{strings.TrimPrefix(project, "chromiumos/")}
			}
		} else {
			if category == "chromeos-base" {
				paths = []string{localName}
			} else if strings.HasPrefix(localName, "../") {
				paths = []string{strings.TrimPrefix(localName, "../")}
			} else {
				paths = []string{"third_party/" + localName}
			}
		}

		if subtree != "" {
			var newPaths = []string{}

			for _, path := range paths {
				if path == "platform/vboot_reference" || path == "third_party/coreboot"  {
					// coreboot-utils maps a lot of different sub folders.
					// TODO: Do we want to support such fine granularity?
					newPaths = append(newPaths, path)
					continue
				} else if !strings.HasPrefix(path, "platform2") {
					// TODO: Should we support sub paths for non-platform2?
					// It requires adding BUILD files in a lot more places
					// Or we need to figure out how to pass individual files
					// into the build.
					newPaths = append(newPaths, path)
					continue
				}

				for _, subtree := range strings.Split(subtree, " ") {
					if subtree == ".gn" {
						// Use the platform2 src package instead
						newPaths = append(newPaths, path)
					} else if subtree == ".clang-format" {
						// We really don't need .clang-format to build...
						continue
					} else if subtree == "chromeos-config/cros_config_host" {
						// We don't have a sub package for chromeos-config
						newPaths = append(newPaths, path+"/chromeos-config")
					} else {
						newPaths = append(newPaths, path+"/"+subtree)
					}
				}
			}
			paths = newPaths
		}

		if project == "chromiumos/third_party/kernel" {
			paths = append(paths, "third_party/chromiumos-overlay/eclass/cros-kernel")
		}

		allPaths = append(allPaths, paths...)
	}

	sort.Strings(allPaths)

	var srcDeps []string
	var previousPath string
	for _, path := range allPaths {
		if previousPath == path {
			// Some packages contain duplicate paths
			continue
		}
		previousPath = path
		srcDeps = append(srcDeps, "//"+path+":src")
	}

	return srcDeps, nil
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

func filterPackages(pkgs []string, dropSet map[string]struct{}) []string {
	var filtered []string
	for _, pkg := range pkgs {
		if _, ok := dropSet[pkg]; !ok {
			filtered = append(filtered, pkg)
		}
	}
	return filtered
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

func computeRuntimeDeps(repoSet *repository.RepoSet, processor *ebuild.CachedProcessor, useContext *useflags.Context, providedPackages map[string]struct{}, startPackageNames []string) (map[string][]string, error) {
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

		parsedDeps = filterPackages(parsedDeps, providedPackages)

		log.Printf("R: %s => %s", packageName, strings.Join(parsedDeps, ", "))
		depsMap[packageName] = parsedDeps
		return parsedDeps, nil
	}); err != nil {
		return nil, err
	}
	return depsMap, nil
}

func computeBuildDeps(repoSet *repository.RepoSet, processor *ebuild.CachedProcessor, useContext *useflags.Context, providedPackages map[string]struct{}, installPackageNames []string) (map[string][]string, map[string][]string, error) {
	depsMap := make(map[string][]string)
	srcDepMap := make(map[string][]string)
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

		parsedDeps = filterPackages(parsedDeps, providedPackages)

		log.Printf("B: %s => %s", packageName, strings.Join(parsedDeps, ", "))
		depsMap[packageName] = parsedDeps

		srcDeps, err := computeSrcPackages(pkg.Category(), vars["CROS_WORKON_PROJECT"], vars["CROS_WORKON_LOCALNAME"], vars["CROS_WORKON_SUBTREE"])
		if err != nil {
			return nil, err
		}
		log.Printf("   PROJECT: '%s', LOCALNAME: '%s', SUBTREE: '%s' -> %s", vars["CROS_WORKON_PROJECT"], vars["CROS_WORKON_LOCALNAME"], vars["CROS_WORKON_SUBTREE"], srcDeps)
		srcDepMap[packageName] = srcDeps

		return parsedDeps, nil
	}); err != nil {
		return nil, nil, err
	}
	return depsMap, srcDepMap, nil
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
	LocalSrc    []string `json:"localSrc"`
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
		makeConfVars["USE"] += " " + strings.Join(forceUse, " ")

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

		providedPackages := map[string]struct{}{
			// TODO: Parse /etc/portage/profile/package.provided and obtain this list.
			"sys-devel/gcc":  {},
			"sys-libs/glibc": {},
			"dev-lang/go":    {},
		}
		for _, pp := range profile.Provided() {
			providedPackages[pp.Name()] = struct{}{}
		}

		processor := ebuild.NewCachedProcessor(ebuild.NewProcessor(profile.Vars(), repoSet.EClassDirs()))

		useContext := useflags.NewContext(makeConfVars, profile)

		runtimeDeps, err := computeRuntimeDeps(repoSet, processor, useContext, providedPackages, startPackageNames)
		if err != nil {
			return err
		}

		installPackageNames := make([]string, 0, len(runtimeDeps))
		for packageName := range runtimeDeps {
			installPackageNames = append(installPackageNames, packageName)
		}
		sort.Strings(installPackageNames)

		buildDeps, buildSrcDeps, err := computeBuildDeps(repoSet, processor, useContext, providedPackages, installPackageNames)
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
				LocalSrc:    nonNil(buildSrcDeps[packageName]),
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
