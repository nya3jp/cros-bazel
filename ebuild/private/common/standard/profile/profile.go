// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package profile

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
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

func Load(path string, name string, resolver Resolver) (*Profile, error) {
	if _, err := os.Stat(path); err != nil {
		if errors.Is(err, fs.ErrNotExist) {
			return nil, fmt.Errorf("profile %s: not found", name)
		}
		return nil, fmt.Errorf("profile %s: %w", name, err)
	}

	parentPaths, err := config.ParseLines(filepath.Join(path, "parent"))
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
	var parents []*ParsedProfile
	for _, pp := range p.parents {
		parent, err := pp.Parse()
		if err != nil {
			return nil, err
		}
		parents = append(parents, parent)
	}

	// Just make sure make.defaults successfully parses. Actual evaluation is
	// done later.
	if _, err := makevars.Eval(filepath.Join(p.path, makeDefaults), make(makevars.Vars), false); err != nil && !errors.Is(err, fs.ErrNotExist) {
		return nil, err
	}

	packageUse, err := config.ParsePackageUseList(filepath.Join(p.path, "package.use"))
	if err != nil {
		return nil, err
	}

	useMask, err := config.ParseUseList(filepath.Join(p.path, "use.mask"))
	if err != nil {
		return nil, err
	}

	useForce, err := config.ParseUseList(filepath.Join(p.path, "use.force"))
	if err != nil {
		return nil, err
	}

	packageUseMask, err := config.ParsePackageUseList(filepath.Join(p.path, "package.use.mask"))
	if err != nil {
		return nil, err
	}

	packageUseForce, err := config.ParsePackageUseList(filepath.Join(p.path, "package.use.force"))
	if err != nil {
		return nil, err
	}

	packageMasks, err := config.ParseAtomList(filepath.Join(p.path, "package.mask"))
	if err != nil {
		return nil, err
	}

	provided, err := config.ParsePackageList(filepath.Join(p.path, "package.provided"))
	if err != nil {
		return nil, err
	}

	return &ParsedProfile{
		profile:         p,
		parents:         parents,
		useMask:         useMask,
		useForce:        useForce,
		packageUse:      packageUse,
		packageUseMask:  packageUseMask,
		packageUseForce: packageUseForce,
		packageMasks:    packageMasks,
		provided:        provided,
	}, nil
}

type ParsedProfile struct {
	profile         *Profile
	parents         []*ParsedProfile
	useMask         []string
	useForce        []string
	packageUse      []*config.PackageUse
	packageUseMask  []*config.PackageUse
	packageUseForce []*config.PackageUse
	packageMasks    []*dependency.Atom
	provided        []*config.TargetPackage
}

var _ config.Source = &ParsedProfile{}

func (p *ParsedProfile) EvalGlobalVars(env makevars.Vars) ([]makevars.Vars, error) {
	var varsList []makevars.Vars
	for _, parent := range p.parents {
		subVarsList, err := parent.EvalGlobalVars(env)
		if err != nil {
			return nil, err
		}
		varsList = append(varsList, subVarsList...)
	}

	vars, err := makevars.Eval(filepath.Join(p.profile.path, makeDefaults), env, false)
	if err != nil && !errors.Is(err, fs.ErrNotExist) {
		return nil, err
	}
	varsList = append(varsList, vars)

	return varsList, nil
}

func (p *ParsedProfile) EvalPackageVars(pkg *config.TargetPackage, env makevars.Vars) ([]makevars.Vars, error) {
	var varsList []makevars.Vars
	for _, parent := range p.parents {
		subVarsList, err := parent.EvalPackageVars(pkg, env)
		if err != nil {
			return nil, err
		}
		varsList = append(varsList, subVarsList...)
	}

	vars, err := makevars.Eval(filepath.Join(p.profile.path, makeDefaults), env, false)
	if err != nil && !errors.Is(err, fs.ErrNotExist) {
		return nil, err
	}
	varsList = append(varsList, vars)

	targetPkg := &dependency.TargetPackage{
		Name:     pkg.Name,
		Version:  pkg.Version,
		MainSlot: "",  // SLOT unavailable
		Uses:     nil, // USE dependencies are unavailable
	}
	var uses []string
	for _, pu := range p.packageUse {
		if pu.Atom.Match(targetPkg) {
			uses = append(uses, pu.Uses...)
		}
	}
	if len(uses) > 0 {
		varsList = append(varsList, makevars.Vars{"USE": strings.Join(uses, " ")})
	}

	return varsList, nil
}

func (p *ParsedProfile) UseMasksAndForces(pkg *config.TargetPackage, masks map[string]bool, forces map[string]bool) error {
	for _, parent := range p.parents {
		if err := parent.UseMasksAndForces(pkg, masks, forces); err != nil {
			return err
		}
	}

	updateMap := func(set map[string]bool, uses []string) {
		for _, use := range uses {
			value := true
			if strings.HasPrefix(use, "-") {
				use = strings.TrimPrefix(use, "-")
				value = false
			}
			set[use] = value
		}
	}

	updateMap(masks, p.useMask)
	updateMap(forces, p.useForce)

	targetPkg := &dependency.TargetPackage{
		Name:     pkg.Name,
		Version:  pkg.Version,
		MainSlot: "",  // SLOT unavailable
		Uses:     nil, // USE dependencies are unavailable
	}
	for _, pu := range p.packageUseMask {
		if pu.Atom.Match(targetPkg) {
			updateMap(masks, pu.Uses)
		}
	}
	for _, pu := range p.packageUseForce {
		if pu.Atom.Match(targetPkg) {
			updateMap(forces, pu.Uses)
		}
	}
	return nil
}

func (p *ParsedProfile) PackageMasks() ([]*dependency.Atom, error) {
	var atoms []*dependency.Atom
	for _, parent := range p.parents {
		subatoms, err := parent.PackageMasks()
		if err != nil {
			return nil, err
		}
		atoms = append(atoms, subatoms...)
	}
	return append(atoms, p.packageMasks...), nil
}

func (p *ParsedProfile) ProvidedPackages() ([]*config.TargetPackage, error) {
	var provided []*config.TargetPackage
	for _, parent := range p.parents {
		pkgs, err := parent.ProvidedPackages()
		if err != nil {
			return nil, err
		}
		provided = append(provided, pkgs...)
	}
	return append(provided, p.provided...), nil
}
