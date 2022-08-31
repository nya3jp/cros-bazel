// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/common/portage/makeconf"
	"cros.local/bazel/ebuild/private/common/portage/portagevars"
	"cros.local/bazel/ebuild/private/common/portage/repository"
	"cros.local/bazel/ebuild/private/common/runfiles"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/ebuild"
)

var knownMissingPackages = map[string]struct{}{
	"media-libs/jpeg":         {},
	"net-firewall/nftables":   {},
	"sys-freebsd/freebsd-lib": {},
	"sys-fs/eudev":            {},
	// "sys-libs/e2fsprogs-libs": {},
	// "sys-libs/glibc": {},
}

func simplifyDeps(deps *dependency.Deps, use map[string]struct{}) *dependency.Deps {
	deps = dependency.ResolveUse(deps, use)

	// Rewrite package atoms.
	deps = dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return expr
		}

		// Remove blocks.
		if pkg.Blocks() > 0 {
			return dependency.ConstTrue
		}

		// Remove known missing packages.
		if _, missing := knownMissingPackages[pkg.Atom().PackageName()]; missing {
			return dependency.ConstFalse
		}

		// Strip modifiers.
		atom := dependency.NewAtom(pkg.Atom().PackageName(), dependency.OpNone, nil, false, "", nil)
		return dependency.NewPackage(atom, 0)
	})

	deps = dependency.Simplify(deps)

	// Unify AnyOf whose children refer to the same package.
	deps = dependency.Map(deps, func(expr dependency.Expr) dependency.Expr {
		anyOf, ok := expr.(*dependency.AnyOf)
		if !ok {
			return expr
		}

		children := anyOf.Children()
		if len(children) == 0 {
			return expr
		}

		pkg0, ok := children[0].(*dependency.Package)
		if !ok {
			return expr
		}

		same := true
		for _, child := range children {
			pkg, ok := child.(*dependency.Package)
			if ok && pkg.Atom().PackageName() == pkg0.Atom().PackageName() {
				continue
			}
			same = false
			break
		}
		if !same {
			return expr
		}
		return pkg0
	})

	deps = dependency.Simplify(deps)
	return deps
}

func parseSimpleDeps(deps *dependency.Deps) ([]string, bool) {
	nameSet := make(map[string]struct{})
	for _, expr := range deps.Expr().Children() {
		pkg, ok := expr.(*dependency.Package)
		if !ok {
			return nil, false
		}
		nameSet[pkg.Atom().PackageName()] = struct{}{}
	}

	names := make([]string, 0, len(nameSet))
	for name := range nameSet {
		names = append(names, name)
	}
	sort.Strings(names)

	return names, true
}

var flagPackages = &cli.StringFlag{
	Name:     "packages",
	Required: true,
}

type packageInfo struct {
	BuildDeps            []string `json:"buildDeps"`
	RuntimeDeps          []string `json:"runtimeDeps"`
	ProcessedBuildDeps   string   `json:"processedBuildDeps"`
	ProcessedRuntimeDeps string   `json:"processedRuntimeDeps"`
	RawBuildDeps         string   `json:"rawBuildDeps"`
	RawRuntimeDeps       string   `json:"rawRuntimeDeps"`
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagPackages,
	},
	Action: func(c *cli.Context) error {
		packagesPath := c.String(flagPackages.Name)

		// TODO: Stop hard-coding.
		const rootDir = "/build/arm64-generic"

		b, err := os.ReadFile(packagesPath)
		if err != nil {
			return err
		}
		packageNames := strings.Split(strings.TrimSpace(string(b)), "\n")
		sort.Strings(packageNames)

		configVars, err := makeconf.ParseDefaults(rootDir)
		if err != nil {
			return err
		}

		overlays := portagevars.Overlays(configVars)

		repoSet, err := repository.NewRepoSet(overlays)
		if err != nil {
			return err
		}

		profilePath, err := os.Readlink(filepath.Join(rootDir, "etc/portage/make.profile"))
		if err != nil {
			return fmt.Errorf("detecting default profile: %w", err)
		}

		profile, err := repoSet.ProfileByPath(profilePath)
		if err != nil {
			return err
		}

		processor := ebuild.NewCachedProcessor(ebuild.NewProcessor(profile.Vars(), repoSet.EClassDirs()))

		infoMap := make(map[string]*packageInfo)

		for _, packageName := range packageNames {
			if err := func() error {
				atom, err := dependency.ParseAtom(packageName)
				if err != nil {
					return err
				}

				pkg, err := repoSet.BestPackage(atom, processor)
				if err != nil {
					return err
				}

				vars := pkg.Vars()
				use := vars.ComputeUse()

				buildDeps, err := dependency.Parse(vars["DEPEND"])
				if err != nil {
					return err
				}

				runtimeDeps, err := dependency.Parse(vars["RDEPEND"])
				if err != nil {
					return err
				}

				simpleBuildDeps := simplifyDeps(buildDeps, use)
				simpleRuntimeDeps := simplifyDeps(runtimeDeps, use)

				info := &packageInfo{
					ProcessedBuildDeps:   simpleBuildDeps.String(),
					ProcessedRuntimeDeps: simpleRuntimeDeps.String(),
					RawBuildDeps:         buildDeps.String(),
					RawRuntimeDeps:       runtimeDeps.String(),
				}
				if parsedDeps, ok := parseSimpleDeps(simpleBuildDeps); ok {
					info.BuildDeps = parsedDeps
				}
				if parsedDeps, ok := parseSimpleDeps(simpleRuntimeDeps); ok {
					info.RuntimeDeps = parsedDeps
				}

				infoMap[pkg.Name()] = info
				log.Print(packageName)

				return nil
			}(); err != nil {
				log.Printf("WARNING: %s: %v", packageName, err)
			}
		}

		encoder := json.NewEncoder(os.Stdout)
		encoder.SetIndent("", "  ")
		encoder.SetEscapeHTML(false)
		return encoder.Encode(infoMap)
	},
}

func main() {
	runfiles.FixEnv()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
