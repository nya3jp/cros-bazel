// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package dependency

import (
	"fmt"
)

var (
	ConstTrue  = NewAllOf(nil)
	ConstFalse = NewAnyOf(nil)
)

func IsConstTrue(e Expr) bool {
	if e, ok := e.(*AllOf); ok && len(e.children) == 0 {
		return true
	}
	return false
}

func IsConstFalse(e Expr) bool {
	if e, ok := e.(*AnyOf); ok && len(e.children) == 0 {
		return true
	}
	return false
}

func EnsureAllOf(expr Expr) *AllOf {
	if expr, ok := expr.(*AllOf); ok {
		return expr
	}
	return NewAllOf([]Expr{expr})
}

func MapChildrenWithError(expr Expr, f func(expr Expr) (Expr, error)) (Expr, error) {
	switch expr := expr.(type) {
	case *AllOf:
		newChildren, err := mapChildrenWithError(expr.Children(), f)
		if err != nil {
			return nil, err
		}
		return NewAllOf(newChildren), nil

	case *AnyOf:
		newChildren, err := mapChildrenWithError(expr.Children(), f)
		if err != nil {
			return nil, err
		}
		return NewAnyOf(newChildren), nil

	case *ExactlyOneOf:
		newChildren, err := mapChildrenWithError(expr.Children(), f)
		if err != nil {
			return nil, err
		}
		return NewExactlyOneOf(newChildren), nil

	case *AtMostOneOf:
		newChildren, err := mapChildrenWithError(expr.Children(), f)
		if err != nil {
			return nil, err
		}
		return NewAtMostOneOf(newChildren), nil

	case *UseConditional:
		newExpr, err := f(expr.Child())
		if err != nil {
			return nil, err
		}
		return NewUseConditional(expr.Name(), expr.Expect(), EnsureAllOf(newExpr)), nil

	case *Package:
		return expr, nil

	case *Uri:
		return expr, nil

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func mapChildrenWithError(exprs []Expr, f func(expr Expr) (Expr, error)) ([]Expr, error) {
	var newExprs []Expr
	for _, expr := range exprs {
		newExpr, err := f(expr)
		if err != nil {
			return nil, err
		}
		newExprs = append(newExprs, newExpr)
	}
	return newExprs, nil
}

func MapChildren(expr Expr, f func(expr Expr) Expr) Expr {
	newExpr, _ := MapChildrenWithError(expr, func(expr Expr) (Expr, error) {
		return f(expr), nil
	})
	return newExpr
}

func MapWithError(deps *Deps, f func(expr Expr) (Expr, error)) (*Deps, error) {
	var ff func(expr Expr) (Expr, error)
	ff = func(expr Expr) (Expr, error) {
		newExpr, err := MapChildrenWithError(expr, ff)
		if err != nil {
			return nil, err
		}
		return f(newExpr)
	}
	newExpr, err := ff(deps.Expr())
	if err != nil {
		return nil, err
	}
	return NewDeps(EnsureAllOf(newExpr)), nil
}

func Map(deps *Deps, f func(expr Expr) Expr) *Deps {
	newDeps, _ := MapWithError(deps, func(expr Expr) (Expr, error) {
		return f(expr), nil
	})
	return newDeps
}

func ForEach(deps *Deps, f func(expr Expr)) {
	Map(deps, func(expr Expr) Expr {
		f(expr)
		return expr
	})
}
