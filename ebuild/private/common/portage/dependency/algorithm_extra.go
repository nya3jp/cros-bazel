// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package dependency

import (
	"fmt"
	"sort"
)

func ResolveUse(deps *Deps, use map[string]bool) *Deps {
	return Map(deps, func(expr Expr) Expr {
		if cond, ok := expr.(*UseConditional); ok {
			if use[cond.Name()] == cond.Expect() {
				return cond.Child()
			}
			return ConstTrue
		}
		return expr
	})
}

func Atoms(deps *Deps) []*Atom {
	var atoms []*Atom
	ForEach(deps, func(expr Expr) {
		if pkg, ok := expr.(*Package); ok {
			if pkg.Blocks() == 0 {
				atoms = append(atoms, pkg.Atom())
			}
		}
	})
	return atoms
}

func Packages(deps *Deps) []string {
	pkgSet := make(map[string]struct{})
	for _, atom := range Atoms(deps) {
		pkgSet[atom.PackageName()] = struct{}{}
	}
	pkgs := make([]string, 0, len(pkgSet))
	for pkg := range pkgSet {
		pkgs = append(pkgs, pkg)
	}
	sort.Strings(pkgs)
	return pkgs
}

func Simplify(deps *Deps) *Deps {
	return NewDeps(EnsureAllOf(simplifyExpr(deps.Expr())))
}

func simplifyExpr(expr Expr) Expr {
	switch expr := expr.(type) {
	case *AllOf:
		newChildren := simplifyExprs(expr.Children(), IsConstTrue, true)
		if len(newChildren) == 1 {
			return newChildren[0]
		}
		for _, newChild := range newChildren {
			if IsConstFalse(newChild) {
				return ConstFalse
			}
		}
		return NewAllOf(newChildren)

	case *AnyOf:
		newChildren := simplifyExprs(expr.Children(), IsConstFalse, false)
		if len(newChildren) == 1 {
			return newChildren[0]
		}
		for _, newChild := range newChildren {
			if IsConstTrue(newChild) {
				return ConstTrue
			}
		}
		return NewAnyOf(newChildren)

	case *ExactlyOneOf:
		newChildren := simplifyExprs(expr.Children(), IsConstFalse, false)
		trues := 0
		for _, newChild := range newChildren {
			if IsConstTrue(newChild) {
				trues++
			}
		}
		if trues >= 2 {
			return ConstFalse
		}
		if trues == 1 {
			if len(newChildren) == 1 {
				return ConstTrue
			}
			// TODO: Consider simplifying more.
			return NewExactlyOneOf(newChildren)
		}
		if len(newChildren) == 1 {
			return newChildren[0]
		}
		if len(newChildren) == 0 {
			return ConstFalse
		}
		return NewExactlyOneOf(newChildren)

	case *AtMostOneOf:
		newChildren := simplifyExprs(expr.Children(), IsConstFalse, false)
		trues := 0
		for _, newChild := range newChildren {
			if IsConstTrue(newChild) {
				trues++
			}
		}
		if trues >= 2 {
			return ConstFalse
		}
		if trues == 1 {
			if len(newChildren) == 1 {
				return ConstTrue
			}
			// TODO: Consider simplifying more.
			return NewAtMostOneOf(newChildren)
		}
		if len(newChildren) <= 1 {
			return ConstTrue
		}
		return NewAtMostOneOf(newChildren)

	case *UseConditional:
		newChild := simplifyExpr(expr.Child())
		if IsConstTrue(newChild) {
			return ConstTrue
		}
		if IsConstFalse(newChild) {
			return ConstFalse
		}
		return NewUseConditional(expr.Name(), expr.Expect(), EnsureAllOf(newChild))

	case *Package:
		return expr

	case *Uri:
		return expr

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func simplifyExprs(children []Expr, omit func(Expr) bool, inlineAllOf bool) []Expr {
	var newChildren []Expr
	for _, child := range children {
		newChild := simplifyExpr(child)
		if omit != nil && omit(newChild) {
			continue
		}
		if newAllOf, ok := newChild.(*AllOf); ok && inlineAllOf {
			newChildren = append(newChildren, newAllOf.Children()...)
		} else {
			newChildren = append(newChildren, newChild)
		}
	}
	return newChildren
}
