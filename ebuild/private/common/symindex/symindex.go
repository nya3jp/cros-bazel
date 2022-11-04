// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Package symindex provides utilities to work with symbolic link index files.
package symindex

import (
	"bufio"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
)

// Ext is a file name extension added to symindex files.
const Ext = ".symindex"

// Generate creates a symindex file at indexPath scanning rootDir.
// It also removes all symlinks under rootDir.
func Generate(rootDir string, indexPath string) error {
	if !strings.HasSuffix(indexPath, Ext) {
		return fmt.Errorf("symindex file name must end with %s", Ext)
	}

	index, err := os.Create(indexPath)
	if err != nil {
		return err
	}
	defer index.Close()

	// Note: WalkDir visits files in lexical order, so the output is deterministic.
	return filepath.WalkDir(rootDir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.Type()&fs.ModeSymlink == 0 {
			return nil
		}

		source, err := filepath.Rel(rootDir, path)
		if err != nil {
			return err
		}
		target, err := os.Readlink(path)
		if err != nil {
			return err
		}

		if err := os.Remove(path); err != nil {
			return err
		}

		fmt.Fprintf(index, "%s\t%s\n", source, target)
		return nil
	})
}

// Expand reads a symindex file at indexPath and creates symlinks under rootDir
// accordingly.
func Expand(indexPath string, rootDir string) error {
	if !strings.HasSuffix(indexPath, Ext) {
		return fmt.Errorf("symindex file name must end with %s", Ext)
	}

	index, err := os.Open(indexPath)
	if err != nil {
		return err
	}
	defer index.Close()

	sc := bufio.NewScanner(index)
	for sc.Scan() {
		line := sc.Text()
		v := strings.Split(line, "\t")
		if len(v) != 2 {
			return fmt.Errorf("malformed symlink index line: %s", line)
		}
		source := v[0]
		target := v[1]
		if err := os.MkdirAll(filepath.Dir(filepath.Join(rootDir, source)), 0o755); err != nil {
			return err
		}
		if err := os.Symlink(target, filepath.Join(rootDir, source)); err != nil {
			return err
		}
	}
	if err := sc.Err(); err != nil {
		return err
	}
	return nil
}
