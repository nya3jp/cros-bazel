// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package depparse

import (
	"fmt"

	"cros.local/bazel/ebuild/private/common/portage"
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

func parseSimpleDeps(deps *dependency.Deps) ([]*dependency.Atom, bool) {
	var atoms []*dependency.Atom
	for _, expr := range deps.Expr().Children() {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return nil, false
		}
		if pkg.Blocks() != 0 {
			return nil, false
		}
		atoms = append(atoms, pkg.Atom())
	}
	return atoms, true
}

func Parse(deps *dependency.Deps, pkg *packages.Package, resolver *portage.Resolver) ([]*dependency.Atom, error) {
	if forceDeps, ok := forceDepsPackages[pkg.Name()]; ok {
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

	simpleDeps, err := simplifyDeps(deps, pkg, resolver)
	if err != nil {
		return nil, fmt.Errorf("failed simplifying deps: %s: %w", deps.String(), err)
	}

	atoms, ok := parseSimpleDeps(simpleDeps)
	if !ok {
		return nil, fmt.Errorf("failed parsing simplify deps as it is not very simple: %s => %s", deps.String(), simpleDeps.String())
	}

	return atoms, nil
}
