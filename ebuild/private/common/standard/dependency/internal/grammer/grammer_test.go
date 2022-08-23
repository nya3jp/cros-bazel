package grammer_test

import (
	"testing"

	"github.com/google/go-cmp/cmp"

	"cros.local/rules_ebuild/ebuild/private/common/standard/dependency/internal/grammer"
)

func mustParse(t *testing.T, s string) *grammer.AllOf {
	p, err := grammer.Parse(s)
	if err != nil {
		t.Fatalf("grammer.Parse(%q): %v", s, err)
	}
	return p
}

func TestParse(t *testing.T) {
	got := mustParse(t, "a ( b )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{Package: &grammer.Package{Raw: "a"}},
			{
				AllOf: &grammer.AllOf{
					Children: []*grammer.Expr{
						{Package: &grammer.Package{Raw: "b"}},
					},
				},
			},
		},
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}

func TestParse_AllOf(t *testing.T) {
	got := mustParse(t, "( a b c )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{
				AllOf: &grammer.AllOf{
					Children: []*grammer.Expr{
						{Package: &grammer.Package{Raw: "a"}},
						{Package: &grammer.Package{Raw: "b"}},
						{Package: &grammer.Package{Raw: "c"}},
					},
				},
			},
		},
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}

func TestParse_AnyOf(t *testing.T) {
	got := mustParse(t, "|| ( a b c )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{
				AnyOf: &grammer.AnyOf{
					Children: []*grammer.Expr{
						{Package: &grammer.Package{Raw: "a"}},
						{Package: &grammer.Package{Raw: "b"}},
						{Package: &grammer.Package{Raw: "c"}},
					},
				},
			},
		},
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}

func TestParse_ExactlyOneOf(t *testing.T) {
	got := mustParse(t, "^^ ( a b c )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{
				ExactlyOneOf: &grammer.ExactlyOneOf{
					Children: []*grammer.Expr{
						{Package: &grammer.Package{Raw: "a"}},
						{Package: &grammer.Package{Raw: "b"}},
						{Package: &grammer.Package{Raw: "c"}},
					},
				},
			},
		},
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}

func TestParse_AtMostOneOf(t *testing.T) {
	got := mustParse(t, "?? ( a b c )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{
				AtMostOneOf: &grammer.AtMostOneOf{
					Children: []*grammer.Expr{
						{Package: &grammer.Package{Raw: "a"}},
						{Package: &grammer.Package{Raw: "b"}},
						{Package: &grammer.Package{Raw: "c"}},
					},
				},
			},
		},
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}

func TestParse_UseConditional(t *testing.T) {
	got := mustParse(t, "foo? ( a b ) !bar? ( c d )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{
				UseConditional: &grammer.UseConditional{
					Condition: "foo?",
					Child: &grammer.AllOf{
						Children: []*grammer.Expr{
							{Package: &grammer.Package{Raw: "a"}},
							{Package: &grammer.Package{Raw: "b"}},
						},
					},
				},
			},
			{
				UseConditional: &grammer.UseConditional{
					Condition: "!bar?",
					Child: &grammer.AllOf{
						Children: []*grammer.Expr{
							{Package: &grammer.Package{Raw: "c"}},
							{Package: &grammer.Package{Raw: "d"}},
						},
					},
				},
			},
		},
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}
