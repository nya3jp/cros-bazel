// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package fileutil

import (
	"os"
	"os/exec"
)

func Copy(src, dst string) error {
	cmd := exec.Command("/usr/bin/cp", "--", src, dst)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func CopyDir(src, dst string) error {
	cmd := exec.Command("/usr/bin/cp", "-r", "--", src, dst)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}
