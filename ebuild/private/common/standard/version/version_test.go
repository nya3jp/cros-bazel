package version_test

import (
	"testing"

	"cros.local/ebuild/private/common/standard/version"
)

func mustParse(t *testing.T, s string) *version.Version {
	ver, err := version.Parse(s)
	if err != nil {
		t.Fatalf("version.Parse(%q): %v", s, err)
	}
	return ver
}

func TestVersionCompare(t *testing.T) {
	for _, tc := range []struct {
		a    string
		b    string
		want int
	}{
		// Main.
		{"0", "0", 0},
		{"0", "1", -1},
		{"1.0", "1.0", 0},
		{"1.0", "1.1", -1},
		{"1.99", "1.100", -1},
		{"1.099", "1.0100", 1},
		{"1.0", "1.0.0", -1},
		{"1.1", "1.0.0", 1},
		{"1.0", "1.000", 0},
		{"1.0280", "1.02800", 0},
		// Letters.
		{"1.0a", "1.0a", 0},
		{"1.0a", "1.0z", -1},
		// Suffixes.
		{"1.0_alpha", "1.0_alpha", 0},
		{"1.0_alpha", "1.0_alpha0", 0},
		{"1.0_alpha1", "1.0_alpha1", 0},
		{"1.0_alpha9", "1.0_alpha10", -1},
		{"1.0_alpha", "1.0_beta", -1},
		{"1.0_beta", "1.0_pre", -1},
		{"1.0_pre", "1.0_rc", -1},
		{"1.0_rc", "1.0_p", -1},
		{"1.0_alpha1_beta2_pre3_rc4_p5", "1.0_alpha1_beta2_pre3_rc4_p5", 0},
		{"1.0_p1_p2_p3_p", "1.0_p1_p2_p3_p", 0},
		{"1.0", "1.0_alpha1", 1},
		{"1.0", "1.0_beta1", 1},
		{"1.0", "1.0_pre1", 1},
		{"1.0", "1.0_rc1", 1},
		{"1.0", "1.0_p1", -1},
		// Revisions.
		{"1.0", "1.0-r0", 0},
		{"1.0-r0", "1.0-r0", 0},
		{"1.0-r9", "1.0-r10", -1},
	} {
		va := mustParse(t, tc.a)
		vb := mustParse(t, tc.b)
		if got := va.Compare(vb); got != tc.want {
			t.Errorf("Compare(%q, %q) = %d; want %d", va, vb, got, tc.want)
		}
		if got := vb.Compare(va); got != -tc.want {
			t.Errorf("Compare(%q, %q) = %d; want %d", vb, va, got, -tc.want)
		}
	}
}

func TestVersionCompare_EquivalentVersions(t *testing.T) {
	// PMS-8 3.4 Uniqueness of Versions
	vers := []*version.Version{
		mustParse(t, "1.0.2"),
		mustParse(t, "1.0.2-r0"),
		mustParse(t, "1.000.2"),
	}

	for _, a := range vers {
		for _, b := range vers {
			if cmp := a.Compare(b); cmp != 0 {
				t.Errorf("Compare(%q, %q) = %d; want 0", a, b, cmp)
			}
		}
	}
}
