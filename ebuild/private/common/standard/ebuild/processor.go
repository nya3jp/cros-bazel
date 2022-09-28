// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package ebuild

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/bashutil"
	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Metadata map[string]string

type Info struct {
	Metadata Metadata
	Uses     map[string]bool
}

type Processor struct {
	config     config.Source
	eclassDirs []string
}

func NewProcessor(config config.Source, eclassDirs []string) *Processor {
	return &Processor{
		config:     config,
		eclassDirs: eclassDirs,
	}
}

func (p *Processor) Read(path string) (*Info, error) {
	absPath, err := filepath.Abs(path)
	if err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	pkg, err := extractPackage(path)
	if err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	env := make(makevars.Vars)
	if _, err := p.config.EvalGlobalVars(env); err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	env.Merge(computePackageVars(pkg))

	metadata, err := runEBuild(absPath, env, p.eclassDirs)
	if err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	uses, err := computeUseFlags(pkg, p.config, metadata)
	if err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	return &Info{
		Metadata: metadata,
		Uses:     uses,
	}, nil
}

func extractPackage(absPath string) (*config.Package, error) {
	const suffix = ".ebuild"
	if !strings.HasSuffix(absPath, suffix) {
		return nil, fmt.Errorf("must have suffix %s", suffix)
	}

	packageShortNameAndVersion := filepath.Base(strings.TrimSuffix(absPath, suffix))
	packageShortName := filepath.Base(filepath.Dir(absPath))
	categoryName := filepath.Base(filepath.Dir(filepath.Dir(absPath)))

	packageShortNameAndHyphen, version, err := version.ExtractSuffix(packageShortNameAndVersion)
	if err != nil {
		return nil, err
	}
	if !strings.HasSuffix(packageShortNameAndHyphen, "-") {
		return nil, errors.New("invalid package name")
	}
	packageShortName2 := strings.TrimSuffix(packageShortNameAndHyphen, "-")
	if packageShortName2 != packageShortName {
		return nil, errors.New("ebuild name mismatch with directory name")
	}

	return &config.Package{
		Name:    path.Join(categoryName, packageShortName),
		Version: version,
	}, nil
}

func computePackageVars(pkg *config.Package) makevars.Vars {
	categoryName := path.Dir(pkg.Name)
	packageShortName := path.Base(pkg.Name)

	return makevars.Vars{
		"P":        fmt.Sprintf("%s-%s", packageShortName, pkg.Version.DropRevision().String()),
		"PF":       fmt.Sprintf("%s-%s", packageShortName, pkg.Version.String()),
		"PN":       packageShortName,
		"CATEGORY": categoryName,
		"PV":       pkg.Version.DropRevision().String(),
		"PR":       fmt.Sprintf("r%s", pkg.Version.Revision),
		"PVR":      pkg.Version.String(),
	}
}

func runEBuild(absPath string, env makevars.Vars, eclassDirs []string) (Metadata, error) {
	tempDir, err := os.MkdirTemp("", "xbuild.*")
	if err != nil {
		return nil, err
	}
	defer os.RemoveAll(tempDir)

	workDir := filepath.Join(tempDir, "work")
	if err := os.Mkdir(workDir, 0700); err != nil {
		return nil, err
	}

	outPath := filepath.Join(tempDir, "vars.txt")

	vars := make(makevars.Vars)
	vars.Merge(env, makevars.Vars{
		"__xbuild_in_ebuild":      absPath,
		"__xbuild_in_eclass_dirs": strings.Join(eclassDirs, "\n") + "\n",
		"__xbuild_in_output_vars": outPath,
	})

	cmd := exec.Command("bash")
	cmd.Stdin = bytes.NewBuffer(preludeCode)
	cmd.Env = vars.Environ()
	cmd.Dir = workDir
	if out, err := cmd.CombinedOutput(); len(out) > 0 {
		os.Stderr.Write(out)
		return nil, errors.New("ebuild printed errors to stdout/stderr (see logs)")
	} else if err != nil {
		return nil, fmt.Errorf("bash: %w", err)
	}

	b, err := os.ReadFile(outPath)
	if err != nil {
		return nil, err
	}

	out, err := bashutil.ParseSetOutput(bytes.NewBuffer(b))
	if err != nil {
		return nil, fmt.Errorf("reading output: %w", err)
	}

	// Remove internal variables.
	for name := range out {
		if strings.HasPrefix(name, "__xbuild_") {
			delete(out, name)
		}
	}
	return out, nil
}

type readResult struct {
	info *Info
	err  error
}

type CachedProcessor struct {
	p     *Processor
	cache map[string]readResult
}

func NewCachedProcessor(p *Processor) *CachedProcessor {
	return &CachedProcessor{
		p:     p,
		cache: make(map[string]readResult),
	}
}

func (p *CachedProcessor) Read(path string) (*Info, error) {
	if res, ok := p.cache[path]; ok {
		return res.info, res.err
	}
	info, err := p.p.Read(path)
	p.cache[path] = readResult{info: info, err: err}
	return info, err
}
