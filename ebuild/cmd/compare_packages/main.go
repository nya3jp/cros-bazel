// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"errors"
	"fmt"
	"log"
	"os"
	"path/filepath"

	"github.com/google/go-cmp/cmp"
	"github.com/urfave/cli/v2"

	"cros.local/bazel/ebuild/cmd/compare_packages/equery"
	"cros.local/bazel/ebuild/cmd/compare_packages/golden"
	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/portage"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

func regenerateGolden(workspaceDir string, board string, goldenPath string) error {
	client := equery.NewClient(workspaceDir, board)

	pkgs, err := client.ListInstalledPackages()
	if err != nil {
		return err
	}

	var infos []*golden.Package
	for _, pkg := range pkgs {
		infos = append(infos, &golden.Package{
			Name:    pkg.Name,
			Version: pkg.Version.String(),
			Uses:    pkg.Uses,
		})
	}

	if err := golden.Save(goldenPath, infos); err != nil {
		return err
	}
	return nil
}

func compareWithPortage(board string, goldenPath string) error {
	wants, err := golden.Load(goldenPath)
	if err != nil {
		return err
	}

	resolver, err := portage.NewResolver(filepath.Join("/build", board), portage.NewHackSource())
	if err != nil {
		return err
	}

	for _, want := range wants {
		ver, err := version.Parse(want.Version)
		if err != nil {
			return err
		}

		pkgs, err := resolver.Packages(dependency.NewAtom(want.Name, dependency.OpExactEqual, ver, false, "", nil))
		if err != nil {
			return err
		}
		if len(pkgs) == 0 {
			return fmt.Errorf("no package found for %s-%s", want.Name, want.Version)
		}
		if len(pkgs) >= 2 {
			return fmt.Errorf("multiple packages found for %s-%s", want.Name, want.Version)
		}
		pkg := pkgs[0]

		pkgUses := pkg.Uses()
		gotUses := make(map[string]bool)
		for use := range want.Uses {
			gotUses[use] = pkgUses[use]
		}

		if diff := cmp.Diff(gotUses, want.Uses); diff != "" {
			fmt.Printf("%s-%s:\n%s", want.Name, want.Version, diff)
		}
	}
	return nil
}

var flagBoard = &cli.StringFlag{
	Name:  "board",
	Value: "arm64-generic",
}

var flagRegenerateGolden = &cli.BoolFlag{
	Name: "regenerate-golden",
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagBoard,
		flagRegenerateGolden,
	},
	Before: func(ctx *cli.Context) error {
		if _, err := os.Stat("/etc/cros_chroot_version"); err != nil {
			return errors.New("this program must be run in chroot")
		}
		return nil
	},
	Action: func(c *cli.Context) error {
		board := c.String(flagBoard.Name)
		regenGolden := c.Bool(flagRegenerateGolden.Name)

		workspaceDir := bazelutil.WorkspaceDir()
		goldenPath := filepath.Join(workspaceDir, "bazel", "data", "packages.golden.json")

		if regenGolden {
			log.Print("Regenerating golden data. This will take quite a long time...")
			if err := regenerateGolden(workspaceDir, board, goldenPath); err != nil {
				return err
			}
		}

		return compareWithPortage(board, goldenPath)
	},
}

func main() {
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
