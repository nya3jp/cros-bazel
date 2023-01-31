// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package fileutil

import (
	"io/fs"
	"os"
	"path/filepath"

	"golang.org/x/sys/unix"
)

// MoveDirContents moves the contents of |from| to |to| after ensuring we have u+w to each directory
// entry and restores the original file permissions.
func MoveDirContents(from string, to string) error {
	es, err := os.ReadDir(from)
	if err != nil {
		return err
	}

	for _, e := range es {
		src := filepath.Join(from, e.Name())
		dest := filepath.Join(to, e.Name())

		var fileMode fs.FileMode
		if e.IsDir() {
			// For directories, we need u+w (S_IWUSR) permission to rename.
			fi, err := e.Info()
			if err != nil {
				return err
			}
			fileMode = fi.Mode()
			if fileMode.Perm()&unix.S_IWUSR == 0 {
				if err := os.Chmod(src, fileMode.Perm()|unix.S_IWUSR); err != nil {
					return err
				}
			}
		}

		if err := os.Rename(src, dest); err != nil {
			return err
		}

		if e.IsDir() {
			// Restore the original file permissions.
			if err := os.Chmod(dest, fileMode.Perm()); err != nil {
				return err
			}
		}
	}

	return nil
}
