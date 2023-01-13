// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package srcparse

import (
	"fmt"
	"sort"
	"strings"

	"cros.local/bazel/ebuild/private/common/depdata"
	"cros.local/bazel/ebuild/private/common/standard/packages"
)

var (
	chromiteEbuilds = map[string]struct{}{
		"dev-libs/gobject-introspection": {},
	}
)

func ExtractLocalPackages(pkg *packages.Package) ([]string, error) {
	metadata := pkg.Metadata()

	splitCrosWorkon := func(value string) []string {
		if value == "" {
			return nil
		}
		// The parser will return | concat arrays, so undo that here.
		return strings.Split(value, "|")
	}

	projects := splitCrosWorkon(metadata["CROS_WORKON_PROJECT"])

	localNames := splitCrosWorkon(metadata["CROS_WORKON_LOCALNAME"])

	if len(localNames) > 0 && len(localNames) != len(projects) {
		return nil, fmt.Errorf("number of elements in LOCAL_NAME (%d) and PROJECT (%d) don't match", len(localNames), len(projects))
	}

	subTrees := splitCrosWorkon(metadata["CROS_WORKON_SUBTREE"])

	if len(subTrees) > 0 && len(subTrees) != len(projects) {
		return nil, fmt.Errorf("number of elements in SUBTREE (%d) and PROJECT (%d) don't match", len(subTrees), len(projects))
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

		if project == "cros/platform/chromiumos-assets" && localName == "chromiumos-assets" {
			// ebuild is incorrect
			localName = "platform/chromiumos-assets"
		}

		var paths []string

		// If there is no local name, then we need to compute it
		if localName == "" {
			if strings.HasPrefix(project, "cros/") {
				paths = []string{strings.TrimPrefix(project, "cros/")}
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
			var newPaths []string

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

		if project == "cros/third_party/kernel" {
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

		if previousPath == path {
			// Some packages contain duplicate paths
			continue
		}
		previousPath = path
		srcDeps = append(srcDeps, "//"+path+":src")
	}

	if pkg.UsesEclass("chromium-source") {
		// TODO: We need use flags to add src-internal
		srcDeps = append(srcDeps, "@chrome//:src")
	}

	// The platform eclass calls `platform2.py` which requires chromite
	// The dlc eclass calls `build_dlc` which lives in chromite
	if _, ok := chromiteEbuilds[pkg.Name()]; ok || pkg.UsesEclass("platform") || pkg.UsesEclass("dlc") {
		srcDeps = append(srcDeps, "@chromite//:src")
	}

	sort.Strings(srcDeps)

	return srcDeps, nil
}

var dontIncludeSubTargets = map[string]struct{}{
	"//platform2": {},
}

// Not all packages use the same level of SUBTREE, some have deeper targets
// than others. This results in the packages that have a shallower SUBTREE
// missing out on the files defined in the deeper tree.
// To fix this we need to populate targets with the shallow tree with
// all the additional deeper paths.
//
// e.g.,
// iioservice requires //platform2/iioservice:src
// cros-camera-libs requires //platform2/iioservice/mojo:src
//
// When trying to build iioservice the mojo directory will be missing.
// So this code will add //platform2/iioservice/mojo:src to iioservice.
func FixupLocalSource(pkgInfoMap depdata.PackageInfoMap) {
	srcTargetMap := make(map[string][]*depdata.PackageInfo)

	for _, pkgInfo := range pkgInfoMap {
		for _, srcTarget := range pkgInfo.LocalSrc {
			srcTarget = strings.TrimSuffix(srcTarget, ":src")
			srcTargetMap[srcTarget] = append(srcTargetMap[srcTarget], pkgInfo)
		}
	}

	var allSrcTargets []string
	for srcTarget := range srcTargetMap {
		allSrcTargets = append(allSrcTargets, srcTarget)
	}

	sort.Strings(allSrcTargets)

	for i, parentSrcTarget := range allSrcTargets {
		if _, ok := dontIncludeSubTargets[parentSrcTarget]; ok {
			continue
		}

		for j := i + 1; j < len(allSrcTargets); j++ {
			srcTarget := allSrcTargets[j]
			if !strings.HasPrefix(srcTarget, parentSrcTarget) {
				break
			}

			relativeTarget := strings.TrimPrefix(srcTarget, parentSrcTarget)
			if !strings.HasPrefix(relativeTarget, "/") {
				// Make sure we match a path and not just a target that has the same
				// prefix
				break
			}

			fullSrcTarget := fmt.Sprintf("%s:src", srcTarget)
			for _, pkgInfo := range srcTargetMap[parentSrcTarget] {
				var found bool
				// Packages like autotest list a parent and child path in the
				// LOCALNAME. This will cause dups to get added, so make sure the
				// target doesn't already exist.
				for _, srcTarget := range pkgInfo.LocalSrc {
					if fullSrcTarget == srcTarget {
						found = true
						break
					}
				}

				if found {
					continue
				}

				pkgInfo.LocalSrc = append(pkgInfo.LocalSrc, fullSrcTarget)
				sort.Strings(pkgInfo.LocalSrc)
			}
		}
	}
}
