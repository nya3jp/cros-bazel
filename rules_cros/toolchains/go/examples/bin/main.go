// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import "cros.local/rules_cros/toolchains/go/examples/lib"

func main() {
	println("hello world")
	println("1 + 1 = ", lib.Add(1, 1))
}
