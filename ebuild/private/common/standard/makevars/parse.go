// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makevars

import (
	"fmt"
	"io"
	"os"
	"strings"

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

	newVars := vars.CopyNoIncrementalVars()

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

	vars.Merge(newVars)
	return nil
}

func ParseSetOutput(r io.Reader) (Vars, error) {
	parser := syntax.NewParser(syntax.Variant(syntax.LangBash))
	parsed, err := parser.Parse(r, "")
	if err != nil {
		return nil, err
	}

	vars := Vars{}

	for _, stmt := range parsed.Stmts {
		call, ok := stmt.Cmd.(*syntax.CallExpr)
		if !ok {
			return nil, fmt.Errorf("%s: unsupported statement", stmt.Pos())
		}

		// Reject calls.
		if len(call.Args) >= 1 {
			return nil, fmt.Errorf("%s: unsupported call", call.Pos())
		}

		// Process variable assignments.
		for _, assign := range call.Assigns {
			name := assign.Name.Value
			// Skip non CROS_WORKON_ or USED_ECLASSES arrays.
			if assign.Array != nil && !strings.HasPrefix(name, "CROS_WORKON_") && name != "USED_ECLASSES" {
				continue
			}
			if assign.Append || assign.Index != nil || assign.Naked {
				return nil, fmt.Errorf("%s: unsupported assignment", assign.Pos())
			}

			cfg := &expand.Config{Env: environ(vars)}

			if assign.Array == nil {
				value, err := expand.Literal(cfg, assign.Value)
				if err != nil {
					return nil, fmt.Errorf("%s: %w", assign.Value.Pos(), err)
				}

				vars[name] = value
			} else {
				var values []string

				for _, elem := range assign.Array.Elems {
					value, err := expand.Literal(cfg, elem.Value)
					if err != nil {
						return nil, fmt.Errorf("%s: %w", elem.Value.Pos(), err)
					}

					values = append(values, value)
				}

				// TODO: Make vars store an actual array
				vars[name] = strings.Join(values, "|")
			}
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
