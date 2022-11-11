// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	_ "embed"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/cliutil"
	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"cros.local/bazel/ebuild/private/common/standard/version"
	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/mountsdk"
)

const (
	ebuildExt = ".ebuild"
	binaryExt = ".tbz2"
)

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagEBuild = &cli.StringFlag{
	Name:     "ebuild",
	Required: true,
}

var flagFile = &cli.StringSliceFlag{
	Name: "file",
}

var flagDistfile = &cli.StringSliceFlag{
	Name: "distfile",
}

var flagInstallTarget = &cli.StringSliceFlag{
	Name:  "install-target",
	Usage: "<binpkg>[:<binpkg>]+: All binpkgs specified will be installed in parallel",
}

var flagOutput = &cli.StringFlag{
	Name:     "output",
	Required: true,
}

type EbuildMetadata struct {
	Overlay     string
	Category    string
	PackageName string
	Version     *version.Version
}

// ParseEbuildMetadata expects path to be in the following form:
// <overlay>/<category>/<packageName>/<packageName>-<version>.ebuild
// i.e., third_party/chromiumos-overlay/app-accessibility/brltty/brltty-6.3-r6.ebuild
// TODO: this currently fails with absolute paths.
func ParseEbuildMetadata(path string) (*EbuildMetadata, error) {
	if !strings.HasSuffix(path, ebuildExt) {
		return nil, errors.New("ebuild must have .ebuild suffix")
	}

	rest, ver, err := version.ExtractSuffix(strings.TrimSuffix(path, ebuildExt))
	if err != nil {
		return nil, err
	}

	info := EbuildMetadata{
		Version: ver,
	}

	parts := strings.Split(rest, string(os.PathSeparator))

	if len(parts) < 4 {
		return nil, fmt.Errorf("unable to parse ebuild path: %s", path)
	}

	info.PackageName = parts[len(parts)-2]
	info.Category = parts[len(parts)-3]
	info.Overlay = filepath.Join(parts[0 : len(parts)-3]...)

	return &info, nil
}

func preparePackages(installPaths []string, dir fileutil.DualPath) ([]string, error) {
	var atoms []string

	for _, installPath := range installPaths {
		xp, err := binarypackage.ReadXpak(installPath)
		if err != nil {
			return nil, fmt.Errorf("reading %s: %w", filepath.Base(installPath), err)
		}
		category := strings.TrimSpace(string(xp["CATEGORY"]))
		pf := strings.TrimSpace(string(xp["PF"]))

		categoryDir := dir.Add(category)
		if err := os.MkdirAll(categoryDir.Outside(), 0o755); err != nil {
			return nil, err
		}

		copyPath := categoryDir.Add(pf + binaryExt)
		if err := fileutil.Copy(installPath, copyPath.Outside()); err != nil {
			return nil, err
		}

		atoms = append(atoms, fmt.Sprintf("=%s/%s", category, pf))
	}

	return atoms, nil
}

func preparePackageGroups(installGroups [][]string, dir fileutil.DualPath) ([][]string, error) {
	var atomGroups [][]string

	for _, installGroup := range installGroups {
		atoms, err := preparePackages(installGroup, dir)
		if err != nil {
			return nil, err
		}
		atomGroups = append(atomGroups, atoms)
	}

	return atomGroups, nil
}

var app = &cli.App{
	Flags: append(mountsdk.CLIFlags,
		flagBoard,
		flagEBuild,
		flagFile,
		flagDistfile,
		flagInstallTarget,
		flagOutput,
	),
	Action: func(c *cli.Context) error {
		finalOutPath := c.String(flagOutput.Name)
		board := c.String(flagBoard.Name)
		ebuildHostPath := c.String(flagEBuild.Name)
		fileSpecs := c.StringSlice(flagFile.Name)
		distfileSpecs := c.StringSlice(flagDistfile.Name)
		installTargetsUnparsed := c.StringSlice(flagInstallTarget.Name)

		// We need "supports-graceful-termination" execution requirement in the
		// build action to let Bazel send SIGTERM instead of SIGKILL.
		ctx, cancel := signal.NotifyContext(c.Context, unix.SIGINT, unix.SIGTERM)
		defer cancel()
		c.Context = ctx

		cfg, err := mountsdk.GetMountConfigFromCLI(c)
		if err != nil {
			return err
		}

		ebuildMetadata, err := ParseEbuildMetadata(ebuildHostPath)
		if err != nil {
			return fmt.Errorf("invalid ebuild file name: %w", err)
		}

		ebuildFile := mountsdk.MappedDualPath{
			HostPath: ebuildHostPath,
			SDKPath:  filepath.Join(mountsdk.SourceDir, "src", ebuildMetadata.Overlay, ebuildMetadata.Category, ebuildMetadata.PackageName, filepath.Base(ebuildHostPath)),
		}
		cfg.Remounts = append(cfg.Remounts, filepath.Dir(ebuildFile.SDKPath))

		cfg.CopyToSDK = append(cfg.CopyToSDK, ebuildFile)
		for _, fileSpec := range fileSpecs {
			v := strings.SplitN(fileSpec, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid file cfg")
			}
			cfg.CopyToSDK = append(cfg.CopyToSDK, mountsdk.MappedDualPath{
				HostPath: v[1],
				SDKPath:  filepath.Join(filepath.Dir(ebuildFile.SDKPath), v[0]),
			})
		}

		for _, distfileSpec := range distfileSpecs {
			v := strings.SplitN(distfileSpec, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid distfile cfg")
			}
			cfg.CopyToSDK = append(cfg.CopyToSDK, mountsdk.MappedDualPath{
				HostPath: v[1],
				SDKPath:  filepath.Join("/var/cache/distfiles", v[0]),
			})
		}

		var targetInstallGroups [][]string
		for _, targetGroupStr := range installTargetsUnparsed {
			targets := strings.Split(targetGroupStr, ":")
			targetInstallGroups = append(targetInstallGroups, targets)
		}

		if err := mountsdk.RunInSDK(cfg, func(s *mountsdk.MountedSDK) error {
			overlayEbuildPath := s.RootDir.Add(ebuildFile.SDKPath)
			targetPackagesDir := s.RootDir.Add("build").Add(board).Add("packages")
			hostPackagesDir := s.RootDir.Add("var/lib/portage/pkgs")
			for _, dir := range []fileutil.DualPath{targetPackagesDir, hostPackagesDir} {
				if err := os.MkdirAll(dir.Outside(), 0o755); err != nil {
					return err
				}
			}

			targetInstallAtomGroups, err := preparePackageGroups(targetInstallGroups, targetPackagesDir)
			if err != nil {
				return err
			}

			cmd := s.Command(ctx, "ebuild", "--skip-manifest", overlayEbuildPath.Inside(), "clean", "package")
			cmd.Env = append(cmd.Env, fmt.Sprintf("BOARD=%s", board))

			for i, atomGroup := range targetInstallAtomGroups {
				cmd.Env = append(cmd.Env,
					fmt.Sprintf("INSTALL_ATOMS_TARGET_%d=%s", i,
						strings.Join(atomGroup, " ")))
			}

			if err := cmd.Run(); err != nil {
				return err
			}

			// TODO: Normalize timestamps in the archive.
			binaryOutPath := targetPackagesDir.Add(
				ebuildMetadata.Category,
				strings.TrimSuffix(filepath.Base(ebuildHostPath), ebuildExt)+binaryExt)

			return fileutil.Copy(filepath.Join(s.DiffDir, binaryOutPath.Inside()), finalOutPath)
		}); err != nil {
			if err, ok := err.(*exec.ExitError); ok {
				return cliutil.ExitCode(err.ExitCode())
			}
			return err
		}

		return nil
	},
}

func main() {
	bazelutil.FixRunfilesEnv()

	cliutil.Exit(app.Run(os.Args))
}
