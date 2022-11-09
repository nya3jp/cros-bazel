// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package fileutil

import (
	"io/fs"
	"os"
	"path/filepath"
)

// RemoveAllWithChmod calls os.RemoveAll after ensuring we have o+rwx to each
// directory so that we can remove all files.
func RemoveAllWithChmod(path string) error {
	if err := filepath.WalkDir(path, func(path string, info fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() {
			return nil
		}

		fileInfo, err := info.Info()
		if err != nil {
			return err
		}

		if fileInfo.Mode().Perm()&0700 == 0700 {
			return nil
		}

		if err := os.Chmod(path, 0700); err != nil {
			return err
		}

		return nil
	}); err != nil {
		return err
	}

	return os.RemoveAll(path)
}
