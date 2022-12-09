// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"cros.local/bazel/ebuild/private/common/tar"
)

func writeReader(src io.ReadCloser, dest string) error {
	tmpName := dest + ".tmp"
	destFile, err := os.Create(tmpName)
	if err != nil {
		return err
	}

	if _, err := io.Copy(destFile, src); err != nil {
		destFile.Close()
		os.Remove(tmpName)
		return err
	}

	if err := destFile.Close(); err != nil {
		os.Remove(tmpName)
		return err
	}

	if err := os.Rename(tmpName, dest); err != nil {
		os.Remove(tmpName)
		return err
	}

	return nil
}

func extractXpak(binPkg *binarypackage.File, dest string) error {
	if err := os.RemoveAll(dest); err != nil {
		return err
	}

	if err := os.MkdirAll(dest, 0755); err != nil {
		return err
	}

	xpakHeader, err := binPkg.Xpak()
	if err != nil {
		return err
	}

	for k, v := range xpakHeader {
		target := filepath.Join(dest, k)
		err := os.WriteFile(target, v, 0644)
		if err != nil {
			return err
		}
	}

	return nil
}

func splitBinaryPackage(ctx context.Context, fileName string, extract bool, dest string) error {
	binPkg, err := binarypackage.BinaryPackage(fileName)
	if err != nil {
		return fmt.Errorf("failed opening binpkg: %w", err)
	}
	defer binPkg.Close()

	baseFileName := strings.TrimSuffix(filepath.Base(fileName), ".tbz2")

	dest = filepath.Join(dest, baseFileName)

	xpakDest := filepath.Join(dest, fmt.Sprintf("%s.xpak", baseFileName))
	if err = extractXpak(binPkg, xpakDest); err != nil {
		return err
	}

	tarDest := filepath.Join(dest, fmt.Sprintf("%s.tar.zst", baseFileName))
	tarReader, err := binPkg.TarballReader()
	if err != nil {
		return err
	}
	defer tarReader.Close()

	if err := writeReader(tarReader, tarDest); err != nil {
		return err
	}

	if extract {
		contentDest := filepath.Join(dest, baseFileName)
		if err := os.RemoveAll(contentDest); err != nil {
			return err
		}

		// TODO: Add a ctx to tar.Extract
		if err := tar.Extract(tarDest, contentDest); err != nil {
			return err
		}
	}

	return nil
}

func splitCmd(ctx context.Context, extract bool, destination string, fileNames []string) error {
	if destination != "" {
		// Pre-create the directory to avoid any race conditions below
		if err := os.MkdirAll(destination, 0755); err != nil {
			return err
		}
	}

	errc := make(chan error, len(fileNames))
	for _, srcFileName := range fileNames {
		go func(fileName string) {
			errc <- func() error {
				if !strings.HasSuffix(fileName, ".tbz2") {
					return fmt.Errorf("%s must have a .tbz2 file extension", fileName)
				}

				dest := destination
				if dest == "" {
					dest = filepath.Dir(fileName)
				}

				if err := splitBinaryPackage(ctx, fileName, extract, dest); err != nil {
					return err
				}
				return nil
			}()
		}(srcFileName)
	}

	var errors []error
	for i := 0; i < len(fileNames); i++ {
		err := <-errc
		if err != nil {
			errors = append(errors, err)
		}
	}

	if len(errors) > 0 {
		return fmt.Errorf("Split error: %s", errors)
	}

	return nil
}
