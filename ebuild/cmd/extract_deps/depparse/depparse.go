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
