// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package repository

import (
	"os"
	"path/filepath"

	"cros.local/bazel/ebuild/private/common/portage/makeconf"
	"cros.local/bazel/ebuild/private/common/portage/portagevars"
	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/profile"
)

type Defaults struct {
	RepoSet *RepoSet
	Profile *profile.ParsedProfile
	Config  config.Bundle
}

// LoadDefaults is a convenient function to load the default RepoSet and its
// related objects.
func LoadDefaults(rootDir string, extraSources ...config.Source) (*Defaults, error) {
	userConfigSource := makeconf.NewUserConfigSource(rootDir)

	bootEnv := make(makevars.Vars)
	if _, err := userConfigSource.EvalGlobalVars(bootEnv); err != nil {
		return nil, err
	}

	overlays := portagevars.Overlays(bootEnv)

	repoSet, err := NewRepoSet(overlays)
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

	return &Defaults{
		RepoSet: repoSet,
		Profile: profile,
		Config:  config.Bundle(append([]config.Source{profile, userConfigSource}, extraSources...)),
	}, nil
}
