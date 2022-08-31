// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package packages

import (
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Package struct {
	path   string
	vars   makevars.Vars
	target *dependency.TargetPackage
}

func NewPackage(path string, vars makevars.Vars, target *dependency.TargetPackage) *Package {
	return &Package{
		path:   path,
		vars:   vars,
		target: target,
	}
}

func (p *Package) Path() string                             { return p.path }
func (p *Package) Name() string                             { return p.target.Name }
func (p *Package) Version() *version.Version                { return p.target.Version }
func (p *Package) Vars() makevars.Vars                      { return p.vars.Copy() }
func (p *Package) TargetPackage() *dependency.TargetPackage { return p.target }

func (p *Package) MainSlot() string {
	slot := p.vars["SLOT"]
	return strings.SplitN(slot, "/", 2)[0]
}

func (p *Package) Stability() Stability {
	arch := p.vars["ARCH"]
	keywords := p.vars.GetAsSet("KEYWORDS")
	for _, s := range []string{arch, "*"} {
		if _, ok := keywords[s]; ok {
			return StabilityStable
		}
		if _, ok := keywords["~"+s]; ok {
			return StabilityTesting
		}
		if _, ok := keywords["-"+s]; ok {
			return StabilityBroken
		}
	}
	return StabilityTesting
}

func SelectByStability(pkgs []*Package) []*Package {
	if len(pkgs) == 0 {
		return nil
	}

	candidates := make(map[Stability][]*Package)
	for _, pkg := range pkgs {
		s := pkg.Stability()
		candidates[s] = append(candidates[s], pkg)
	}

	if stable := candidates[StabilityStable]; len(stable) > 0 {
		return stable
	}
	if testing := candidates[StabilityTesting]; len(testing) > 0 {
		return testing
	}
	return nil
}
