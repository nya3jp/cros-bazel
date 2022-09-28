// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makeconf

import (
	"errors"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
)

type UserConfigSource struct {
	rootDir string
}

var _ config.Source = &UserConfigSource{}

func NewUserConfigSource(rootDir string) *UserConfigSource {
	return &UserConfigSource{rootDir: rootDir}
}

func (s *UserConfigSource) EvalGlobalVars(env makevars.Vars) ([]makevars.Vars, error) {
	var varsList []makevars.Vars
	for _, relPath := range []string{"etc/make.conf", "etc/portage/make.conf"} {
		path := filepath.Join(s.rootDir, relPath)
		if _, err := os.Stat(path); errors.Is(err, fs.ErrNotExist) {
			continue
		}
		vars, err := makevars.Eval(path, env, true)
		if err != nil {
			return nil, err
		}
		varsList = append(varsList, vars)
	}
	return varsList, nil
}

func (s *UserConfigSource) EvalPackageVars(pkg *config.Package, env makevars.Vars) ([]makevars.Vars, error) {
	varsList, err := s.EvalGlobalVars(env)
	if err != nil {
		return nil, err
	}

	packageUse, err := config.ParsePackageUseList(filepath.Join(s.rootDir, "etc/portage/package.use"))
	if err != nil {
		return nil, err
	}

	targetPkg := &dependency.TargetPackage{
		Name:    pkg.Name,
		Version: pkg.Version,
		Uses:    nil,
	}
	var uses []string
	for _, pu := range packageUse {
		if pu.Atom.Match(targetPkg) {
			uses = append(uses, pu.Uses...)
		}
	}
	if len(uses) > 0 {
		varsList = append(varsList, makevars.Vars{"USE": strings.Join(uses, " ")})
	}
	return varsList, nil
}

func (s *UserConfigSource) UseMasksAndForces(pkg *config.Package, masks map[string]bool, forces map[string]bool) error {
	// TODO: Parse /etc/portage/profile/*.
	return nil
}

func (s *UserConfigSource) ProvidedPackages() ([]*config.Package, error) {
	// TODO: Parse /etc/portage/profile/package.provided.
	return nil, nil
}
