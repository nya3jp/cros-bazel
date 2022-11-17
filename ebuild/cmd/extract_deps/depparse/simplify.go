// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package depparse

import (
	"strings"

	"cros.local/bazel/ebuild/private/common/portage"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/packages"
)

func isKnownUnavailable(atom *dependency.Atom) bool {
	// TODO: Remove this hack.
	switch atom.PackageName() {
	case "dev-lang/python":
		slot := atom.SlotDep()
		return slot == "2.7" || slot == "3.7" || slot == "3.8"
	case "dev-libs/libverto":
		// dev-libs/libverto is installed with libev, not libevent.
		useDeps := atom.UseDeps()
		return len(useDeps) >= 1 && useDeps[0].String() == "libevent"
	case "dev-python/m2crypto":
		useDeps := atom.UseDeps()
		return len(useDeps) >= 1 && strings.HasPrefix(useDeps[0].String(), "python_targets_python3_7")
	case "media-libs/jpeg":
		return true
	default:
		return false
	}
}

func simplifyDeps(deps *dependency.Deps, pkg *packages.Package, resolver *portage.Resolver) (*dependency.Deps, error) {
	deps = dependency.ResolveUse(deps, pkg.Uses())

	deps, err := rewritePackageDeps(deps, resolver)
	if err != nil {
		return nil, err
	}

	deps = pickAnyOfDeps(deps)

	deps = dependency.Simplify(deps)

	return deps, nil
}

// rewritePackageDeps rewrites package dependency atoms.
func rewritePackageDeps(deps *dependency.Deps, resolver *portage.Resolver) (*dependency.Deps, error) {
	return dependency.MapWithError(deps, func(expr dependency.Expr) (dependency.Expr, error) {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return expr, nil
		}

		// Remove blocks.
		if pkg.Blocks() > 0 {
			return dependency.ConstTrue, nil
		}

		atom := pkg.Atom()

		// HACK: Rewrite certain known unavailable packages.
		// TODO: Drop these hard-coded hack.
		if isKnownUnavailable(atom) {
			return dependency.ConstFalse, nil
		}

		// Remove provided packages.
		if resolver.IsProvided(atom) {
			return dependency.ConstTrue, nil
		}

		// Remove non-existent packages.
		candidates, err := resolver.Packages(atom)
		if err != nil {
			return nil, err
		}
		if len(candidates) == 0 {
			return dependency.ConstFalse, nil
		}

		return pkg, nil
	})
}

// pickAnyOfDeps resolves any-of dependencies by picking the first child.
func pickAnyOfDeps(deps *dependency.Deps) *dependency.Deps {
	deps = dependency.Simplify(deps)
	return dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		anyOf, ok := expr.(*dependency.AnyOf)
		if !ok {
			return expr
		}

		children := anyOf.Children()
		if len(children) == 0 {
			return expr
		}
		return children[0]
	})
}
