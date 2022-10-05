// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makevars

import (
	"fmt"
	"os"
	"path/filepath"

	"mvdan.cc/sh/v3/expand"
	"mvdan.cc/sh/v3/syntax"

	"cros.local/bazel/ebuild/private/common/standard/bashutil"
)

// Eval evaluates a bash-like file defining variables.
// env gives default values for variable expansions. It *IS* updated as we
// evaluate variable assignments.
// If allowSource is true, "source" directive is allowed to include other files.
// The returned Vars contains variables assigned in the file.
func Eval(path string, env Vars, allowSource bool) (Vars, error) {
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

	vars := make(Vars)

	for _, stmt := range parsed.Stmts {
		call, ok := stmt.Cmd.(*syntax.CallExpr)
		if !ok {
			return nil, fmt.Errorf("%s:%s: unsupported statement", path, stmt.Pos())
		}

		// Process "source" keyword.
		if allowSource && len(call.Args) >= 1 && len(call.Args[0].Parts) == 1 {
			if keyword, ok := call.Args[0].Parts[0].(*syntax.Lit); ok && keyword.Value == "source" {
				if len(call.Args) != 2 {
					return nil, fmt.Errorf("%s:%s: source keyword needs exactly one file name", path, call.Pos())
				}

				cfg := &expand.Config{Env: bashutil.Environ(env)}
				relPath, err := expand.Literal(cfg, call.Args[1])
				if err != nil {
					return nil, fmt.Errorf("%s:%s: %w", path, call.Args[1].Pos(), err)
				}

				var newPath string
				if filepath.IsAbs(relPath) {
					newPath = relPath
				} else {
					newPath = filepath.Join(filepath.Dir(path), relPath)
				}

				subvars, err := Eval(newPath, env, allowSource)
				if err != nil {
					return nil, err
				}
				for name, value := range subvars {
					vars[name] = value
				}
				continue
			}
		}

		// Reject other calls.
		if len(call.Args) >= 1 {
			return nil, fmt.Errorf("%s:%s: unsupported call", path, call.Pos())
		}

		// Process variable assignment.
		for _, assign := range call.Assigns {
			name := assign.Name.Value
			if assign.Append || assign.Array != nil || assign.Index != nil || assign.Naked || assign.Value == nil {
				return nil, fmt.Errorf("%s:%s: unsupported assignment", path, assign.Pos())
			}

			cfg := &expand.Config{Env: bashutil.Environ(env)}
			value, err := expand.Literal(cfg, assign.Value)
			if err != nil {
				return nil, fmt.Errorf("%s:%s: %w", path, assign.Value.Pos(), err)
			}

			env[name] = value
			vars[name] = value
		}
	}
	return vars, nil
}
