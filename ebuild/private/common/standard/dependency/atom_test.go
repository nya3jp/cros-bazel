package dependency_test

import (
	"strings"
	"testing"

	"cros.local/ebuild/private/common/standard/dependency"
	"cros.local/ebuild/private/common/standard/version"
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
			Name:    strings.TrimSuffix(prefix, "-"),
			Version: ver,
			Uses:    nil,
		})
		if got != tc.want {
			t.Fatalf("atom.Parse(%q).Match(%q) = %t; want %t", tc.atom, tc.target, got, tc.want)
		}
	}
}
