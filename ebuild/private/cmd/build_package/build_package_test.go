// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"testing"

	"cros.local/bazel/ebuild/private/common/standard/version"
	"github.com/google/go-cmp/cmp"
)

func TestParseEbuildMetadata(t *testing.T) {
	parseVersion := func(s string) *version.Version {
		v, err := version.Parse(s)
		if err != nil {
			t.Fatal(err)
		}
		return v
	}
	for _, tc := range []struct {
		path    string
		want    *EbuildMetadata
		wantErr bool
	}{
		{
			path: "third_party/chromiumos-overlay/dev-lang/python/python-3.7.9-r1.ebuild",
			want: &EbuildMetadata{
				Overlay:     "third_party/chromiumos-overlay",
				Category:    "dev-lang",
				PackageName: "python",
				Version:     parseVersion("3.7.9-r1"),
			},
		},
		// TODO: this currently fails with absolute paths.
		//{
		//	path: "/absolute/path/to/third_party/chromiumos-overlay/dev-lang/python/python-3.7.9-r1.ebuild",
		//	want: &EbuildMetadata{
		//		Overlay:     "third_party/chromiumos-overlay",
		//		Category:    "dev-lang",
		//		PackageName: "python",
		//		Version:     parseVersion("3.7.9-r1"),
		//	},
		//},
		{
			path:    "third_party/chromiumos-overlay/dev-lang/python",
			wantErr: true,
		},
	} {
		got, err := ParseEbuildMetadata(tc.path)
		if err != nil && !tc.wantErr {
			t.Errorf("ParseEbuildMetadata(%s) returned unexpected error: %v", tc.path, err)
		}
		if err == nil && tc.wantErr {
			t.Errorf("ParseEbuildMetadata(%s) unexpectedly succeeded", tc.path)
		}
		if diff := cmp.Diff(tc.want, got); diff != "" {
			t.Errorf("ParseEbuildMetadata(%s) returned unexpected diff (-want +got):\n%s", tc.path, diff)
		}
	}
}
