// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makechroot

import (
	"errors"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"cros.local/bazel/ebuild/private/common/fileutil"
)

// filepath.Glob doesn't support "**"
func findFiles(root string, pattern string) ([]string, error) {
	var matches []string

	if err := filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		// Don't fail if root doesn't exist
		if path == root && errors.Is(err, fs.ErrNotExist) {
			return fs.SkipDir
		} else if err != nil {
			return err
		}

		if match, err := filepath.Match(pattern, filepath.Base(path)); err != nil {
			return nil
		} else if match {
			matches = append(matches, path)
		}

		return nil
	}); err != nil {
		return nil, err
	}

	return matches, nil
}

func sortContents(pkgDir string) error {
	matches, err := findFiles(pkgDir, "CONTENTS")
	if err != nil {
		return err
	}

	for _, match := range matches {
		contents, err := os.ReadFile(match)
		if err != nil {
			return err
		}

		lines := strings.Split(string(contents), "\n")

		if len(lines) > 0 && lines[len(lines)-1] == "" {
			// Trailing newline
			lines = lines[:len(lines)-1]
		}

		sort.Strings(lines)
		joined := strings.Join(lines, "\n")

		if err = os.WriteFile(match, []byte(joined), 0); err != nil {
			return err
		}
	}
	return nil
}

func zeroCounter(pkgDir string) error {
	matches, err := findFiles(pkgDir, "COUNTER")
	if err != nil {
		return err
	}

	for _, match := range matches {
		if err = os.WriteFile(match, []byte("0"), 0); err != nil {
			return err
		}
	}

	return nil
}

func truncateEnvironment(pkgDir string) error {
	matches, err := findFiles(pkgDir, "environment.bz2")
	if err != nil {
		return err
	}
	for _, match := range matches {
		if err = os.WriteFile(match, nil, 0); err != nil {
			return err
		}
	}
	return nil
}

func CleanLayer(board string, outputDir string) error {
	var rmDirs = []string{
		"mnt/host",
		"run",
		"stage",
		"tmp",
		"var/cache",
		"var/lib/portage/pkgs",
		"var/log",
		"var/tmp",
	}
	if board != "" {
		rmDirs = append(rmDirs,
			filepath.Join("build", board, "tmp"),
			filepath.Join("build", board, "var/cache"),
			filepath.Join("build", board, "packages"))
	}

	for _, exclude := range rmDirs {
		path := filepath.Join(outputDir, exclude)
		if err := fileutil.RemoveAllWithChmod(path); err != nil {
			return err
		}
	}

	// So this is kind of annoying, since we monkey patch the portage .py files the
	// python interpreter will regenerate the bytecode cache. This bytecode file
	// has the timestamp of the source file embedded. Once we stop monkey patching
	// portage and get the changes bundled in the SDK we can delete the following.
	matches, err := findFiles(
		filepath.Join(outputDir, "usr/lib64/python3.6/site-packages"),
		"*.pyc")
	if err != nil {
		return err
	}
	for _, match := range matches {
		fileutil.RemoveWithChmod(match)
	}

	// The portage database contains some non-hermetic install artifacts:
	// COUNTER: Since we are installing packages in parallel the COUNTER variable
	//          can change depending on when it was installed.
	// environment.bz2: The environment contains EPOCHTIME and SRANDOM from when the
	//                  package was installed. We could modify portage to omit these,
	//                  but I didn't think the binpkg-hermetic FEATURE should apply
	//                  to locally installed artifacts. So we just delete the file
	//                  for now.
	// CONTENTS: This file is sorted in the binpkg, but when portage installs the
	//           binpkg it recreates it in a non-hermetic way, so we manually sort
	//           it.
	// Deleting the files causes a "special" delete marker to be created by overlayfs
	// this isn't supported by bazel. So instead we just truncate the files.
	for _, root := range []string{outputDir, filepath.Join(outputDir, "build", board)} {
		pkgDir := filepath.Join(root, "var/db/pkg")

		if err := truncateEnvironment(pkgDir); err != nil {
			return err
		}

		if err := zeroCounter(pkgDir); err != nil {
			return err
		}

		if err := sortContents(pkgDir); err != nil {
			return err
		}
	}

	return nil
}
