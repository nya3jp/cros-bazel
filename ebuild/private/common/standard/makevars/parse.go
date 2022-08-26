package makevars

import (
	"fmt"
	"os"

	"mvdan.cc/sh/v3/expand"
	"mvdan.cc/sh/v3/syntax"
)

func ParseMakeDefaults(path string, vars Vars) error {
	file, err := os.Open(path)
	if err != nil {
		return fmt.Errorf("%s: %w", path, err)
	}
	defer file.Close()

	parser := syntax.NewParser(syntax.Variant(syntax.LangBash))
	parsed, err := parser.Parse(file, path)
	if err != nil {
		return fmt.Errorf("%s: %w", path, err)
	}

	newVars := vars.Copy()

	for _, stmt := range parsed.Stmts {
		call, ok := stmt.Cmd.(*syntax.CallExpr)
		if !ok {
			return fmt.Errorf("%s:%s: unsupported statement", path, stmt.Pos())
		}

		// Reject calls.
		if len(call.Args) >= 1 {
			return fmt.Errorf("%s:%s: unsupported call", path, call.Pos())
		}

		// Process variable assignments.
		for _, assign := range call.Assigns {
			name := assign.Name.Value
			if assign.Append || assign.Array != nil || assign.Index != nil || assign.Naked {
				return fmt.Errorf("%s:%s: unsupported assignment", path, assign.Pos())
			}

			cfg := &expand.Config{Env: environ(newVars)}
			value, err := expand.Literal(cfg, assign.Value)
			if err != nil {
				return fmt.Errorf("%s:%s: %w", path, assign.Value.Pos(), err)
			}

			newVars[name] = value
		}
	}

	MergeTo(vars, newVars)
	return nil
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

	vars := Vars{}

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

			cfg := &expand.Config{Env: environ(vars)}
			value, err := expand.Literal(cfg, assign.Value)
			if err != nil {
				return nil, fmt.Errorf("%s:%s: %w", path, assign.Value.Pos(), err)
			}

			vars[name] = value
		}
	}

	return vars, nil
}

type environ Vars

var _ expand.Environ = environ{}

func (e environ) Get(name string) expand.Variable {
	value, ok := e[name]
	if !ok {
		return expand.Variable{}
	}
	return expand.Variable{
		Local: true,
		Kind:  expand.String,
		Str:   value,
	}
}

func (e environ) Each(f func(name string, v expand.Variable) bool) {
	for name := range e {
		if !f(name, e.Get(name)) {
			return
		}
	}
}

func (e environ) Set(name string, v expand.Variable) {
	if v.Kind != expand.String {
		// Silently ignore non-string variables.
		return
	}
	e[name] = v.Str
}
