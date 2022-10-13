// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package depparse

import (
	"fmt"
	"sort"

	"cros.local/bazel/ebuild/private/common/portage"
	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/packages"
)

// HACK: Hard-code several package info.
// TODO: Remove these hacks.
var (
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
		"virtual/yacc":                  {"sys-devel/bison"},
		// TODO: Figure out why simplifyDeps doesn't compute this correctly
		"virtual/perl-ExtUtils-MakeMaker": {"dev-lang/perl"},
	}
)

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

func filterPackages(pkgs []string, provided []*config.TargetPackage) []string {
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

func Parse(deps *dependency.Deps, pkg *packages.Package, resolver *portage.Resolver) ([]string, error) {
	simpleDeps := simplifyDeps(deps, pkg.Uses(), pkg.Name())

	parsedDeps, ok := forceDepsPackages[pkg.Name()]
	if !ok {
		parsedDeps, ok = parseSimpleDeps(simpleDeps)
		if !ok {
			return nil, fmt.Errorf("cannot simplify deps: %s", simpleDeps.String())
		}
	}

	parsedDeps = filterPackages(parsedDeps, resolver.ProvidedPackages())

	sort.Strings(parsedDeps)

	return parsedDeps, nil
}
