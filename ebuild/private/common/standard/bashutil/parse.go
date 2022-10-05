// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package bashutil

import (
	"fmt"
	"io"
	"strings"

	"mvdan.cc/sh/v3/expand"
	"mvdan.cc/sh/v3/syntax"
)

func ParseSetOutput(r io.Reader) (map[string]string, error) {
	parser := syntax.NewParser(syntax.Variant(syntax.LangBash))
	parsed, err := parser.Parse(r, "")
	if err != nil {
		return nil, err
	}

	vars := make(map[string]string)

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

			cfg := &expand.Config{Env: Environ(vars)}

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
