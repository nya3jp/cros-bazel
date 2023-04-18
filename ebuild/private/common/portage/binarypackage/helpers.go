// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package binarypackage

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/tar"
	"github.com/klauspost/compress/zstd"
)

type XpakSpec struct {
	XpakHeader string
	TargetPath string
	Optional   bool
}

// Spec format: <XPAK key>=[?]<outside path>
// If =? is used, an empty file is written if the key doesn't exist
func ParseXpakSpecs(specs []string) ([]XpakSpec, error) {
	var xpakSpecs []XpakSpec
	for _, spec := range specs {
		optional := false
		v := strings.Split(spec, "=?")
		if len(v) == 2 {
			optional = true
		} else {
			v = strings.Split(spec, "=")
			if len(v) != 2 {
				return nil, fmt.Errorf("invalid xpak spec: %s", spec)
			}
		}
		xpakSpecs = append(xpakSpecs, XpakSpec{
			XpakHeader: v[0],
			TargetPath: v[1],
			Optional:   optional,
		})
	}
	return xpakSpecs, nil
}

func ExtractXpakFiles(binPkg *File, xpakSpecs []XpakSpec) error {
	if len(xpakSpecs) == 0 {
		return nil
	}

	xpakHeader, err := binPkg.Xpak()
	if err != nil {
		return err
	}

	for _, xpakSpec := range xpakSpecs {
		xpakValue, ok := xpakHeader[xpakSpec.XpakHeader]
		if !ok {
			if xpakSpec.Optional {
				xpakValue = nil
			} else {
				return fmt.Errorf("XPAK key %s not found in header", xpakSpec.XpakHeader)
			}
		}

		err := os.WriteFile(xpakSpec.TargetPath, xpakValue, 0666)
		if err != nil {
			return err
		}
	}
	return nil
}

type OutputFileSpec struct {
	InsidePath string
	TargetPath string
}

// Spec format: <inside path>=<outside path>
func ParseOutputFileSpecs(specs []string) ([]OutputFileSpec, error) {
	var outputFileSpecs []OutputFileSpec
	for _, spec := range specs {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid overlay spec: %s", spec)
		}
		if !filepath.IsAbs(v[0]) {
			return nil, fmt.Errorf("invalid overlay spec: %s, %s must be absolute", spec, v[0])
		}
		outputFileSpecs = append(outputFileSpecs, OutputFileSpec{
			InsidePath: v[0],
			TargetPath: v[1],
		})
	}
	return outputFileSpecs, nil
}

func ExtractOutFiles(binPkg *File, outputFileSpecs []OutputFileSpec) error {
	if len(outputFileSpecs) == 0 {
		return nil
	}

	// Convert outputFileSpecs into a map for easy lookup
	fileMap := make(map[string]string, len(outputFileSpecs))
	for _, outFileSpec := range outputFileSpecs {
		// tarball filenames start with ./
		fileMap["."+outFileSpec.InsidePath] = outFileSpec.TargetPath
	}

	tarFile, err := binPkg.TarballReader()
	if err != nil {
		return err
	}
	defer tarFile.Close()

	// TODO: Support different compression algos?
	decoder, err := zstd.NewReader(tarFile, zstd.WithDecoderConcurrency(0))
	if err != nil {
		return fmt.Errorf("Failed to create zstd decoder: %w", err)
	}
	defer decoder.Close()

	return tar.ExtractFiles(decoder, fileMap)
}
