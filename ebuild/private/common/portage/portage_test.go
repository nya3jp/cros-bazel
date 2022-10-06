// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package portage_test

import (
	"bufio"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"testing"

	"github.com/alessio/shellescape"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"

	"cros.local/bazel/ebuild/private/common/portage"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
)

func unpackSpec(specPath, outDir string) error {
	f, err := os.Open(specPath)
	if err != nil {
		return err
	}
	defer f.Close()

	var out *os.File // currently written file; can be nil
	defer func() {
		if out != nil {
			out.Close()
		}
	}()

	sc := bufio.NewScanner(f)

	for sc.Scan() {
		rawLine := sc.Text()
		line := strings.TrimSpace(rawLine)

		if !strings.HasPrefix(line, ">>> ") {
			if out != nil {
				fmt.Fprintln(out, rawLine)
			}
			continue
		}

		relPath := strings.TrimSpace(line[4:])
		var target string
		if v := strings.SplitN(relPath, " -> ", 2); len(v) == 2 {
			relPath, target = v[0], v[1]
		}

		if out != nil {
			out.Close()
			out = nil
		}

		path := filepath.Join(outDir, relPath)
		if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
			return err
		}

		if target != "" {
			if err := os.Symlink(target, path); err != nil {
				return err
			}
		} else {
			out, err = os.Create(path)
			if err != nil {
				return err
			}
		}
	}

	if err := sc.Err(); err != nil {
		return err
	}

	// Generate make.conf.
	makeConf := fmt.Sprintf("PORTDIR=%s\n", shellescape.Quote(filepath.Join(outDir, "overlay")))
	makeConfCustomPath := filepath.Join(outDir, "etc/make.conf.custom")
	if _, err := os.Stat(makeConfCustomPath); err == nil {
		makeConf += fmt.Sprintf("source %s\n", shellescape.Quote(makeConfCustomPath))
	}
	if err := os.WriteFile(filepath.Join(outDir, "etc/make.conf"), []byte(makeConf), 0644); err != nil {
		return err
	}

	return nil
}

func readExpectVars(path string) (makevars.Vars, error) {
	f, err := os.Open(path)
	if errors.Is(err, fs.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	defer f.Close()

	vars := make(makevars.Vars)
	sc := bufio.NewScanner(f)

	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		v := strings.SplitN(line, "=", 2)
		if len(v) != 2 {
			return nil, fmt.Errorf("corrupted line (must have exactly one equal sign): %s", line)
		}

		name := v[0]
		value, err := strconv.Unquote(v[1])
		if err != nil {
			return nil, fmt.Errorf("corrupted line (%w): %s", err, line)
		}

		vars[name] = value
	}

	if err := sc.Err(); err != nil {
		return nil, err
	}

	return vars, nil
}

func readExpectUse(path string) (map[string]string, error) {
	f, err := os.Open(path)
	if errors.Is(err, fs.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	defer f.Close()

	expectUse := make(map[string]string)
	sc := bufio.NewScanner(f)

	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		v := strings.Fields(line)
		if len(v) == 0 {
			continue
		}

		pkg := v[0]
		uses := append(strings.Fields(expectUse[pkg]), v[1:]...)
		expectUse[pkg] = strings.Join(uses, " ")
	}

	if err := sc.Err(); err != nil {
		return nil, err
	}

	return expectUse, nil
}

func TestAll(t *testing.T) {
	const testdataDir = "testdata"

	fis, err := os.ReadDir(testdataDir)
	if err != nil {
		t.Fatal(err)
	}

	for _, fi := range fis {
		name := fi.Name()
		if filepath.Ext(name) != ".txt" {
			continue
		}

		t.Run(name, func(t *testing.T) {
			tempDir := t.TempDir()
			if err := unpackSpec(filepath.Join(testdataDir, name), tempDir); err != nil {
				t.Fatalf("Failed to unpack spec: %v", err)
			}

			resolver, err := portage.NewResolver(tempDir)
			if err != nil {
				t.Fatalf("Failed to initialize: %v", err)
			}

			globalVarsList, err := resolver.Config().EvalGlobalVars(make(makevars.Vars))
			if err != nil {
				t.Fatalf("Failed to evaluate global variables: %v", err)
			}
			globalVars := makevars.Finalize(globalVarsList)

			wantGlobalVars, err := readExpectVars(filepath.Join(tempDir, "expect.vars"))
			if err != nil {
				t.Fatalf("Failed to parse expect.vars: %v", err)
			}

			gotGlobalVars := make(makevars.Vars)
			for name := range wantGlobalVars {
				if gotValue, ok := globalVars[name]; ok {
					gotGlobalVars[name] = gotValue
				}
			}

			if diff := cmp.Diff(gotGlobalVars, wantGlobalVars, cmpopts.EquateEmpty()); diff != "" {
				t.Errorf("expect.vars mismatch (-got +want):\n%s", diff)
			}

			wantUse, err := readExpectUse(filepath.Join(tempDir, "expect.use"))
			if err != nil {
				t.Fatalf("Failed to parse expect.use: %v", err)
			}

			gotUse := make(map[string]string)
			for packageName := range wantUse {
				atom, err := dependency.ParseAtom(packageName)
				if err != nil {
					t.Fatalf("ParseAtom(%q) failed: %v", packageName, err)
				}
				pkg, err := resolver.BestPackage(atom)
				if err != nil {
					t.Fatalf("BestPackage(%q) failed: %v", packageName, err)
				}

				var validUses []string
				for name := range pkg.Uses() {
					validUses = append(validUses, name)
				}
				sort.Strings(validUses)

				var uses []string
				for _, name := range strings.Fields(wantUse[packageName]) {
					if strings.HasPrefix(name, "-") {
						name = strings.TrimPrefix(name, "-")
					}
					if pkg.Uses()[name] {
						uses = append(uses, name)
					} else {
						uses = append(uses, "-"+name)
					}
				}
				gotUse[packageName] = strings.Join(uses, " ")
			}

			if diff := cmp.Diff(gotUse, wantUse, cmpopts.EquateEmpty()); diff != "" {
				t.Errorf("expect.use mismatch (-got +want):\n%s", diff)
			}
		})
	}
}
