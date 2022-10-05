// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package config

import (
	"bufio"
	"errors"
	"fmt"
	"os"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type PackageUse struct {
	Atom *dependency.Atom
	Uses []string
}

func ParseUseList(path string) ([]string, error) {
	lines, err := ParseLines(path)
	if errors.Is(err, os.ErrNotExist) {
		return nil, nil
	}
	return lines, nil
}

func ParsePackageUseList(path string) ([]*PackageUse, error) {
	lines, err := ParseLines(path)
	if errors.Is(err, os.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	var packageUseList []*PackageUse
	for _, line := range lines {
		fields := strings.Fields(line)
		if len(fields) == 0 {
			continue
		}

		atom, err := dependency.ParseAtom(fields[0])
		if err != nil {
			return nil, err
		}
		packageUseList = append(packageUseList, &PackageUse{
			Atom: atom,
			Uses: fields[1:],
		})
	}
	return packageUseList, nil
}

func ParsePackageProvided(path string) ([]*Package, error) {
	lines, err := ParseLines(path)
	if errors.Is(err, os.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	var provided []*Package

	for _, line := range lines {
		prefix, ver, err := version.ExtractSuffix(line)
		if err != nil {
			return nil, fmt.Errorf("invalid provided package spec: %s: %w", line, err)
		}

		const hyphen = "-"
		if !strings.HasSuffix(prefix, hyphen) {
			return nil, fmt.Errorf("invalid provided package spec: %s", line)
		}
		name := strings.TrimSuffix(prefix, hyphen)
		provided = append(provided, &Package{
			Name:    name,
			Version: ver,
		})
	}
	return provided, nil
}

func ParseLines(path string) ([]string, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	var lines []string
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		lines = append(lines, line)
	}
	if err := sc.Err(); err != nil {
		return nil, err
	}
	return lines, nil
}
