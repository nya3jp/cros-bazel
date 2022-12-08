// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package tar

import (
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"archive/tar"

	"github.com/klauspost/compress/zstd"
)

func extractTar(r io.Reader, dest string) error {
	tarReader := tar.NewReader(r)

	for true {
		header, err := tarReader.Next()

		if err == io.EOF {
			break
		} else if err != nil {
			return fmt.Errorf("failed decoding tar: %w", err)
		}

		switch header.Typeflag {
		case tar.TypeDir:
			path := filepath.Join(dest, header.Name)
			if err := os.Mkdir(path, fs.FileMode(header.Mode)); err != nil {
				return fmt.Errorf("failed to mkdir %s with mode: %o: %w", path, header.Mode, err)
			}
		case tar.TypeReg:
			path := filepath.Join(dest, header.Name)

			outFile, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY, fs.FileMode(header.Mode).Perm())
			if err != nil {
				return fmt.Errorf("failed to open %s with mode: %o: %w", path, header.Mode, err)
			}
			_, err = io.Copy(outFile, tarReader)
			outFile.Close()
			if err != nil {
				return fmt.Errorf("failed to write %s: %w", path, err)
			}
		case tar.TypeSymlink:
			path := filepath.Join(dest, header.Name)
			if err = os.Symlink(header.Linkname, path); err != nil {
				return fmt.Errorf("failed to symlink %s -> %s: %w", path, header.Linkname, err)
			}
		case tar.TypeLink:
			path := filepath.Join(dest, header.Name)
			// TODO: Add support for hard links. We need to make sure all the files
			// have been created before we create the hard links. Though it might get
			// tricky because a hard link could have an absolute path, and we need to
			// hard link to the path in the chroot. Using symlinks works just fine
			// for now.
			if err = os.Symlink(header.Linkname, path); err != nil {
				return fmt.Errorf("failed to hard link %s -> %s: %w", path, header.Linkname, err)
			}
		default:
			return fmt.Errorf("unknown type: %#x for file %s", header.Typeflag, header.Name)
		}
	}

	return nil
}

func extractTarZstd(src string, dest string) error {
	file, err := os.Open(src)
	if err != nil {
		return err
	}
	defer file.Close()

	decoder, err := zstd.NewReader(file, zstd.WithDecoderConcurrency(0))
	if err != nil {
		return fmt.Errorf("Failed to create zstd decoder for file: %s", src)
	}
	defer decoder.Close()

	if err = extractTar(decoder, dest); err != nil {
		return err
	}

	return nil
}

func findTarExtractor(path string) func(string, string) error {
	if strings.HasSuffix(path, ".tar.zst") {
		return extractTarZstd
	}

	return nil
}

func IsTar(path string) bool {
	if fn := findTarExtractor(path); fn != nil {
		return true
	}

	return false
}

func Extract(src string, dest string) error {
	fn := findTarExtractor(src)
	if fn == nil {
		return fmt.Errorf("%s has an unknown file type", src)
	}

	return fn(src, dest)
}

// files defines the src files inside the tarball and where to extract them to.
// files will be mutated and be left containing the files that didn't get
// extracted.
func ExtractFiles(r io.Reader, files map[string]string) error {
	tarReader := tar.NewReader(r)

	for {
		header, err := tarReader.Next()

		if err == io.EOF {
			break
		} else if err != nil {
			return fmt.Errorf("failed decoding tar: %w", err)
		}

		switch header.Typeflag {
		case tar.TypeReg:
			outPath, fileNameMatches := files[header.Name]
			if !fileNameMatches {
				continue
			}
			delete(files, header.Name)

			outFile, err := os.OpenFile(outPath, os.O_CREATE|os.O_WRONLY, fs.FileMode(header.Mode).Perm())
			if err != nil {
				return fmt.Errorf("failed to open %s with mode: %o: %w", outPath, header.Mode, err)
			}
			_, err = io.Copy(outFile, tarReader)
			outFile.Close() // Close the file regardless of Copy's outcome
			if err != nil {
				return fmt.Errorf("failed to write %s: %w", outPath, err)
			}
		case tar.TypeSymlink:
			outPath, fileNameMatches := files[header.Name]
			if !fileNameMatches {
				continue
			}
			delete(files, header.Name)

			// bazel only supports relative symlinks that point to existing files.
			// Let's limit this to symlinks that point to files in the same directory
			// for now.
			if strings.Contains(header.Linkname, "/") {
				return fmt.Errorf("symlinks paths separators are currently supported %s -> %s", header.Name, header.Linkname)
			}

			if err = os.Symlink(header.Linkname, outPath); err != nil {
				return fmt.Errorf("failed to create symlink %s -> %s: %w", outPath, header.Linkname, err)
			}
		case tar.TypeDir:
			// We only extract files for now
			continue
		default:
			return fmt.Errorf("Unknown tar type %#x", tar.TypeDir)
		}
	}

	if len(files) > 0 {
		return fmt.Errorf("Failed to extract: %v", files)
	}

	return nil
}
