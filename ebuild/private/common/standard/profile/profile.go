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

func Parse(path string, name string, resolver Resolver) (*Profile, error) {
	if _, err := os.Stat(path); err != nil {
		if errors.Is(err, fs.ErrNotExist) {
			return nil, fmt.Errorf("profile %s: not found", name)
		}
		return nil, fmt.Errorf("profile %s: %w", name, err)
	}

	parentPaths, err := readParents(filepath.Join(path, "parent"))
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

	// Parse make.defaults without evaluating to make sure there are no syntax errors.
	if err := makevars.ParseMakeDefaults(filepath.Join(path, makeDefaults), makevars.Vars{}); err != nil && !errors.Is(err, fs.ErrNotExist) {
		return nil, fmt.Errorf("profile %s: %w", name, err)
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

func (p *Profile) Vars() makevars.Vars {
	vars := makevars.Vars{}
	p.parseVars(vars)
	return vars
}

func (p *Profile) parseVars(vars makevars.Vars) {
	for _, parent := range p.parents {
		parent.parseVars(vars)
	}

	// Assume no error as we already checked syntax in Parse.
	_ = makevars.ParseMakeDefaults(filepath.Join(p.path, makeDefaults), vars)
}

func readParents(path string) ([]string, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	var names []string
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		// PMS doesn't allow comments, but in reality there are comments.
		line := strings.TrimSpace(strings.SplitN(sc.Text(), "#", 2)[0])
		if line == "" {
			continue
		}
		names = append(names, line)
	}
	if err := sc.Err(); err != nil {
		return nil, err
	}
	return names, nil
}
