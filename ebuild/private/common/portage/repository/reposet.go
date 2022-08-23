package repository

import (
	"fmt"
	"path/filepath"
	"sort"
	"strings"

	"cros.local/rules_ebuild/ebuild/private/common/standard/dependency"
	"cros.local/rules_ebuild/ebuild/private/common/standard/ebuild"
	"cros.local/rules_ebuild/ebuild/private/common/standard/packages"
	"cros.local/rules_ebuild/ebuild/private/common/standard/profile"
)

type RepoSet struct {
	ordered []*Repo
	byName  map[string]*Repo
}

func NewRepoSet(rootDirs []string) (*RepoSet, error) {
	repoSet := &RepoSet{
		byName: make(map[string]*Repo),
	}

	for _, rootDir := range rootDirs {
		repo, err := parseRepo(repoSet, rootDir)
		if err != nil {
			return nil, fmt.Errorf("failed to parse repo: %s: %w", rootDir, err)
		}
		repoSet.ordered = append(repoSet.ordered, repo)
		repoSet.byName[repo.Name()] = repo
	}

	return repoSet, nil
}

func (s *RepoSet) Repo(name string) (*Repo, bool) {
	repo, ok := s.byName[name]
	return repo, ok
}

func (s *RepoSet) Profile(name string) (*profile.Profile, error) {
	segments := strings.SplitN(name, ":", -1)
	if len(segments) != 2 {
		return nil, fmt.Errorf("invalid profile name: %s (must be <repo-name>:<profile-path>)", name)
	}
	repo, ok := s.byName[segments[0]]
	if !ok {
		return nil, fmt.Errorf("profile not found: %s (repository %s does not exist)", name, segments[0])
	}
	return repo.Profile(segments[1])
}

func (s *RepoSet) ProfileByPath(path string) (*profile.Profile, error) {
	path = filepath.Clean(path)

	// TODO: Improve efficiency.
	for _, repo := range s.byName {
		profilesDir := filepath.Join(repo.RootDir(), "profiles") + "/"
		if strings.HasPrefix(path, profilesDir) {
			return repo.Profile(path[len(profilesDir):])
		}
	}
	return nil, fmt.Errorf("profile not found at %s (not under known repository directory)", path)
}

func (s *RepoSet) EClassDirs() []string {
	var dirs []string
	for _, repo := range s.ordered {
		dirs = append(dirs, filepath.Join(repo.RootDir(), "eclass"))
	}
	return dirs
}

func (s *RepoSet) Package(atom *dependency.Atom, processor *ebuild.CachedProcessor) ([]*packages.Package, error) {
	var pkgs []*packages.Package
	for _, repo := range s.ordered {
		repoPkgs, err := repo.Package(atom, processor)
		if err != nil {
			return nil, err
		}
		pkgs = append(pkgs, repoPkgs...)
	}

	sort.SliceStable(pkgs, func(i, j int) bool {
		return pkgs[i].Version().Compare(pkgs[j].Version()) > 0
	})

	return pkgs, nil
}
