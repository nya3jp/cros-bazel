// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/url"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"encoding/base64"
	"encoding/hex"
	"github.com/urfave/cli"
	"strconv"

	"cros.local/bazel/ebuild/private/common/portage/repository"
	"cros.local/bazel/ebuild/private/common/runfiles"
	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/ebuild"
	"cros.local/bazel/ebuild/private/common/standard/packages"
	"cros.local/bazel/ebuild/private/common/standard/version"
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
	additionalSrcPackages = map[string][]string{
		"app-accessibility/pumpkin":        []string{"@chromite//:src"},
		"chromeos-base/libchrome":          []string{"@chromite//:src"},
		"chromeos-languagepacks/tts-es-us": []string{"@chromite//:src"},
		"chromeos-base/sample-dlc":         []string{"@chromite//:src"},
		"dev-libs/modp_b64":                []string{"@chromite//:src"},
		"media-sound/sr-bt-dlc":            []string{"@chromite//:src"},
	}
	badSrcUris = map[string]struct{}{
		"x11-misc/xkeyboard-config": {},
	}

	invalidUnstablePackage = map[string]struct{}{
		// The 9999 ebuild isn't actually functional.
		"chromeos-base/chromeos-lacros": {},
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

func isCrosWorkonPackage(pkg *packages.Package) bool {
	return pkg.UsesEclass("cros-workon")
}

func isRustPackage(pkg *packages.Package) bool {
	return pkg.UsesEclass("cros-rust")
}

func isRustSrcPackage(pkg *packages.Package) bool {
	return isRustPackage(pkg) && !isCrosWorkonPackage(pkg) && pkg.Metadata()["HAS_SRC_COMPILE"] == "0"
}

func simplifyDeps(deps *dependency.Deps, use map[string]bool, packageName string) *dependency.Deps {
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

		// So we have circular dependencies in the rust graph. See:
		// * src/third_party/chromiumos-overlay/dev-rust/futures-util/futures-util-0.3.13.ebuild
		// * src/third_party/chromiumos-overlay/dev-rust/hashbrown/hashbrown-0.11.2.ebuild
		// In order to break it there is an empty package that is used to break the deps. Since
		// the package is empty we can get away with just dropping the dependency.
		if pkg.Atom().String() == "~dev-rust/tokio-io-0.1.9" ||
			pkg.Atom().String() == "~dev-rust/ahash-0.7.0:=" {
			return dependency.ConstTrue
		}

		// Rewrite known packages.
		if _, installed := knownInstalledPackages[packageName]; installed {
			return dependency.ConstTrue
		}
		if _, missing := knownMissingPackages[packageName]; missing {
			return dependency.ConstFalse
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

func computeSrcPackages(pkg *packages.Package, project string, localName string, subtree string) ([]string, error) {

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
			if pkg.Category() == "chromeos-base" {
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
				if path == "platform/vboot_reference" || path == "third_party/coreboot" {
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
		if pkg.Name() == "dev-rust/sys_util_core" && path == "platform/crosvm" {
			// We need a pinned version of crosvm for sys_util_core, so we can't
			// use the default location.
			path = "platform/crosvm-sys_util_core"
		}

		if pkg.Name() == "sys-apps/mosys" && path == "platform2/common-mk" {
			// Mosys calls some unsupported qemu testing code in common-mk.
			// Instead of pulling this in, use the stubbed out version in the
			// sdk.
			continue
		}

		if previousPath == path {
			// Some packages contain duplicate paths
			continue
		}
		previousPath = path
		srcDeps = append(srcDeps, "//"+path+":src")
	}

	if additionalSrcPackages, ok := additionalSrcPackages[pkg.Name()]; ok {
		srcDeps = append(srcDeps, additionalSrcPackages...)
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

type manifestEntry struct {
	size      int
	integrity string
	SHA256    string
	SHA512    string
}

func parseSimpleUris(deps *dependency.Deps, manifest map[string]manifestEntry) (map[string]uriInfo, error) {
	uriMap := make(map[string][]string)
	for _, expr := range deps.Expr().Children() {
		uri, ok := expr.(*dependency.Uri)
		if !ok {
			return nil, fmt.Errorf("Expected Uri, got %s", expr)
		}
		var fileName string
		if uri.FileName() != nil {
			fileName = *uri.FileName()
		} else {
			parsedUri, err := url.ParseRequestURI(uri.Uri())
			if err != nil {
				return nil, err
			}
			fileName = filepath.Base(parsedUri.Path)
		}

		uriMap[fileName] = append(uriMap[fileName], uri.Uri())
	}

	uriInfoMap := make(map[string]uriInfo)
	for fileName, uris := range uriMap {
		entry, ok := manifest[fileName]
		if !ok {
			return nil, fmt.Errorf("Cannot find file %s in Manifest %s", fileName, manifest)
		}

		uriInfoMap[fileName] = uriInfo{
			Uris:      uris,
			Size:      entry.size,
			Integrity: entry.integrity,
			// TODO: Remove these when we can use integrity
			SHA256: entry.SHA256,
			SHA512: entry.SHA512,
		}
	}

	return uriInfoMap, nil
}

func hashToIntegrity(name string, hexHash string) (string, error) {
	hashBytes, err := hex.DecodeString(hexHash)
	if err != nil {
		return "", err
	}

	hashBase64 := base64.StdEncoding.EncodeToString(hashBytes)

	integrity := fmt.Sprintf("%s-%s", strings.ToLower(name), hashBase64)

	return integrity, nil
}

func parseManifest(eBuildPath string) (map[string]manifestEntry, error) {
	files := make(map[string]manifestEntry)

	ebuildDir := filepath.Dir(eBuildPath)

	// Read Manifest to get a list of distfiles.
	manifest, err := os.ReadFile(filepath.Join(ebuildDir, "Manifest"))
	if err != nil {
		return nil, err
	}

	for _, line := range strings.Split(string(manifest), "\n") {
		fields := strings.Fields(line)
		if len(fields) < 3 || fields[0] != "DIST" {
			continue
		}

		fileName, err := url.PathUnescape(fields[1])
		if err != nil {
			return nil, err
		}

		size, err := strconv.Atoi(fields[2])
		if err != nil {
			return nil, err
		}

		hexHashes := make(map[string]string)
		for i := 3; i+1 < len(fields); i += 2 {
			hexHashes[fields[i]] = fields[i+1]
		}

		// We prefer SHA512 for integrity checking
		for _, hashName := range []string{"SHA512", "SHA256", "BLAKE2B"} {
			hexHash, ok := hexHashes[hashName]
			if ok {
				integrity, err := hashToIntegrity(hashName, hexHash)
				if err != nil {
					return nil, err
				}

				files[fileName] = manifestEntry{
					size:      size,
					integrity: integrity,
					// Our version of bazel doesn't support integrity on http_file, only http_archive
					// so we need to plumb in the hashes.
					SHA256: hexHashes["SHA256"],
					// If we don't have a SHA256 we will use the SHA512 to verify the downloaded file
					// and then compute the SHA256
					SHA512: hexHashes["SHA512"],
				}
				break
			}
		}
	}

	return files, nil
}

func filterPackages(pkgs []string, provided []*config.Package) []string {
	providedSet := make(map[string]struct{})
	for _, p := range provided {
		providedSet[p.Name] = struct{}{}
	}

	var filtered []string
	for _, pkg := range pkgs {
		if _, ok := providedSet[pkg]; !ok {
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

func selectBestPackages(atom *dependency.Atom, processor *ebuild.CachedProcessor, s *repository.RepoSet) ([]*packages.Package, error) {
	candidates, err := s.Package(atom, processor)
	if err != nil {
		return nil, err
	}

	stabilityTargets := []packages.Stability{packages.StabilityTesting, packages.StabilityStable}
	if _, ok := invalidUnstablePackage[atom.PackageName()]; ok {
		stabilityTargets = stabilityTargets[1:]
	}

	for _, stabilityTarget := range stabilityTargets {
		var matchingPackages []*packages.Package
		for _, candidate := range candidates {
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

func computeRuntimeDeps(repoSet *repository.RepoSet, processor *ebuild.CachedProcessor, provided []*config.Package, startPackageNames []string) (map[string][]string, error) {
	depsMap := make(map[string][]string)
	if err := genericBFS(startPackageNames, func(packageName string) ([]string, error) {
		atom, err := dependency.ParseAtom(packageName)
		if err != nil {
			return nil, err
		}

		candidates, err := selectBestPackages(atom, processor, repoSet)
		if err != nil {
			return nil, err
		}

		pkg := candidates[0]

		metadata := pkg.Metadata()

		rawDeps, err := dependency.Parse(metadata["RDEPEND"])
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

		parsedDeps = filterPackages(parsedDeps, provided)

		log.Printf("R: %s => %s", packageName, strings.Join(parsedDeps, ", "))
		depsMap[packageName] = parsedDeps
		return parsedDeps, nil
	}); err != nil {
		return nil, err
	}
	return depsMap, nil
}

func computeBuildDeps(repoSet *repository.RepoSet, processor *ebuild.CachedProcessor, provided []*config.Package, installPackageNames []string) (map[string]packageInfo, error) {
	pkgs := make(map[string]packageInfo)

	if err := genericBFS(installPackageNames, func(packageName string) ([]string, error) {
		var info packageInfo

		atom, err := dependency.ParseAtom(packageName)
		if err != nil {
			return nil, err
		}

		candidates, err := selectBestPackages(atom, processor, repoSet)
		if err != nil {
			return nil, err
		}
		pkg := candidates[0]

		info.Version = pkg.Version().String()

		metadata := pkg.Metadata()

		rawDeps, err := dependency.Parse(metadata["DEPEND"])
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

		parsedDeps = filterPackages(parsedDeps, provided)

		// Some rust src packages have their dependencies only listed as DEPEND.
		// They also need to be listed as RDPEND so they get pulled in as
		// transitive deps.
		if isRustSrcPackage(pkg) {
			log.Printf("B & R: %s => %s", packageName, strings.Join(parsedDeps, ", "))
			info.RuntimeDeps = parsedDeps
			info.BuildDeps = parsedDeps
		} else {
			log.Printf("B: %s => %s", packageName, strings.Join(parsedDeps, ", "))
			info.BuildDeps = parsedDeps
		}

		srcDeps, err := computeSrcPackages(pkg, metadata["CROS_WORKON_PROJECT"], metadata["CROS_WORKON_LOCALNAME"], metadata["CROS_WORKON_SUBTREE"])
		if err != nil {
			return nil, err
		}
		log.Printf("   PROJECT: '%s', LOCALNAME: '%s', SUBTREE: '%s' -> %s", metadata["CROS_WORKON_PROJECT"], metadata["CROS_WORKON_LOCALNAME"], metadata["CROS_WORKON_SUBTREE"], srcDeps)
		info.LocalSrc = srcDeps

		srcUri, ok := metadata["SRC_URI"]
		_, hasBadSrcUri := badSrcUris[packageName]
		if ok && !hasBadSrcUri && srcUri != "" {
			srcUris, err := dependency.Parse(srcUri)
			if err != nil {
				return nil, err
			}

			srcUris = dependency.ResolveUse(srcUris, pkg.Uses())
			srcUris = dependency.Simplify(srcUris)

			if len(srcUris.Expr().Children()) > 0 {
				manifest, err := parseManifest(pkg.Path())
				if err != nil {
					return nil, err
				}

				srcUriInfo, err := parseSimpleUris(srcUris, manifest)
				if err != nil {
					return nil, err
				}
				info.SrcUris = srcUriInfo
			}
		}

		pkgs[packageName] = info

		return parsedDeps, nil
	}); err != nil {
		return nil, err
	}
	return pkgs, nil
}

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagStart = &cli.StringSliceFlag{
	Name:     "start",
	Required: true,
}

type uriInfo struct {
	Uris      []string `json:"uris"`
	Size      int      `json:"size"`
	Integrity string   `json:"integrity"`
	// TODO: Remove when we can use integrity
	SHA256 string `json:"SHA256"`
	SHA512 string `json:"SHA512"`
}

type packageInfo struct {
	Version     string             `json:"version"`
	BuildDeps   []string           `json:"buildDeps"`
	LocalSrc    []string           `json:"localSrc"`
	RuntimeDeps []string           `json:"runtimeDeps"`
	SrcUris     map[string]uriInfo `json:"srcUris"`
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

		var providedPackages []*config.Package
		for _, name := range forceProvided {
			providedPackages = append(providedPackages, &config.Package{
				Name:    name,
				Version: &version.Version{Main: []string{"0"}}, // assume version 0
			})
		}
		hackSource := config.NewHackSource(strings.Join(forceUse, " "), providedPackages)

		defaults, err := repository.LoadDefaults(rootDir, hackSource)
		if err != nil {
			return err
		}

		processor := ebuild.NewCachedProcessor(ebuild.NewProcessor(defaults.Config, defaults.RepoSet.EClassDirs()))
		provided, err := defaults.Config.ProvidedPackages()
		if err != nil {
			return err
		}

		runtimeDeps, err := computeRuntimeDeps(defaults.RepoSet, processor, provided, startPackageNames)
		if err != nil {
			return err
		}

		installPackageNames := make([]string, 0, len(runtimeDeps))
		for packageName := range runtimeDeps {
			installPackageNames = append(installPackageNames, packageName)
		}
		sort.Strings(installPackageNames)

		pkgInfoByPkgName, err := computeBuildDeps(
			defaults.RepoSet, processor, provided, installPackageNames)
		if err != nil {
			return err
		}

		var sortedPackageNames []string
		for packageName := range pkgInfoByPkgName {
			sortedPackageNames = append(sortedPackageNames, packageName)
		}
		sort.Strings(sortedPackageNames)

		nonNil := func(deps []string) []string {
			if deps == nil {
				deps = []string{}
			}
			return deps
		}

		nonNilUri := func(deps map[string]uriInfo) map[string]uriInfo {
			if deps == nil {
				deps = map[string]uriInfo{}
			}
			return deps
		}

		infoMap := make(map[string][]*packageInfo)
		for _, packageName := range sortedPackageNames {
			info := pkgInfoByPkgName[packageName]

			newRuntimeDeps := append(runtimeDeps[packageName], info.RuntimeDeps...)
			sort.Strings(newRuntimeDeps)

			var uniqueRuntimeDeps = []string{}
			var previousDep string
			for _, dep := range newRuntimeDeps {
				if dep == previousDep {
					continue
				}
				previousDep = dep
				uniqueRuntimeDeps = append(uniqueRuntimeDeps, dep)
			}

			info.BuildDeps = nonNil(info.BuildDeps)
			info.LocalSrc = nonNil(info.LocalSrc)
			info.RuntimeDeps = nonNil(uniqueRuntimeDeps)
			info.SrcUris = nonNilUri(info.SrcUris)

			infoMap[packageName] = []*packageInfo{&info}
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
