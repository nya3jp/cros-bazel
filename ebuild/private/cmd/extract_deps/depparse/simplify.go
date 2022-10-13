// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package depparse

import (
	"log"

	"cros.local/bazel/ebuild/private/common/standard/dependency"
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
)

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
