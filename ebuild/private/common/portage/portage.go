// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package portage

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"cros.local/bazel/ebuild/private/common/portage/makeconf"
	"cros.local/bazel/ebuild/private/common/portage/portagevars"
	"cros.local/bazel/ebuild/private/common/portage/repository"
	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/ebuild"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/packages"
)

type Resolver struct {
	repoSet   *repository.RepoSet
	bundle    config.Bundle
	processor *ebuild.CachedProcessor
	masks     []*dependency.Atom
	provided  map[string][]*config.TargetPackage
}

func NewResolver(rootDir string, extraSources ...config.Source) (*Resolver, error) {
	userConfigSource := makeconf.NewUserConfigSource(rootDir)

	bootEnv := make(makevars.Vars)
	if _, err := userConfigSource.EvalGlobalVars(bootEnv); err != nil {
		return nil, err
	}

	overlays := portagevars.Overlays(bootEnv)

	repoSet, err := repository.NewRepoSet(overlays)
	if err != nil {
		return nil, err
	}

	profilePath, err := os.Readlink(filepath.Join(rootDir, "etc/portage/make.profile"))
	if err != nil {
		return nil, err
	}

	if !filepath.IsAbs(profilePath) {
		profilePath = filepath.Clean(filepath.Join(rootDir, "etc/portage", profilePath))
	}

	rawProfile, err := repoSet.ProfileByPath(profilePath)
	if err != nil {
		return nil, err
	}

	profile, err := rawProfile.Parse()
	if err != nil {
		return nil, err
	}

	bundle := config.Bundle(append([]config.Source{profile, userConfigSource}, extraSources...))

	processor := ebuild.NewCachedProcessor(ebuild.NewProcessor(bundle, repoSet.EClassDirs()))

	masks, err := bundle.PackageMasks()
	if err != nil {
		return nil, err
	}

	rawProvided, err := bundle.ProvidedPackages()
	if err != nil {
		return nil, err
	}

	provided := make(map[string][]*config.TargetPackage)
	for _, pkg := range rawProvided {
		provided[pkg.Name] = append(provided[pkg.Name], pkg)
	}

	return &Resolver{
		repoSet:   repoSet,
		bundle:    bundle,
		processor: processor,
		masks:     masks,
		provided:  provided,
	}, nil
}

func (r *Resolver) Config() config.Source {
	return r.bundle
}

func (r *Resolver) Packages(atom *dependency.Atom) ([]*packages.Package, error) {
	repoPkgs, err := r.repoSet.Packages(atom.PackageName())
	if err != nil {
		return nil, err
	}

	var pkgs []*packages.Package
	for _, repoPkg := range repoPkgs {
		info, err := r.processor.Read(repoPkg.Path)
		if err != nil {
			fmt.Fprintf(os.Stderr, "WARNING: Ignored ebuild: failed to evaluate %s: %v\n", repoPkg.Path, err)
			continue
		}

		target := &dependency.TargetPackage{
			Name:     atom.PackageName(),
			Version:  repoPkg.Version,
			MainSlot: strings.SplitN(info.Metadata["SLOT"], "/", 2)[0],
			Uses:     info.Uses,
		}

		masked := false
		for _, mask := range r.masks {
			if mask.Match(target) {
				masked = true
				break
			}
		}
		if masked {
			continue
		}

		if atom.Match(target) {
			pkgs = append(pkgs, packages.NewPackage(repoPkg.Path, info.Metadata, target))
		}
	}

	sort.SliceStable(pkgs, func(i, j int) bool {
		return pkgs[i].Version().Compare(pkgs[j].Version()) > 0
	})

	return pkgs, nil
}

func (r *Resolver) BestPackage(atom *dependency.Atom) (*packages.Package, error) {
	pkgs, err := r.Packages(atom)
	if err != nil {
		return nil, err
	}

	pkgs = packages.SelectByStability(pkgs)
	if len(pkgs) == 0 {
		return nil, fmt.Errorf("no package satisfies %s", atom.String())
	}
	return pkgs[0], nil
}

func (r *Resolver) IsProvided(atom *dependency.Atom) bool {
	for _, pkg := range r.provided[atom.PackageName()] {
		if atom.Match(&dependency.TargetPackage{
			Name:     pkg.Name,
			Version:  pkg.Version,
			MainSlot: "",  // SLOT unavailable in package.provided
			Uses:     nil, // USE flags unavailable in package.provided
		}) {
			return true
		}
	}
	return false
}
