// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	_ "embed"
	"errors"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/runfiles"
)

//go:embed setup.sh
var setupScript []byte

func runCommand(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

var flagInputSquashfs = &cli.StringSliceFlag{
	Name:     "input-squashfs",
	Required: true,
}

var flagOutputSquashfs = &cli.StringFlag{
	Name:     "output-squashfs",
	Required: true,
}

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagOverlay = &cli.StringSliceFlag{
	Name:     "overlay",
	Required: true,
}

var flagInstallHost = &cli.StringSliceFlag{
	Name: "install-host",
}

var flagInstallTarget = &cli.StringSliceFlag{
	Name: "install-target",
}

var flagInstallTarball = &cli.StringSliceFlag{
	Name: "install-tarball",
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagInputSquashfs,
		flagOutputSquashfs,
		flagBoard,
		flagOverlay,
		flagInstallHost,
		flagInstallTarget,
		flagInstallTarball,
	},
	Action: func(c *cli.Context) error {
		inputSquashfsPaths := c.StringSlice(flagInputSquashfs.Name)
		outputSquashfsPath := c.String(flagOutputSquashfs.Name)
		board := c.String(flagBoard.Name)
		overlays, err := makechroot.ParseOverlaySpecs(c.StringSlice(flagOverlay.Name))
		if err != nil {
			return err
		}
		hostInstallPaths := c.StringSlice(flagInstallHost.Name)
		targetInstallPaths := c.StringSlice(flagInstallTarget.Name)
		tarballPaths := c.StringSlice(flagInstallTarball.Name)

		runInContainerPath, ok := bazel.FindBinary("bazel/ebuild/private/cmd/run_in_container", "run_in_container")
		if !ok {
			return errors.New("run_in_container not found")
		}

		tmpDir, err := os.MkdirTemp("", "build_sdk.*")
		if err != nil {
			return err
		}
		defer os.RemoveAll(tmpDir)

		diffDir := filepath.Join(tmpDir, "diff")
		workDir := filepath.Join(tmpDir, "work")

		rootDir := fileutil.NewDualPath(filepath.Join(tmpDir, "root"), "/")
		sysrootDir := rootDir.Add("build", board)
		sourceDir := rootDir.Add("mnt/host/source")
		stageDir := rootDir.Add("stage")
		tarballsDir := stageDir.Add("tarballs")
		hostPackagesDir := rootDir.Add("var/lib/portage/pkgs")
		targetPackagesDir := sysrootDir.Add("packages")

		for _, dir := range []string{diffDir, workDir, stageDir.Outside(), tarballsDir.Outside(), hostPackagesDir.Outside(), targetPackagesDir.Outside()} {
			if err := os.MkdirAll(dir, 0o755); err != nil {
				return err
			}
		}

		hostInstallAtoms, err := makechroot.CopyBinaryPackages(hostPackagesDir.Outside(), hostInstallPaths)
		if err != nil {
			return err
		}

		targetInstallAtoms, err := makechroot.CopyBinaryPackages(targetPackagesDir.Outside(), targetInstallPaths)
		if err != nil {
			return err
		}

		for _, path := range tarballPaths {
			if err := fileutil.Copy(path, tarballsDir.Add(filepath.Base(path)).Outside()); err != nil {
				return err
			}
		}

		scriptPath := stageDir.Add("setup.sh")
		if err := os.WriteFile(scriptPath.Outside(), setupScript, 0o755); err != nil {
			return err
		}

		args := []string{
			runInContainerPath,
			"--diff-dir=" + diffDir,
			"--work-dir=" + workDir,
			"--overlay-dir=" + stageDir.Inside() + "=" + stageDir.Outside(),
			"--overlay-dir=" + hostPackagesDir.Inside() + "=" + hostPackagesDir.Outside(),
			"--overlay-dir=" + targetPackagesDir.Inside() + "=" + targetPackagesDir.Outside(),
		}

		for _, inputSquashfsPath := range inputSquashfsPaths {
			args = append(args, "--overlay-squashfs=/="+inputSquashfsPath)
		}

		for _, overlay := range overlays {
			overlayDir := sourceDir.Add(overlay.MountDir)
			args = append(args, "--overlay-squashfs="+overlayDir.Inside()+"="+overlay.SquashfsPath)
		}

		args = append(args, scriptPath.Inside())

		cmd := exec.Command(args[0], args[1:]...)
		cmd.Env = append(
			os.Environ(),
			"PATH=/usr/sbin:/usr/bin:/sbin:/bin",
			"BOARD="+board,
			"INSTALL_ATOMS_HOST="+strings.Join(hostInstallAtoms, " "),
			"INSTALL_ATOMS_TARGET="+strings.Join(targetInstallAtoms, " "),
		)
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			return err
		}

		args = []string{
			"/usr/bin/mksquashfs",
			diffDir,
			outputSquashfsPath,
			"-all-time",
			"0",
			// TODO: Avoid -all-root.
			"-all-root",
		}
		for _, exclude := range []string{
			strings.TrimLeft(hostPackagesDir.Inside(), "/"),
			strings.TrimLeft(targetPackagesDir.Inside(), "/"),
			"mnt",
			"stage",
			"tmp",
			"var/tmp",
			"var/log",
			"var/cache",
		} {
			args = append(args, "-e", exclude)
		}
		if err := runCommand(args[0], args[1:]...); err != nil {
			return err
		}
		return nil
	},
}

func main() {
	runfiles.FixEnv()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
