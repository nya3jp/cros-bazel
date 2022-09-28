// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package packages

import (
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/ebuild"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Package struct {
	path     string
	metadata ebuild.Metadata
	target   *dependency.TargetPackage
}

func NewPackage(path string, metadata ebuild.Metadata, target *dependency.TargetPackage) *Package {
	return &Package{
		path:     path,
		metadata: metadata,
		target:   target,
	}
}

func (p *Package) Path() string                             { return p.path }
func (p *Package) Name() string                             { return p.target.Name }
func (p *Package) Category() string                         { return strings.Split(p.target.Name, "/")[0] }
func (p *Package) Version() *version.Version                { return p.target.Version }
func (p *Package) Uses() map[string]bool                    { return p.target.Uses }
func (p *Package) Metadata() ebuild.Metadata                { return p.metadata }
func (p *Package) TargetPackage() *dependency.TargetPackage { return p.target }

func (p *Package) MainSlot() string {
	slot := p.metadata["SLOT"]
	return strings.SplitN(slot, "/", 2)[0]
}

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
	eclasses := strings.Split(p.metadata["USED_ECLASSES"], "|")
	for _, used_eclass := range eclasses {
		if used_eclass == eclass {
			return true
		}
	}

	return false
}
