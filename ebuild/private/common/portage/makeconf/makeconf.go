package makeconf

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"

	"mvdan.cc/sh/v3/expand"
	"mvdan.cc/sh/v3/syntax"
)

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

func parseFile(path string, env environ) error {
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

	for _, stmt := range parsed.Stmts {
		call, ok := stmt.Cmd.(*syntax.CallExpr)
		if !ok {
			return fmt.Errorf("%s:%s: unsupported statement", path, stmt.Pos())
		}

		// Process "source" keyword.
		if len(call.Args) >= 1 && len(call.Args[0].Parts) == 1 {
			if keyword, ok := call.Args[0].Parts[0].(*syntax.Lit); ok && keyword.Value == "source" {
				if len(call.Args) != 2 {
					return fmt.Errorf("%s:%s: source keyword needs exactly one file name", path, call.Pos())
				}

				cfg := &expand.Config{Env: env}
				relPath, err := expand.Literal(cfg, call.Args[1])
				if err != nil {
					return fmt.Errorf("%s:%s: %w", path, call.Args[1].Pos(), err)
				}

				var newPath string
				if filepath.IsAbs(relPath) {
					newPath = relPath
				} else {
					newPath = filepath.Join(filepath.Dir(path), relPath)
				}

				if err := parseFile(newPath, env); err != nil {
					return err
				}
				continue
			}
		}

		// Reject other calls.
		if len(call.Args) >= 1 {
			return fmt.Errorf("%s:%s: unsupported call", path, call.Pos())
		}

		// Process variable assignment.
		for _, assign := range call.Assigns {
			name := assign.Name.Value
			if assign.Append || assign.Array != nil || assign.Index != nil || assign.Naked || assign.Value == nil {
				return fmt.Errorf("%s:%s: unsupported assignment", path, assign.Pos())
			}

			cfg := &expand.Config{Env: env}
			value, err := expand.Literal(cfg, assign.Value)
			if err != nil {
				return fmt.Errorf("%s:%s: %w", path, assign.Value.Pos(), err)
			}

			env.Set(name, expand.Variable{
				Kind: expand.String,
				Str:  value,
			})

		}
	}
	return nil
}

func Parse(path string) (map[string]string, error) {
	env := make(environ)
	if err := parseFile(path, env); err != nil {
		return nil, err
	}

	vars := make(map[string]string)
	for name, v := range env {
		if v.IsSet() && v.Kind == expand.String {
			vars[name] = v.Str
		}
	}
	return vars, nil
}

func ParseDefaults(rootDir string) (map[string]string, error) {
	vars := make(map[string]string)
	for _, relPath := range []string{"etc/make.conf", "etc/portage/make.conf"} {
		path := filepath.Join(rootDir, relPath)
		if _, err := os.Stat(path); errors.Is(err, fs.ErrNotExist) {
			continue
		}

		subVars, err := Parse(path)
		if err != nil {
			return nil, err
		}

		for name, value := range subVars {
			vars[name] = value
		}
	}
	return vars, nil
}
