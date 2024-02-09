// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main_test

import (
	"debug/elf"
	"testing"

	"github.com/bazelbuild/rules_go/go/runfiles"
)

func TestDynamicLink(t *testing.T) {
	bin, err := runfiles.Rlocation("cros/bazel/portage/bin/fakefs/testhelper/testhelper")
	if err != nil {
		t.Fatal(err)
	}

	file, err := elf.Open(bin)
	if err != nil {
		t.Fatal(err)
	}

	isDyn := false
	for _, s := range file.Sections {
		if s.Type == elf.SHT_DYNSYM {
			isDyn = true
			break
		}
	}
	if !isDyn {
		t.Fatal("testhelper is not dynamically linked")
	}
}
