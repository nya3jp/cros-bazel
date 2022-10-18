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

	deps = mergeOverlappedDeps(deps)

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

// mergeOverlappedDeps simplifies deps by merging redundantly overlapped
// package dependencies.
//
// Example:
//
//	>=dev-libs/openssl-1.0 >=dev-libs/openssl-1.1  ==>  >=dev-libs/openssl-1.1
func mergeOverlappedDeps(deps *dependency.Deps) *dependency.Deps {
	deps = dependency.Simplify(deps)
	return dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		var allOf bool
		var children []dependency.Expr
		switch expr := expr.(type) {
		case *dependency.AllOf:
			allOf = true
			children = expr.Children()
		case *dependency.AnyOf:
			allOf = false
			children = expr.Children()
		default:
			return expr
		}

		// First, index atoms by package names.
		originalsMap := make(map[string][]*dependency.Atom)

		for _, child := range children {
			pkg, ok := child.(*dependency.Package)
			if !ok {
				continue
			}
			atom := pkg.Atom()
			packageName := atom.PackageName()
			originalsMap[packageName] = append(originalsMap[packageName], atom)
		}

		// Decide which packages to rewrite.
		rewriteMap := make(map[string]*dependency.Atom)

		for packageName, atoms := range originalsMap {
			if len(atoms) <= 1 {
				continue
			}

			atom0 := atoms[0]
			// Handle >= operator only (for now).
			if atom0.VersionOperator() != dependency.OpGreaterEqual {
				continue
			}
			if atom0.Wildcard() {
				continue
			}

			// Find atoms with highest/lowest versions.
			hi, lo, ok := func() (hi, lo *dependency.Atom, ok bool) {
				matchExceptVersion := func(atom *dependency.Atom) bool {
					if atom.VersionOperator() != atom0.VersionOperator() {
						return false
					}
					if atom.Wildcard() != atom0.Wildcard() {
						return false
					}
					if atom.SlotDep() != atom0.SlotDep() {
						return false
					}
					if len(atom.UseDeps()) != len(atom0.UseDeps()) {
						return false
					}
					for i, use0 := range atom0.UseDeps() {
						use := atom.UseDeps()[i]
						if use.String() != use0.String() {
							return false
						}
					}
					return true
				}

				hi, lo = atom0, atom0

				for _, atom := range atoms {
					if !matchExceptVersion(atom) {
						return nil, nil, false
					}
					if atom.Version().Compare(hi.Version()) > 0 {
						hi = atom
					}
					if atom.Version().Compare(lo.Version()) < 0 {
						lo = atom
					}
				}
				return hi, lo, true
			}()
			if !ok {
				continue
			}

			var rewrite *dependency.Atom
			if allOf {
				rewrite = hi
			} else {
				rewrite = lo
			}
			rewriteMap[packageName] = rewrite
		}

		// Return early if there is no package to rewrite.
		if len(rewriteMap) == 0 {
			return expr
		}

		// Perform actual rewrites.
		var newChildren []dependency.Expr
		for _, child := range children {
			pkg, ok := child.(*dependency.Package)
			if !ok {
				newChildren = append(newChildren, child)
				continue
			}

			packageName := pkg.Atom().PackageName()
			rewrite, ok := rewriteMap[packageName]
			if !ok {
				newChildren = append(newChildren, child)
				continue
			}
			if rewrite == nil {
				continue
			}

			// Rewrite the first occurrence, and drop all others.
			newChildren = append(newChildren, dependency.NewPackage(rewrite, 0))
			rewriteMap[packageName] = nil
		}

		if allOf {
			return dependency.NewAllOf(newChildren)
		}
		return dependency.NewAnyOf(newChildren)
	})
}
