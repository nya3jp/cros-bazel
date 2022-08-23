package dependency

import (
	"fmt"
	"sort"
)

func ResolveUse(deps *Deps, use map[string]struct{}) *Deps {
	return NewDeps(resolveUseExpr(deps.Expr(), use).(*AllOf))
}

func resolveUseExpr(expr Expr, use map[string]struct{}) Expr {
	switch expr := expr.(type) {
	case *AllOf:
		return NewAllOf(resolveUseExprs(expr.Children(), use))

	case *AnyOf:
		return NewAnyOf(resolveUseExprs(expr.Children(), use))

	case *ExactlyOneOf:
		return NewExactlyOneOf(resolveUseExprs(expr.Children(), use))

	case *AtMostOneOf:
		return NewAtMostOneOf(resolveUseExprs(expr.Children(), use))

	case *UseConditional:
		if _, ok := use[expr.Name()]; ok == expr.Expect() {
			return resolveUseExpr(expr.Child(), use)
		}
		return constTrue

	case *Package:
		return expr

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func resolveUseExprs(exprs []Expr, use map[string]struct{}) []Expr {
	var newExprs []Expr
	for _, expr := range exprs {
		newExprs = append(newExprs, resolveUseExpr(expr, use))
	}
	return newExprs
}

type NondeterministicDeps struct {
	Deps Expr
}

func (e *NondeterministicDeps) Error() string {
	return fmt.Sprintf("non-deterministic dependencies: %s", e.Deps.String())
}

func DeterministicDeps(deps *Deps) ([]*Atom, error) {
	return deterministicDepsExpr(deps.Expr())
}

func deterministicDepsExpr(expr Expr) ([]*Atom, error) {
	switch expr := expr.(type) {
	case *AllOf:
		return deterministicDepsExprs(expr.Children())

	case *AnyOf:
		if len(expr.children) >= 2 {
			return nil, &NondeterministicDeps{Deps: expr}
		}
		return deterministicDepsExprs(expr.Children())

	case *ExactlyOneOf:
		if len(expr.children) != 1 {
			return nil, &NondeterministicDeps{Deps: expr}
		}
		return deterministicDepsExprs(expr.Children())

	case *AtMostOneOf:
		if len(expr.children) != 0 {
			return nil, &NondeterministicDeps{Deps: expr}
		}
		return nil, nil

	case *UseConditional:
		return deterministicDepsExpr(expr.Child())

	case *Package:
		if expr.Blocks() > 0 {
			return nil, nil
		}
		return []*Atom{expr.Atom()}, nil

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func deterministicDepsExprs(children []Expr) ([]*Atom, error) {
	var atoms []*Atom
	for _, child := range children {
		childAtoms, err := deterministicDepsExpr(child)
		if err != nil {
			return nil, err
		}
		atoms = append(atoms, childAtoms...)
	}
	return atoms, nil
}

func Atoms(deps *Deps) []*Atom {
	return atomsExpr(deps.Expr())
}

func atomsExpr(expr Expr) []*Atom {
	switch expr := expr.(type) {
	case *AllOf, *AnyOf, *ExactlyOneOf, *AtMostOneOf:
		return childAtomsExpr(expr.(interface{ Children() []Expr }).Children())

	case *UseConditional:
		return atomsExpr(expr.Child())

	case *Package:
		if expr.Blocks() > 0 {
			return nil
		}
		return []*Atom{expr.Atom()}

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func childAtomsExpr(children []Expr) []*Atom {
	var atoms []*Atom
	for _, child := range children {
		atoms = append(atoms, atomsExpr(child)...)
	}
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
	return NewDeps(ensureAllOf(simplifyExpr(deps.Expr())))
}

func simplifyExpr(expr Expr) Expr {
	switch expr := expr.(type) {
	case *AllOf:
		newChildren := simplifyExprs(expr.Children(), isConstTrue, true)
		if len(newChildren) == 1 {
			return newChildren[0]
		}
		return NewAllOf(newChildren)

	case *AnyOf:
		newChildren := simplifyExprs(expr.Children(), isConstFalse, false)
		if len(newChildren) == 1 {
			return newChildren[0]
		}
		return NewAnyOf(newChildren)

	case *ExactlyOneOf:
		newChildren := simplifyExprs(expr.Children(), isConstFalse, false)
		trues := 0
		for _, newChild := range newChildren {
			if isConstTrue(newChild) {
				trues++
			}
		}
		if trues >= 2 {
			return constFalse
		}
		if trues == 1 {
			if len(newChildren) == 1 {
				return constTrue
			}
			// TODO: Consider simplifying more.
			return NewExactlyOneOf(newChildren)
		}
		if len(newChildren) == 1 {
			return newChildren[0]
		}
		if len(newChildren) == 0 {
			return constFalse
		}
		return NewExactlyOneOf(newChildren)

	case *AtMostOneOf:
		newChildren := simplifyExprs(expr.Children(), isConstFalse, false)
		trues := 0
		for _, newChild := range newChildren {
			if isConstTrue(newChild) {
				trues++
			}
		}
		if trues >= 2 {
			return constFalse
		}
		if trues == 1 {
			if len(newChildren) == 1 {
				return constTrue
			}
			// TODO: Consider simplifying more.
			return NewAtMostOneOf(newChildren)
		}
		if len(newChildren) <= 1 {
			return constTrue
		}
		return NewAtMostOneOf(newChildren)

	case *UseConditional:
		newChild := simplifyExpr(expr.Child())
		if isConstTrue(newChild) {
			return constTrue
		}
		if isConstFalse(newChild) {
			return constFalse
		}
		return NewUseConditional(expr.Name(), expr.Expect(), ensureAllOf(newChild))

	case *Package:
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
		if newAllOf, ok := newChild.(*AllOf); ok {
			newChildren = append(newChildren, newAllOf.Children()...)
		} else {
			newChildren = append(newChildren, newChild)
		}
	}
	return newChildren
}

func RemoveBlocks(deps *Deps) (*Deps, error) {
	expr, err := removeBlocksExpr(deps.Expr())
	if err != nil {
		return nil, err
	}
	return NewDeps(expr.(*AllOf)), nil
}

func removeBlocksExpr(expr Expr) (Expr, error) {
	switch expr := expr.(type) {
	case *AllOf:
		newChildren, _, err := removeBlocksExprs(expr.Children())
		if err != nil {
			return nil, err
		}
		return NewAllOf(newChildren), nil

	case *AnyOf:
		newChildren, removed, err := removeBlocksExprs(expr.Children())
		if err != nil {
			return nil, err
		}
		if removed {
			return constTrue, nil
		}
		return NewAnyOf(newChildren), nil

	case *ExactlyOneOf:
		newChildren, removed, err := removeBlocksExprs(expr.Children())
		if err != nil {
			return nil, err
		}
		if removed {
			return nil, fmt.Errorf("cannot remove blocks under ExactlyOneOf")
		}
		return NewExactlyOneOf(newChildren), nil

	case *AtMostOneOf:
		newChildren, removed, err := removeBlocksExprs(expr.Children())
		if err != nil {
			return nil, err
		}
		if removed {
			return nil, fmt.Errorf("cannot remove blocks under AtMostOneOf")
		}
		return NewAtMostOneOf(newChildren), nil

	case *UseConditional:
		newChild, err := removeBlocksExpr(expr.Child())
		if err != nil {
			return nil, err
		}
		return NewUseConditional(expr.Name(), expr.Expect(), newChild.(*AllOf)), nil

	case *Package:
		if expr.Blocks() > 0 {
			return constTrue, nil
		}
		return expr, nil

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func removeBlocksExprs(children []Expr) (newChildren []Expr, removed bool, err error) {
	for _, child := range children {
		newChild, err := removeBlocksExpr(child)
		if err != nil {
			return nil, false, err
		}
		if isConstTrue(newChild) {
			removed = true
		} else {
			newChildren = append(newChildren, newChild)
		}
	}
	return newChildren, removed, nil
}

func MapPackage(deps *Deps, f func(p *Package) *Package) *Deps {
	return NewDeps(mapPackageExpr(deps.Expr(), f).(*AllOf))
}

func mapPackageExpr(expr Expr, f func(p *Package) *Package) Expr {
	switch expr := expr.(type) {
	case *AllOf:
		return NewAllOf(mapPackageExprs(expr.Children(), f))

	case *AnyOf:
		return NewAnyOf(mapPackageExprs(expr.Children(), f))

	case *ExactlyOneOf:
		return NewExactlyOneOf(mapPackageExprs(expr.Children(), f))

	case *AtMostOneOf:
		return NewAtMostOneOf(mapPackageExprs(expr.Children(), f))

	case *UseConditional:
		return NewUseConditional(expr.Name(), expr.Expect(), mapPackageExpr(expr.Child(), f).(*AllOf))

	case *Package:
		return f(expr)

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func mapPackageExprs(children []Expr, f func(p *Package) *Package) []Expr {
	var newChildren []Expr
	for _, child := range children {
		newChildren = append(newChildren, mapPackageExpr(child, f))
	}
	return newChildren
}

func FilterForPackage(deps *Deps, packageName string) (*Deps, error) {
	newExpr, err := filterForPackageExpr(deps.Expr(), packageName)
	if err != nil {
		return nil, err
	}
	if newExpr == nil {
		return NewDeps(NewAllOf(nil)), nil
	}
	return NewDeps(newExpr.(*AllOf)), nil
}

func filterForPackageExpr(expr Expr, packageName string) (Expr, error) {
	switch expr := expr.(type) {
	case *AllOf:
		newChildren, err := filterForPackageExprs(expr.Children(), packageName)
		if err != nil {
			return nil, err
		}
		if len(newChildren) == 0 {
			return nil, nil
		}
		return NewAllOf(newChildren), nil

	case *AnyOf:
		newChildren, err := filterForPackageExprs(expr.Children(), packageName)
		if err != nil {
			return nil, err
		}
		if len(newChildren) == 0 {
			return nil, nil
		}
		return NewAnyOf(newChildren), nil

	case *ExactlyOneOf, *AtMostOneOf:
		return nil, fmt.Errorf("can not filter dependencies for packege with %T", expr)

	case *UseConditional:
		newChild, err := filterForPackageExpr(expr.Child(), packageName)
		if err != nil {
			return nil, err
		}
		if newChild == nil {
			return nil, nil
		}
		return NewUseConditional(expr.Name(), expr.Expect(), newChild.(*AllOf)), nil

	case *Package:
		if expr.Atom().PackageName() != packageName {
			return nil, nil
		}
		return expr, nil

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

func filterForPackageExprs(exprs []Expr, packageName string) ([]Expr, error) {
	var newExprs []Expr
	for _, expr := range exprs {
		newExpr, err := filterForPackageExpr(expr, packageName)
		if err != nil {
			return nil, err
		}
		if newExpr != nil {
			newExprs = append(newExprs, newExpr)
		}
	}
	return newExprs, nil
}

func Satisfies(deps *Deps, pkgs []*TargetPackage, use map[string]struct{}) bool {
	pkgMap := make(map[string][]*TargetPackage)
	for _, pkg := range pkgs {
		pkgMap[pkg.Name] = append(pkgMap[pkg.Name], pkg)
	}
	return satisfiesExpr(deps.Expr(), pkgMap, use)
}

func satisfiesExpr(expr Expr, pkgMap map[string][]*TargetPackage, use map[string]struct{}) bool {
	switch expr := expr.(type) {
	case *AllOf:
		for _, child := range expr.Children() {
			if !satisfiesExpr(child, pkgMap, use) {
				return false
			}
		}
		return true

	case *AnyOf:
		for _, child := range expr.Children() {
			if satisfiesExpr(child, pkgMap, use) {
				return true
			}
		}
		return false

	case *ExactlyOneOf:
		count := 0
		for _, child := range expr.Children() {
			if satisfiesExpr(child, pkgMap, use) {
				count++
			}
		}
		return count == 1

	case *AtMostOneOf:
		count := 0
		for _, child := range expr.Children() {
			if satisfiesExpr(child, pkgMap, use) {
				count++
			}
		}
		return count <= 1

	case *UseConditional:
		if _, ok := use[expr.Name()]; ok != expr.Expect() {
			return true
		}
		return satisfiesExpr(expr.Child(), pkgMap, use)

	case *Package:
		for _, pkg := range pkgMap[expr.Atom().name] {
			if expr.Atom().Match(pkg) {
				return true
			}
		}
		return false

	default:
		panic(fmt.Sprintf("unknown Expr type %T", expr))
	}
}

var (
	constTrue  = NewAllOf(nil)
	constFalse = NewAnyOf(nil)
)

func isConstTrue(e Expr) bool {
	if e, ok := e.(*AllOf); ok && len(e.children) == 0 {
		return true
	}
	return false
}

func isConstFalse(e Expr) bool {
	if e, ok := e.(*AnyOf); ok && len(e.children) == 0 {
		return true
	}
	return false
}

func ensureAllOf(expr Expr) *AllOf {
	if expr, ok := expr.(*AllOf); ok {
		return expr
	}
	return NewAllOf([]Expr{expr})
}
