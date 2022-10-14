// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package packages

import (
	"sort"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/ebuild"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Package struct {
	path     string
	metadata ebuild.Metadata
	target   *dependency.TargetPackage
	eclasses []string
}

func NewPackage(path string, metadata ebuild.Metadata, target *dependency.TargetPackage) *Package {
	eclasses := strings.Split(metadata["USED_ECLASSES"], "|")

	sort.Strings(eclasses)

	return &Package{
		path:     path,
		metadata: metadata,
		target:   target,
		eclasses: eclasses,
	}
}

func (p *Package) Path() string                             { return p.path }
func (p *Package) Name() string                             { return p.target.Name }
func (p *Package) Category() string                         { return strings.Split(p.target.Name, "/")[0] }
func (p *Package) MainSlot() string                         { return p.target.MainSlot }
func (p *Package) Version() *version.Version                { return p.target.Version }
func (p *Package) Uses() map[string]bool                    { return p.target.Uses }
func (p *Package) Metadata() ebuild.Metadata                { return p.metadata }
func (p *Package) TargetPackage() *dependency.TargetPackage { return p.target }
func (p *Package) Eclasses() []string                       { return p.eclasses }

func (p *Package) Stability() Stability {
	arch := p.metadata["ARCH"]
	keywordSet := make(map[string]struct{})
	for _, k := range strings.Fields(p.metadata["KEYWORDS"]) {
		keywordSet[k] = struct{}{}
	}

	for _, s := range []string{arch, "*"} {
		if _, ok := keywordSet[s]; ok {
			return StabilityStable
		}
		if _, ok := keywordSet["~"+s]; ok {
			return StabilityTesting
		}
		if _, ok := keywordSet["-"+s]; ok {
			return StabilityBroken
		}
	}
	return StabilityTesting
}

func (p *Package) UsesEclass(eclass string) bool {
	for _, used_eclass := range p.eclasses {
		if used_eclass == eclass {
			return true
		}
	}

	return false
}
