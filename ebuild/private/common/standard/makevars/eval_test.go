// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makevars_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/google/go-cmp/cmp"

	"cros.local/bazel/ebuild/private/common/standard/makevars"
)

func writeFiles(t *testing.T, files map[string]string) string {
	dir := t.TempDir()
	for name, content := range files {
		if err := os.WriteFile(filepath.Join(dir, name), []byte(content), 0600); err != nil {
			t.Fatal(err)
		}
	}
	return dir
}

func TestEval(t *testing.T) {
	dir := writeFiles(t, map[string]string{
		"make.conf": `A=a; B="b"`,
	})
	env := make(makevars.Vars)
	vars, err := makevars.Eval(filepath.Join(dir, "make.conf"), env, false)
	if err != nil {
		t.Fatalf("Eval failed: %v", err)
	}

	want := makevars.Vars{
		"A": "a",
		"B": "b",
	}
	if diff := cmp.Diff(env, want); diff != "" {
		t.Fatalf("Eval left unexpected vars in env (-got +want):\n%s", diff)
	}
	if diff := cmp.Diff(vars, want); diff != "" {
		t.Fatalf("Eval returned unexpected vars in env (-got +want):\n%s", diff)
	}
}

func TestEval_Expands(t *testing.T) {
	dir := writeFiles(t, map[string]string{
		"make.conf": `A=a; B="${A} $A"`,
	})
	env := make(makevars.Vars)
	vars, err := makevars.Eval(filepath.Join(dir, "make.conf"), env, false)
	if err != nil {
		t.Fatalf("Eval failed: %v", err)
	}

	want := makevars.Vars{
		"A": "a",
		"B": "a a",
	}
	if diff := cmp.Diff(env, want); diff != "" {
		t.Fatalf("Eval left unexpected vars in env (-got +want):\n%s", diff)
	}
	if diff := cmp.Diff(vars, want); diff != "" {
		t.Fatalf("Eval returned unexpected vars in env (-got +want):\n%s", diff)
	}
}

func TestEval_Source(t *testing.T) {
	dir := writeFiles(t, map[string]string{
		"make.conf": `A=a; source sub.conf; C="c${B}c"`,
		"sub.conf":  `B="${A}${A}"`,
	})

	if _, err := makevars.Eval(filepath.Join(dir, "make.conf"), make(makevars.Vars), false); err == nil {
		t.Errorf("Eval unexpectedly succeeded while allowSource=false")
	}

	env := make(makevars.Vars)
	vars, err := makevars.Eval(filepath.Join(dir, "make.conf"), env, true)
	if err != nil {
		t.Fatalf("Eval failed: %v", err)
	}

	want := makevars.Vars{
		"A": "a",
		"B": "aa",
		"C": "caac",
	}
	if diff := cmp.Diff(env, want); diff != "" {
		t.Fatalf("Eval left unexpected vars in env (-got +want):\n%s", diff)
	}
	if diff := cmp.Diff(vars, want); diff != "" {
		t.Fatalf("Eval returned unexpected vars in env (-got +want):\n%s", diff)
	}
}

func TestEval_Env(t *testing.T) {
	dir := writeFiles(t, map[string]string{
		"make.conf": `B=$A`,
	})

	env := makevars.Vars{
		"A": "a",
		"B": "b",
	}
	vars, err := makevars.Eval(filepath.Join(dir, "make.conf"), env, true)
	if err != nil {
		t.Fatalf("Eval failed: %v", err)
	}

	if diff := cmp.Diff(env, makevars.Vars{"A": "a", "B": "a"}); diff != "" {
		t.Fatalf("Eval left unexpected vars in env (-got +want):\n%s", diff)
	}
	if diff := cmp.Diff(vars, makevars.Vars{"B": "a"}); diff != "" {
		t.Fatalf("Eval returned unexpected vars in env (-got +want):\n%s", diff)
	}
}
