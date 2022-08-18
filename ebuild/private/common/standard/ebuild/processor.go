package ebuild

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"cros.local/ebuild/private/common/standard/makevars"
	"cros.local/ebuild/private/common/standard/version"
)

type Processor struct {
	profileVars makevars.Vars
	eclassDirs  []string
}

func NewProcessor(profileVars makevars.Vars, eclassDirs []string) *Processor {
	return &Processor{
		profileVars: profileVars,
		eclassDirs:  eclassDirs,
	}
}

func (p *Processor) ReadMetadata(path string) (makevars.Vars, error) {
	absPath, err := filepath.Abs(path)
	if err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	packageVars, err := computePackageVars(absPath)
	if err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	allVars := makevars.Merge(p.profileVars, packageVars)
	outVars, err := runEBuild(absPath, allVars, p.eclassDirs)
	if err != nil {
		return nil, fmt.Errorf("reading ebuild metadata: %s: %w", absPath, err)
	}

	return outVars, nil
}

func computePackageVars(absPath string) (makevars.Vars, error) {
	const suffix = ".ebuild"
	if !strings.HasSuffix(absPath, suffix) {
		return nil, fmt.Errorf("must have suffix %s", suffix)
	}

	packageNameAndVersion := filepath.Base(strings.TrimSuffix(absPath, suffix))
	categoryName := filepath.Base(filepath.Dir(filepath.Dir(absPath)))

	packageNameAndHyphen, version, err := version.ExtractSuffix(packageNameAndVersion)
	if err != nil {
		return nil, err
	}
	if !strings.HasSuffix(packageNameAndHyphen, "-") {
		return nil, errors.New("invalid package name")
	}
	packageName := strings.TrimSuffix(packageNameAndHyphen, "-")

	return makevars.Vars{
		"P":        fmt.Sprintf("%s-%s", packageName, version.DropRevision().String()),
		"PF":       fmt.Sprintf("%s-%s", packageName, version.String()),
		"PN":       packageName,
		"CATEGORY": categoryName,
		"PV":       version.DropRevision().String(),
		"PR":       fmt.Sprintf("r%s", version.Revision),
		"PVR":      version.String(),
	}, nil
}

func runEBuild(absPath string, vars makevars.Vars, eclassDirs []string) (makevars.Vars, error) {
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

	internalVars := makevars.Vars{
		"__xbuild_in_ebuild":      absPath,
		"__xbuild_in_eclass_dirs": strings.Join(eclassDirs, "\n") + "\n",
		"__xbuild_in_output_vars": outPath,
	}
	finalVars := makevars.Merge(vars, internalVars)

	cmd := exec.Command("bash")
	cmd.Stdin = bytes.NewBuffer(preludeCode)
	cmd.Env = finalVars.Environ()
	cmd.Dir = workDir
	if out, err := cmd.CombinedOutput(); len(out) > 0 {
		os.Stderr.Write(out)
		return nil, errors.New("ebuild printed errors to stdout/stderr (see logs)")
	} else if err != nil {
		return nil, fmt.Errorf("bash: %w", err)
	}

	outVars, err := makevars.ParseSetOutput(outPath)
	if err != nil {
		return nil, fmt.Errorf("reading output: %w", err)
	}

	// Remove internal variables.
	for name := range outVars {
		if strings.HasPrefix(name, "__xbuild_") {
			delete(outVars, name)
		}
	}
	return outVars, nil
}

type readMetadataResult struct {
	vars makevars.Vars
	err  error
}

type CachedProcessor struct {
	p                   *Processor
	readMetadataResults map[string]readMetadataResult
}

func NewCachedProcessor(p *Processor) *CachedProcessor {
	return &CachedProcessor{
		p:                   p,
		readMetadataResults: make(map[string]readMetadataResult),
	}
}

func (p *CachedProcessor) ReadMetadata(path string) (makevars.Vars, error) {
	if res, ok := p.readMetadataResults[path]; ok {
		return res.vars, res.err
	}
	vars, err := p.p.ReadMetadata(path)
	p.readMetadataResults[path] = readMetadataResult{vars: vars, err: err}
	return vars, err
}
