// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package dependency

import (
	"errors"
	"strings"

	"cros.local/bazel/portage/common/portage/dependency/internal/grammer"
)

func Parse(s string) (*Deps, error) {
	g, err := grammer.Parse(s)
	if err != nil {
		return nil, err
	}
	d, err := compileDeps(g)
	if err != nil {
		return nil, err
	}
	return d, nil
}

func compileDeps(g *grammer.AllOf) (*Deps, error) {
	expr, err := compileAllOf(g)
	if err != nil {
		return nil, err
	}
	return NewDeps(expr), nil
}

func compileAllOf(g *grammer.AllOf) (*AllOf, error) {
	var children []Expr
	for _, c := range g.Children {
		child, err := compileExpr(c)
		if err != nil {
			return nil, err
		}
		children = append(children, child)
	}
	return NewAllOf(children), nil
}

func compileAnyOf(g *grammer.AnyOf) (*AnyOf, error) {
	var children []Expr
	for _, c := range g.Children {
		child, err := compileExpr(c)
		if err != nil {
			return nil, err
		}
		children = append(children, child)
	}
	return NewAnyOf(children), nil
}

func compileExactlyOneOf(g *grammer.ExactlyOneOf) (*ExactlyOneOf, error) {
	var children []Expr
	for _, c := range g.Children {
		child, err := compileExpr(c)
		if err != nil {
			return nil, err
		}
		children = append(children, child)
	}
	return NewExactlyOneOf(children), nil
}

func compileAtMostOneOf(g *grammer.AtMostOneOf) (*AtMostOneOf, error) {
	var children []Expr
	for _, c := range g.Children {
		child, err := compileExpr(c)
		if err != nil {
			return nil, err
		}
		children = append(children, child)
	}
	return NewAtMostOneOf(children), nil
}

func compileUseConditional(g *grammer.UseConditional) (*UseConditional, error) {
	expect := !strings.HasPrefix(g.Condition, "!")
	name := strings.TrimSuffix(strings.TrimPrefix(g.Condition, "!"), "?")
	child, err := compileAllOf(g.Child)
	if err != nil {
		return nil, err
	}
	return NewUseConditional(name, expect, child), nil
}

func compilePackage(g *grammer.Package) (*Package, error) {
	const mark = "!"

	rest := g.Raw
	blocks := 0
	for strings.HasPrefix(rest, mark) {
		rest = strings.TrimPrefix(rest, mark)
		blocks++
	}

	a, err := ParseAtom(rest)
	if err != nil {
		return nil, err
	}
	return NewPackage(a, blocks), nil
}

func compileUri(g *grammer.Uri) (Expr, error) {
	return NewUri(g.Uri, g.FileName), nil
}

func compileExpr(g *grammer.Expr) (Expr, error) {
	switch {
	case g.AllOf != nil:
		return compileAllOf(g.AllOf)
	case g.AnyOf != nil:
		return compileAnyOf(g.AnyOf)
	case g.ExactlyOneOf != nil:
		return compileExactlyOneOf(g.ExactlyOneOf)
	case g.AtMostOneOf != nil:
		return compileAtMostOneOf(g.AtMostOneOf)
	case g.UseConditional != nil:
		return compileUseConditional(g.UseConditional)
	case g.Value != nil:
		v := g.Value
		switch {
		case v.Uri != nil:
			return compileUri(v.Uri)
		case v.Package != nil:
			return compilePackage(v.Package)
		default:
			return nil, errors.New("unknown value")
		}
	default:
		return nil, errors.New("unknown expr")
	}
}
