// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package config

import (
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

// TargetPackage represents a package to compute configurations for.
// It only contains info that can be extracted without evaluating ebuilds
// because configurations are used to evaluate ebuilds.
type TargetPackage struct {
	Name    string
	Version *version.Version
}

type Source interface {
	EvalGlobalVars(env makevars.Vars) ([]makevars.Vars, error)
	EvalPackageVars(pkg *TargetPackage, env makevars.Vars) ([]makevars.Vars, error)
	UseMasksAndForces(pkg *TargetPackage, masks map[string]bool, forces map[string]bool) error
	PackageMasks() ([]*dependency.Atom, error)
	ProvidedPackages() ([]*TargetPackage, error)
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

func (ss Bundle) EvalPackageVars(pkg *TargetPackage, env makevars.Vars) ([]makevars.Vars, error) {
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

func (ss Bundle) UseMasksAndForces(pkg *TargetPackage, masks map[string]bool, forces map[string]bool) error {
	for _, s := range ss {
		if err := s.UseMasksAndForces(pkg, masks, forces); err != nil {
			return err
		}
	}
	return nil
}

func (ss Bundle) PackageMasks() ([]*dependency.Atom, error) {
	var atoms []*dependency.Atom
	for _, s := range ss {
		subatoms, err := s.PackageMasks()
		if err != nil {
			return nil, err
		}
		atoms = append(atoms, subatoms...)
	}
	return atoms, nil
}

func (ss Bundle) ProvidedPackages() ([]*TargetPackage, error) {
	var pkgs []*TargetPackage
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
	provided []*TargetPackage
}

var _ Source = &HackSource{}

func NewHackSource(use string, provided []*TargetPackage) *HackSource {
	return &HackSource{
		use:      use,
		provided: provided,
	}
}

func (s *HackSource) EvalGlobalVars(env makevars.Vars) ([]makevars.Vars, error) {
	env["USE"] = s.use
	return []makevars.Vars{{"USE": s.use}}, nil
}

func (s *HackSource) EvalPackageVars(pkg *TargetPackage, env makevars.Vars) ([]makevars.Vars, error) {
	return s.EvalGlobalVars(env)
}

func (s *HackSource) UseMasksAndForces(pkg *TargetPackage, masks map[string]bool, forces map[string]bool) error {
	return nil
}

func (s *HackSource) PackageMasks() ([]*dependency.Atom, error) {
	return nil, nil
}

func (s *HackSource) ProvidedPackages() ([]*TargetPackage, error) {
	return append([]*TargetPackage(nil), s.provided...), nil
}
