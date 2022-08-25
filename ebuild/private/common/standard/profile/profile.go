package profile

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/makevars"
)

type Resolver interface {
	ResolveProfile(path, base string) (*Profile, error)
}

type Profile struct {
	name      string
	path      string
	parents   []*Profile
	localVars makevars.Vars
	vars      makevars.Vars
}

func Parse(path string, name string, resolver Resolver) (*Profile, error) {
	if _, err := os.Stat(path); err != nil {
		if errors.Is(err, fs.ErrNotExist) {
			return nil, fmt.Errorf("profile %s: not found", name)
		}
		return nil, fmt.Errorf("profile %s: %w", name, err)
	}

	parentPaths, err := readParents(filepath.Join(path, "parent"))
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

	localVars, err := makevars.ParseMakeDefaults(filepath.Join(path, "make.defaults"))
	if err != nil && !errors.Is(err, fs.ErrNotExist) {
		return nil, fmt.Errorf("profile %s: %w", name, err)
	}

	var parentVars []makevars.Vars
	for _, parent := range parents {
		parentVars = append(parentVars, parent.Vars())
	}
	vars := makevars.Merge(append(append([]makevars.Vars(nil), parentVars...), localVars)...)

	return &Profile{
		name:      name,
		path:      path,
		parents:   parents,
		localVars: localVars,
		vars:      vars,
	}, nil
}

func (p *Profile) Name() string        { return p.name }
func (p *Profile) Path() string        { return p.path }
func (p *Profile) Parents() []*Profile { return append([]*Profile(nil), p.parents...) }

func (p *Profile) Vars() makevars.Vars {
	// Make a copy.
	vars := make(makevars.Vars)
	for name, value := range p.vars {
		vars[name] = value
	}
	return vars
}

func (p *Profile) DumpForTesting(w io.Writer) {
	fmt.Fprint(w, "[vars]\n")
	var names []string
	for name := range p.vars {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		value := p.vars[name]
		fmt.Fprintf(w, "    %s=%s\n", name, value)
	}

	var printTree func(p *Profile, depth int)
	printTree = func(p *Profile, depth int) {
		fmt.Fprintf(w, "%s%s VIDEO_CARDS=%s\n", strings.Repeat("  ", depth), p.name, p.localVars["VIDEO_CARDS"])
		for _, pp := range p.parents {
			printTree(pp, depth+1)
		}
	}
	fmt.Fprint(w, "[tree]\n")
	printTree(p, 0)
}

func readParents(path string) ([]string, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	var names []string
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		// PMS doesn't allow comments, but in reality there are comments.
		line := strings.TrimSpace(strings.SplitN(sc.Text(), "#", 2)[0])
		if line == "" {
			continue
		}
		names = append(names, line)
	}
	if err := sc.Err(); err != nil {
		return nil, err
	}
	return names, nil
}
