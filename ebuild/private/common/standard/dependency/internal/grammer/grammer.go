// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package grammer

import (
	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
)

var lex = lexer.MustSimple([]lexer.SimpleRule{
	{Name: "whitespace", Pattern: `\s+`},
	{Name: "Parentheses", Pattern: `[()]`},
	{Name: "Operators", Pattern: `\|\||\^\^|\?\?`},
	{Name: "Condition", Pattern: `!?[A-Za-z0-9][A-Za-z0-9+_@-]*\?`},
	{Name: "Package", Pattern: `\S+`},
})

var parser = participle.MustBuild[AllOf](participle.Lexer(lex))

func Parse(s string) (*AllOf, error) {
	return parser.ParseString("", s)
}

type Expr struct {
	AllOf          *AllOf          `parser:"'(' @@ ')'"`
	AnyOf          *AnyOf          `parser:"| '||' '(' @@ ')'"`
	ExactlyOneOf   *ExactlyOneOf   `parser:"| '^^' '(' @@ ')'"`
	AtMostOneOf    *AtMostOneOf    `parser:"| '??' '(' @@ ')'"`
	UseConditional *UseConditional `parser:"| @@"`
	Package        *Package        `parser:"| @@"`
}

type AllOf struct {
	Children []*Expr `parser:"@@*"`
}

type AnyOf struct {
	Children []*Expr `parser:"@@*"`
}

type ExactlyOneOf struct {
	Children []*Expr `parser:"@@*"`
}

type AtMostOneOf struct {
	Children []*Expr `parser:"@@*"`
}

type UseConditional struct {
	Condition string `parser:"@Condition"`
	Child     *AllOf `parser:"'(' @@ ')'"`
}

type Package struct {
	Raw string `parser:"@Package"`
}
