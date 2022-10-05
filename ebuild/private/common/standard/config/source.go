// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package config

import (
	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Package struct {
	Name    string
	Version *version.Version
}

type Source interface {
	EvalGlobalVars(env makevars.Vars) ([]makevars.Vars, error)
	EvalPackageVars(pkg *Package, env makevars.Vars) ([]makevars.Vars, error)
	UseMasksAndForces(pkg *Package, masks map[string]bool, forces map[string]bool) error
	ProvidedPackages() ([]*Package, error)
}

type Bundle []Source

var _ Source = Bundle{}

func (ss Bundle) EvalGlobalVars(env makevars.Vars) ([]makevars.Vars, error) {
	var varsList []makevars.Vars
	for _, s := range ss {
		subVarsList, err := s.EvalGlobalVars(env)
		if err != nil {
			return nil, err
		}
		varsList = append(varsList, subVarsList...)
	}
	return varsList, nil
}

func (ss Bundle) EvalPackageVars(pkg *Package, env makevars.Vars) ([]makevars.Vars, error) {
	var varsList []makevars.Vars
	for _, s := range ss {
		subVarsList, err := s.EvalPackageVars(pkg, env)
		if err != nil {
			return nil, err
		}
		varsList = append(varsList, subVarsList...)
	}
	return varsList, nil
}

func (ss Bundle) UseMasksAndForces(pkg *Package, masks map[string]bool, forces map[string]bool) error {
	for _, s := range ss {
		if err := s.UseMasksAndForces(pkg, masks, forces); err != nil {
			return err
		}
	}
	return nil
}

func (ss Bundle) ProvidedPackages() ([]*Package, error) {
	var pkgs []*Package
	for _, s := range ss {
		subpkgs, err := s.ProvidedPackages()
		if err != nil {
			return nil, err
		}
		pkgs = append(pkgs, subpkgs...)
	}
	return pkgs, nil
}

type HackSource struct {
	use      string
	provided []*Package
}

var _ Source = &HackSource{}

func NewHackSource(use string, provided []*Package) *HackSource {
	return &HackSource{
		use:      use,
		provided: provided,
	}
}

func (s *HackSource) EvalGlobalVars(env makevars.Vars) ([]makevars.Vars, error) {
	env["USE"] = s.use
	return []makevars.Vars{{"USE": s.use}}, nil
}

func (s *HackSource) EvalPackageVars(pkg *Package, env makevars.Vars) ([]makevars.Vars, error) {
	return s.EvalGlobalVars(env)
}

func (s *HackSource) UseMasksAndForces(pkg *Package, masks map[string]bool, forces map[string]bool) error {
	return nil
}

func (s *HackSource) ProvidedPackages() ([]*Package, error) {
	return append([]*Package(nil), s.provided...), nil
}
