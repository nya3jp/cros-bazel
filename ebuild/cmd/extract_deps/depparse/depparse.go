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

func parseSimpleDeps(deps *dependency.Deps) ([]*dependency.Atom, error) {
	var atoms []*dependency.Atom
	for _, expr := range deps.Expr().Children() {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return nil, fmt.Errorf("found non-package top-level dep: %s", expr.String())
		}
		if pkg.Blocks() != 0 {
			return nil, fmt.Errorf("found block dep: %s", pkg.String())
		}
		atoms = append(atoms, pkg.Atom())
	}
	return atoms, nil
}

func Parse(deps *dependency.Deps, pkg *packages.Package, resolver *portage.Resolver) ([]*dependency.Atom, error) {
	simpleDeps, err := simplifyDeps(deps, pkg, resolver)
	if err != nil {
		return nil, fmt.Errorf("failed simplifying deps: %s: %w", deps.String(), err)
	}

	atoms, err := parseSimpleDeps(simpleDeps)
	if err != nil {
		return nil, fmt.Errorf("failed parsing simplified deps as it is not very simple:\noriginal deps: %s\nsimplified deps: %s\n%v", deps.String(), simpleDeps.String(), err)
	}

	return atoms, nil
}
