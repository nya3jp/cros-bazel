// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package srcparse

import (
	"fmt"
	"sort"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/packages"
)

// HACK: Hard-code several package info.
// TODO: Remove these hacks.
var (
	additionalSrcPackages = map[string][]string{
		"app-accessibility/pumpkin":        {"@chromite//:src"},
		"chromeos-base/libchrome":          {"@chromite//:src"},
		"chromeos-languagepacks/tts-es-us": {"@chromite//:src"},
		"chromeos-base/sample-dlc":         {"@chromite//:src"},
		"dev-libs/modp_b64":                {"@chromite//:src"},
		"media-sound/sr-bt-dlc":            {"@chromite//:src"},
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

	// Not a cros-workon package
	if len(projects) == 0 {
		if additionalSrcPackages, ok := additionalSrcPackages[pkg.Name()]; ok {
			return additionalSrcPackages, nil
		}
		return nil, nil
	}

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
