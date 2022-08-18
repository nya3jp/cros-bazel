package dependency

import (
	"fmt"
	"strings"
)

type Deps struct {
	expr *AllOf
}

func NewDeps(expr *AllOf) *Deps {
	return &Deps{expr: expr}
}

func (d *Deps) Expr() *AllOf {
	return d.expr
}

func (d *Deps) String() string {
	s := d.expr.String()
	return strings.TrimSuffix(strings.TrimPrefix(s, "( "), " )")
}

type Expr interface {
	isExpr()
	String() string
}

type AllOf struct {
	children []Expr
}

func NewAllOf(children []Expr) *AllOf {
	return &AllOf{children: children}
}

func (d *AllOf) Children() []Expr { return append([]Expr(nil), d.children...) }

func (d *AllOf) isExpr() {}

func (d *AllOf) String() string {
	var substrings []string
	for _, child := range d.children {
		substrings = append(substrings, child.String())
	}
	return fmt.Sprintf("( %s )", strings.Join(substrings, " "))
}

type AnyOf struct {
	children []Expr
}

func NewAnyOf(children []Expr) *AnyOf {
	return &AnyOf{children: children}
}

func (d *AnyOf) Children() []Expr { return append([]Expr(nil), d.children...) }

func (d *AnyOf) isExpr() {}

func (d *AnyOf) String() string {
	var substrings []string
	for _, child := range d.children {
		substrings = append(substrings, child.String())
	}
	return fmt.Sprintf("|| ( %s )", strings.Join(substrings, " "))
}

type ExactlyOneOf struct {
	children []Expr
}

func NewExactlyOneOf(children []Expr) *ExactlyOneOf {
	return &ExactlyOneOf{children: children}
}

func (d *ExactlyOneOf) Children() []Expr { return append([]Expr(nil), d.children...) }

func (d *ExactlyOneOf) isExpr() {}

func (d *ExactlyOneOf) String() string {
	var substrings []string
	for _, child := range d.children {
		substrings = append(substrings, child.String())
	}
	return fmt.Sprintf("^^ ( %s )", strings.Join(substrings, " "))
}

type AtMostOneOf struct {
	children []Expr
}

func NewAtMostOneOf(children []Expr) *AtMostOneOf {
	return &AtMostOneOf{children: children}
}

func (d *AtMostOneOf) Children() []Expr { return append([]Expr(nil), d.children...) }

func (d *AtMostOneOf) isExpr() {}

func (d *AtMostOneOf) String() string {
	var substrings []string
	for _, child := range d.children {
		substrings = append(substrings, child.String())
	}
	return fmt.Sprintf("?? ( %s )", strings.Join(substrings, " "))
}

type UseConditional struct {
	name   string
	expect bool
	child  *AllOf
}

func NewUseConditional(name string, expect bool, child *AllOf) *UseConditional {
	return &UseConditional{
		name:   name,
		expect: expect,
		child:  child,
	}
}

func (d *UseConditional) Name() string  { return d.name }
func (d *UseConditional) Child() *AllOf { return d.child }
func (d *UseConditional) Expect() bool  { return d.expect }

func (d *UseConditional) isExpr() {}

func (d *UseConditional) String() string {
	cond := d.name
	if !d.expect {
		cond = "!" + cond
	}
	return fmt.Sprintf("%s %s", cond, d.child.String())
}

type Package struct {
	atom   *Atom
	blocks int
}

func NewPackage(atom *Atom, blocks int) *Package {
	return &Package{
		atom:   atom,
		blocks: blocks,
	}
}

func (p *Package) Atom() *Atom { return p.atom }
func (p *Package) Blocks() int { return p.blocks }

func (p *Package) isExpr() {}

func (p *Package) String() string {
	return strings.Repeat("!", p.blocks) + p.atom.String()
}
