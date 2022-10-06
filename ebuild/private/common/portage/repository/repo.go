// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package repository

import (
	"bufio"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"sync"

	"cros.local/bazel/ebuild/private/common/standard/profile"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Repo struct {
	repoSet  *RepoSet
	rootDir  string
	name     string
	eapi     string
	profiles *repoProfiles
}

type Package struct {
	Path    string
	Version *version.Version
}

func parseRepo(repoSet *RepoSet, rootDir string) (*Repo, error) {
	layout, err := readLayoutConf(filepath.Join(rootDir, "metadata", "layout.conf"))
	if err != nil && !errors.Is(err, fs.ErrNotExist) {
		return nil, err
	}

	name, err := readSingleLineFile(filepath.Join(rootDir, "profiles", "repo_name"))
	if errors.Is(err, fs.ErrNotExist) {
		// PMS mandates repo_name to exist, but overlays often miss it.
		name = layout["repo-name"]
	} else if err != nil {
		return nil, err
	}

	// Auto-generate name if it's missing.
	if name == "" {
		name = fmt.Sprintf("x-%s", filepath.Base(rootDir))
	}

	eapi, err := readSingleLineFile(filepath.Join(rootDir, "profiles", "eapi"))
	if errors.Is(err, fs.ErrNotExist) {
		eapi = "0"
	} else if err != nil {
		return nil, err
	}

	return &Repo{
		repoSet:  repoSet,
		rootDir:  rootDir,
		name:     name,
		eapi:     eapi,
		profiles: newRepoProfiles(),
	}, nil
}

func (r *Repo) RepoSet() *RepoSet { return r.repoSet }
func (r *Repo) RootDir() string   { return r.rootDir }
func (r *Repo) Name() string      { return r.name }
func (r *Repo) EAPI() string      { return r.eapi }

func (r *Repo) Profile(relPath string) (*profile.Profile, error) {
	return r.profiles.LookupOrParse(r, relPath)
}

func (r *Repo) Packages(packageName string) ([]*Package, error) {
	packageDir := filepath.Join(r.rootDir, packageName)
	es, err := os.ReadDir(packageDir)
	if errors.Is(err, os.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	var pkgs []*Package

	const ebuildExt = ".ebuild"
	prefix := path.Base(packageName) + "-"
	for _, e := range es {
		if !e.Type().IsRegular() && e.Type() != os.ModeSymlink {
			continue
		}
		if !strings.HasPrefix(e.Name(), prefix) {
			continue
		}
		if !strings.HasSuffix(e.Name(), ebuildExt) {
			continue
		}
		verStr := strings.TrimSuffix(strings.TrimPrefix(e.Name(), prefix), ebuildExt)

		fullPath := filepath.Join(packageDir, e.Name())

		ver, err := version.Parse(verStr)
		if err != nil {
			fmt.Fprintf(os.Stderr, "WARNING: Ignored ebuild: invalid version: %s\n", fullPath)
			continue
		}

		pkgs = append(pkgs, &Package{
			Path:    fullPath,
			Version: ver,
		})
	}

	sort.Slice(pkgs, func(i, j int) bool {
		return pkgs[i].Version.Compare(pkgs[i].Version) > 0
	})
	return pkgs, nil
}

type repoProfiles struct {
	mu    sync.RWMutex
	cache map[string]*profile.Profile
}

func newRepoProfiles() *repoProfiles {
	return &repoProfiles{
		cache: make(map[string]*profile.Profile),
	}
}

func (p *repoProfiles) LookupOrParse(repo *Repo, relPath string) (*profile.Profile, error) {
	relPath = filepath.Clean(strings.TrimRight(relPath, "/"))
	if filepath.IsAbs(relPath) || strings.HasPrefix(relPath, "..") {
		return nil, fmt.Errorf("invalid profile path %s", relPath)
	}

	p.mu.RLock()
	prof, ok := p.cache[relPath]
	p.mu.RUnlock()

	if ok {
		return prof, nil
	}

	// parseProfile may call into this method recursively. Release the mutex
	// before proceeding.
	path := filepath.Join(repo.RootDir(), "profiles", relPath)
	name := fmt.Sprintf("%s:%s", repo.Name(), relPath)
	resolver := &repoSetProfileResolver{repoSet: repo.RepoSet()}
	prof, err := profile.Load(path, name, resolver)
	if err != nil {
		return nil, err
	}

	p.mu.Lock()
	p.cache[relPath] = prof
	p.mu.Unlock()
	return prof, nil
}

func readSingleLineFile(path string) (string, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(b)), nil
}

func readLayoutConf(path string) (map[string]string, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	kvs := make(map[string]string)

	sc := bufio.NewScanner(f)
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		segments := strings.SplitN(line, "=", 2)
		if len(segments) != 2 {
			return nil, fmt.Errorf("%s: corrupted format", path)
		}
		key := strings.TrimSpace(segments[0])
		value := strings.TrimSpace(segments[1])
		kvs[key] = value
	}

	if err := sc.Err(); err != nil {
		return nil, fmt.Errorf("%s: %w", path, err)
	}
	return kvs, nil
}

type repoSetProfileResolver struct {
	repoSet *RepoSet
}

var _ profile.Resolver = &repoSetProfileResolver{}

func (r *repoSetProfileResolver) ResolveProfile(path, base string) (*profile.Profile, error) {
	if strings.Contains(path, ":") {
		return r.repoSet.Profile(path)
	}
	return r.repoSet.ProfileByPath(filepath.Join(base, path))
}
