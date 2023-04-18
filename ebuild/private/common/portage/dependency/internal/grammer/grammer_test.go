// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package grammer_test

import (
	"testing"

	"github.com/google/go-cmp/cmp"

	"cros.local/bazel/ebuild/private/common/portage/dependency/internal/grammer"
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
			{Value: &grammer.Value{Package: &grammer.Package{Raw: "a"}}},
			{
				AllOf: &grammer.AllOf{
					Children: []*grammer.Expr{
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "b"}}},
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
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "a"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "b"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "c"}}},
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
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "a"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "b"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "c"}}},
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
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "a"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "b"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "c"}}},
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
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "a"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "b"}}},
						{Value: &grammer.Value{Package: &grammer.Package{Raw: "c"}}},
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
							{Value: &grammer.Value{Package: &grammer.Package{Raw: "a"}}},
							{Value: &grammer.Value{Package: &grammer.Package{Raw: "b"}}},
						},
					},
				},
			},
			{
				UseConditional: &grammer.UseConditional{
					Condition: "!bar?",
					Child: &grammer.AllOf{
						Children: []*grammer.Expr{
							{Value: &grammer.Value{Package: &grammer.Package{Raw: "c"}}},
							{Value: &grammer.Value{Package: &grammer.Package{Raw: "d"}}},
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

func TestParse_Uri(t *testing.T) {
	got := mustParse(t, "http://example.com http://example.org")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{Value: &grammer.Value{Uri: &grammer.Uri{Uri: "http://example.com"}}},
			{Value: &grammer.Value{Uri: &grammer.Uri{Uri: "http://example.org"}}},
		},
	}
	if diff := cmp.Diff(got, want); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}

func TestParse_UseConditional_Uri(t *testing.T) {
	got := mustParse(t, "a? ( http://example.com )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{
				UseConditional: &grammer.UseConditional{
					Condition: "a?",
					Child: &grammer.AllOf{
						Children: []*grammer.Expr{
							{Value: &grammer.Value{Uri: &grammer.Uri{Uri: "http://example.com"}}},
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

func TestParse_UseConditional_Uri_WithRename(t *testing.T) {
	fileName := "foo.tgz"

	got := mustParse(t, "a? ( http://example.com -> foo.tgz )")
	want := &grammer.AllOf{
		Children: []*grammer.Expr{
			{
				UseConditional: &grammer.UseConditional{
					Condition: "a?",
					Child: &grammer.AllOf{
						Children: []*grammer.Expr{
							{Value: &grammer.Value{Uri: &grammer.Uri{Uri: "http://example.com", FileName: &fileName}}},
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
