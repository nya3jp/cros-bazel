// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package dependency_test

import (
	"testing"

	"github.com/google/go-cmp/cmp"

	"cros.local/bazel/ebuild/private/common/portage/dependency"
)

func TestParseUri(t *testing.T) {
	got, err := dependency.Parse("a? ( http://example.com )")
	if err != nil {
		t.Fatal(err)
	}

	want := dependency.NewDeps(
		dependency.NewAllOf([]dependency.Expr{
			dependency.NewUseConditional("a", true,
				dependency.NewAllOf([]dependency.Expr{
					dependency.NewUri("http://example.com", nil),
				})),
		}))
	// We use a string compare since the fields are unexported
	if diff := cmp.Diff(got.String(), want.String()); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}

func TestParseUriWithhName(t *testing.T) {
	got, err := dependency.Parse("a? ( http://example.com -> foo.bar )")
	if err != nil {
		t.Fatal(err)
	}

	fileName := "foo.bar"
	want := dependency.NewDeps(
		dependency.NewAllOf([]dependency.Expr{
			dependency.NewUseConditional("a", true,
				dependency.NewAllOf([]dependency.Expr{
					dependency.NewUri("http://example.com", &fileName),
				})),
		}))
	// We use a string compare since the fields are unexported
	if diff := cmp.Diff(got.String(), want.String()); diff != "" {
		t.Fatalf("Parse result mismatch (-got want):\n%s", diff)
	}
}
