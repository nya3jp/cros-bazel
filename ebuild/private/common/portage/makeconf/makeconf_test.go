// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makeconf_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/google/go-cmp/cmp"

	"cros.local/bazel/ebuild/private/common/portage/makeconf"
)

func callParse(t *testing.T, entryName string, files map[string]string) map[string]string {
	rootDir := t.TempDir()
	for name, content := range files {
		if err := os.WriteFile(filepath.Join(rootDir, name), []byte(content), 0600); err != nil {
			t.Fatal(err)
		}
	}

	vars, err := makeconf.Parse(filepath.Join(rootDir, entryName))
	if err != nil {
		t.Fatalf("makeconf.Parse failed: %v", err)
	}
	return vars
}

func TestParse(t *testing.T) {
	got := callParse(t, "make.conf", map[string]string{
		"make.conf": `A=a; B="b"`,
	})
	want := map[string]string{
		"A": "a",
		"B": "b",
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse returned unexpected vars (-got +want):\n%s", diff)
	}
}

func TestParse_Expands(t *testing.T) {
	got := callParse(t, "make.conf", map[string]string{
		"make.conf": `A=a; B="${A} $A"`,
	})
	want := map[string]string{
		"A": "a",
		"B": "a a",
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse returned unexpected vars (-got +want):\n%s", diff)
	}
}

func TestParse_Source(t *testing.T) {
	got := callParse(t, "make.conf", map[string]string{
		"make.conf": `A=a; source sub.conf; B="b${A}b"`,
		"sub.conf":  `A="${A}${A}"`,
	})
	want := map[string]string{
		"A": "aa",
		"B": "baab",
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse returned unexpected vars (-got +want):\n%s", diff)
	}
}
