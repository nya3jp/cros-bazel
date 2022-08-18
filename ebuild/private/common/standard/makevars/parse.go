package makevars

import (
	"fmt"
	"os"

	"mvdan.cc/sh/v3/expand"
	"mvdan.cc/sh/v3/syntax"
)

func ParseMakeDefaults(path string) (Vars, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", path, err)
	}
	defer file.Close()

	parser := syntax.NewParser(syntax.Variant(syntax.LangBash))
	parsed, err := parser.Parse(file, path)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", path, err)
	}

	env := make(environ)

	for _, stmt := range parsed.Stmts {
		call, ok := stmt.Cmd.(*syntax.CallExpr)
		if !ok {
			return nil, fmt.Errorf("%s:%s: unsupported statement", path, stmt.Pos())
		}

		// Reject calls.
		if len(call.Args) >= 1 {
			return nil, fmt.Errorf("%s:%s: unsupported call", path, call.Pos())
		}

		// Process variable assignments.
		for _, assign := range call.Assigns {
			name := assign.Name.Value
			if assign.Append || assign.Array != nil || assign.Index != nil || assign.Naked {
				return nil, fmt.Errorf("%s:%s: unsupported assignment", path, assign.Pos())
			}

			cfg := &expand.Config{Env: env}
			value, err := expand.Literal(cfg, assign.Value)
			if err != nil {
				return nil, fmt.Errorf("%s:%s: %w", path, assign.Value.Pos(), err)
			}

			env.Set(name, expand.Variable{
				Kind: expand.String,
				Str:  value,
			})
		}
	}

	vars := make(Vars)
	for name, v := range env {
		if v.IsSet() && v.Kind == expand.String {
			vars[name] = v.Str
		}
	}
	return vars, nil
}

func ParseSetOutput(path string) (Vars, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", path, err)
	}
	defer file.Close()

	parser := syntax.NewParser(syntax.Variant(syntax.LangBash))
	parsed, err := parser.Parse(file, path)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", path, err)
	}

	env := make(environ)

	for _, stmt := range parsed.Stmts {
		call, ok := stmt.Cmd.(*syntax.CallExpr)
		if !ok {
			return nil, fmt.Errorf("%s:%s: unsupported statement", path, stmt.Pos())
		}

		// Reject calls.
		if len(call.Args) >= 1 {
			return nil, fmt.Errorf("%s:%s: unsupported call", path, call.Pos())
		}

		// Process variable assignments.
		for _, assign := range call.Assigns {
			name := assign.Name.Value
			// Skip arrays.
			if assign.Array != nil {
				continue
			}
			if assign.Append || assign.Index != nil || assign.Naked {
				return nil, fmt.Errorf("%s:%s: unsupported assignment", path, assign.Pos())
			}

			cfg := &expand.Config{Env: env}
			value, err := expand.Literal(cfg, assign.Value)
			if err != nil {
				return nil, fmt.Errorf("%s:%s: %w", path, assign.Value.Pos(), err)
			}

			env.Set(name, expand.Variable{
				Kind: expand.String,
				Str:  value,
			})
		}
	}

	vars := make(Vars)
	for name, v := range env {
		if v.IsSet() && v.Kind == expand.String {
			vars[name] = v.Str
		}
	}
	return vars, nil
}

type environ map[string]expand.Variable

var _ expand.Environ = environ{}

func (e environ) Get(name string) expand.Variable {
	return e[name]
}

func (e environ) Each(f func(name string, v expand.Variable) bool) {
	for name, v := range e {
		if !f(name, v) {
			return
		}
	}
}

func (e environ) Set(name string, v expand.Variable) {
	e[name] = v
}
