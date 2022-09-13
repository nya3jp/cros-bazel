// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package profile

import (
	"bufio"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

const makeDefaults = "make.defaults"

type Resolver interface {
	ResolveProfile(path, base string) (*Profile, error)
}

type Profile struct {
	name    string
	path    string
	parents []*Profile
}

func Load(path string, name string, resolver Resolver) (*Profile, error) {
	if _, err := os.Stat(path); err != nil {
		if errors.Is(err, fs.ErrNotExist) {
			return nil, fmt.Errorf("profile %s: not found", name)
		}
		return nil, fmt.Errorf("profile %s: %w", name, err)
	}

	parentPaths, err := readLines(filepath.Join(path, "parent"))
	if err != nil && !errors.Is(err, fs.ErrNotExist) {
		return nil, fmt.Errorf("profile %s: reading parents: %w", name, err)
	}

	var parents []*Profile
	for _, parentPath := range parentPaths {
		parent, err := resolver.ResolveProfile(parentPath, path)
		if err != nil {
			return nil, fmt.Errorf("profile %s: %w", name, err)
		}
		parents = append(parents, parent)
	}

	return &Profile{
		name:    name,
		path:    path,
		parents: parents,
	}, nil
}

func (p *Profile) Name() string        { return p.name }
func (p *Profile) Path() string        { return p.path }
func (p *Profile) Parents() []*Profile { return append([]*Profile(nil), p.parents...) }

func (p *Profile) Parse() (*ParsedProfile, error) {
	vars := makevars.Vars{}
	if err := p.parseVars(vars); err != nil {
		return nil, err
	}

	overrides := &Overrides{
		packageUse: make(map[string]string),
	}
	if err := p.parseOverrides(overrides); err != nil {
		return nil, err
	}

	var provided []*ProvidedPackage
	if err := p.parseProvided(&provided); err != nil {
		return nil, err
	}

	return &ParsedProfile{
		profile:   p,
		vars:      vars,
		overrides: overrides,
		provided:  provided,
	}, nil
}

func (p *Profile) parseVars(vars makevars.Vars) error {
	for _, parent := range p.parents {
		if err := parent.parseVars(vars); err != nil {
			return err
		}
	}

	if err := makevars.ParseMakeDefaults(filepath.Join(p.path, makeDefaults), vars); err != nil && !errors.Is(err, fs.ErrNotExist) {
		return err
	}
	return nil
}

func (p *Profile) parseOverrides(overrides *Overrides) error {
	for _, parent := range p.parents {
		parent.parseOverrides(overrides)
	}

	return readPackageUse(filepath.Join(p.path, "package.use"), overrides)
}

func (p *Profile) parseProvided(provided *[]*ProvidedPackage) error {
	for _, parent := range p.parents {
		parent.parseProvided(provided)
	}

	return readPackageProvided(filepath.Join(p.path, "package.provided"), provided)
}

type ParsedProfile struct {
	profile   *Profile
	vars      makevars.Vars
	overrides *Overrides
	provided  []*ProvidedPackage
}

func (p *ParsedProfile) Vars() makevars.Vars {
	return p.vars.Copy()
}

func (p *ParsedProfile) Overrides() *Overrides {
	return p.overrides
}

func (p *ParsedProfile) Provided() []*ProvidedPackage {
	return p.provided
}

type Overrides struct {
	packageUse map[string]string
}

func (o *Overrides) ForPackage(packageName string, ver *version.Version) *PackageOverrides {
	return &PackageOverrides{
		use: o.packageUse[packageName],
	}
}

type PackageOverrides struct {
	use string
}

func (po *PackageOverrides) Use() string {
	return po.use
}

type ProvidedPackage struct {
	name string
	ver  *version.Version
}

func (pp *ProvidedPackage) Name() string {
	return pp.name
}

func (pp *ProvidedPackage) Version() *version.Version {
	return pp.ver
}

func readPackageUse(path string, overrides *Overrides) error {
	lines, err := readLines(path)
	if errors.Is(err, os.ErrNotExist) {
		return nil
	}
	if err != nil {
		return err
	}

	for _, line := range lines {
		fields := strings.Fields(line)
		if len(fields) == 0 {
			continue
		}
		packageName := fields[0]
		uses := fields[1:]
		overrides.packageUse[packageName] += " " + strings.Join(uses, " ")
	}
	return nil
}

func readPackageProvided(path string, provided *[]*ProvidedPackage) error {
	lines, err := readLines(path)
	if errors.Is(err, os.ErrNotExist) {
		return nil
	}
	if err != nil {
		return err
	}

	for _, line := range lines {
		prefix, ver, err := version.ExtractSuffix(line)
		if err != nil {
			return fmt.Errorf("invalid provided package spec: %s: %w", line, err)
		}

		const hyphen = "-"
		if !strings.HasSuffix(prefix, hyphen) {
			return fmt.Errorf("invalid provided package spec: %s", line)
		}
		name := strings.TrimSuffix(prefix, hyphen)
		*provided = append(*provided, &ProvidedPackage{
			name: name,
			ver:  ver,
		})
	}
	return nil
}

func readLines(path string) ([]string, error) {
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
