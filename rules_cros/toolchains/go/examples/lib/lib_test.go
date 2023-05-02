// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package lib_test

import (
	"os"
	"testing"

	"github.com/bazelbuild/rules_go/go/runfiles"
	"github.com/google/go-cmp/cmp"

	"cros.local/rules_cros/toolchains/go/examples/lib"
)

// Check that we can access libraries.
func TestPackageFunction(t *testing.T) {
	lib.Add(1, 1)
}

func TestThirdParty(t *testing.T) {
	cmp.Diff("", "")
}

func TestRunfiles(t *testing.T) {
	path, err := runfiles.Rlocation("cros/rules_cros/toolchains/testdata/example.txt")
	if err != nil {
		t.Fatalf("Expected rlocation to succeed. Got %v", err)
	}
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("Unable to stat file: %v", path, err)
	}
}
