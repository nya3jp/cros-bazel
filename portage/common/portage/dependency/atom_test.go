// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package dependency_test

import (
	"testing"

	"cros.local/bazel/portage/common/portage/dependency"
	"cros.local/bazel/portage/common/portage/version"
)

func verifyParseAtom(t *testing.T, atomStr string) {
	a, err := dependency.ParseAtom(atomStr)
	if err != nil {
		t.Errorf("atom.Parse(%q) failed: %v", atomStr, err)
		return
	}
	s := a.String()
	if s != atomStr {
		t.Errorf("atom.Parse(%q).String() = %q; want %q", atomStr, s, atomStr)
	}
}

func TestParseAtom(t *testing.T) {
	verifyParseAtom(t, "<=dev-libs/9libs-1.0")
}

func TestAtomMatch(t *testing.T) {
	for _, tc := range []struct {
		atom   string
		target string
		want   bool
	}{
		{
			atom:   "=dev-rust/atomic-polyfill-0.1*",
			target: "dev-rust/atomic-polyfill-0.1.0",
			want:   true,
		},
		{
			atom:   "=dev-rust/rustc-std-workspace-core-1*:=",
			target: "dev-rust/rustc-std-workspace-core-1.0.0",
			want:   true,
		},
	} {
		a, err := dependency.ParseAtom(tc.atom)
		if err != nil {
			t.Fatalf("atom.Parse(%q) failed: %v", tc.atom, err)
		}
		prefix, ver, err := version.ExtractSuffix(tc.target)
		if err != nil {
			t.Fatalf("BUG: version.ExtractSuffix(%q) failed: %v", tc.target, err)
		}
		got := a.Match(&dependency.TargetPackage{
			Name:     prefix,
			Version:  ver,
			MainSlot: "",
			Uses:     nil,
		})
		if got != tc.want {
			t.Fatalf("atom.Parse(%q).Match(%q) = %t; want %t", tc.atom, tc.target, got, tc.want)
		}
	}
}

func TestAtomMatch_Slot(t *testing.T) {
	const packageName = "pkg/x"

	for _, tc := range []struct {
		slot string
		dep  string
		want bool
	}{
		{"0", "", true},
		{"0", "*", true},
		{"0", "=", true},
		{"0", "0", true},
		{"0", "1", false},
		{"0", "0=", true},
		{"0", "1=", false},
	} {
		atom := dependency.NewAtom(packageName, dependency.OpNone, nil, false, tc.dep, nil)
		got := atom.Match(&dependency.TargetPackage{
			Name:     packageName,
			Version:  &version.Version{Main: []string{"1", "0", "0"}},
			MainSlot: tc.slot,
			Uses:     nil,
		})
		if got != tc.want {
			t.Fatalf("Match(slot=%q, dep=%q) = %t; want %t", tc.slot, tc.dep, got, tc.want)
		}
	}
}
