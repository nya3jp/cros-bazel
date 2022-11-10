// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"context"
	_ "embed"
	"errors"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strings"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/cliutil"
	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/symindex"
)

//go:embed setup.sh
var setupScript []byte

var flagInput = &cli.StringSliceFlag{
	Name:     "input",
	Usage:    "A path to a file representing a file system layer",
	Required: true,
}

var flagOutputDir = &cli.StringFlag{
	Name:     "output-dir",
	Usage:    "A path to a directory to write non-symlink files under",
	Required: true,
}

var flagOutputSymindex = &cli.StringFlag{
	Name:     "output-symindex",
	Usage:    "A path to write a symindex file to",
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
		flagInput,
		flagOutputDir,
		flagOutputSymindex,
		flagBoard,
		flagOverlay,
		flagInstallHost,
		flagInstallTarget,
		flagInstallTarball,
	},
	Action: func(c *cli.Context) error {
		// We need "supports-graceful-termination" execution requirement in the
		// build action to let Bazel send SIGTERM instead of SIGKILL.
		ctx, cancel := signal.NotifyContext(context.Background(), unix.SIGINT, unix.SIGTERM)
		defer cancel()

		inputPaths := c.StringSlice(flagInput.Name)
		outputDirPath := c.String(flagOutputDir.Name)
		outputSymindexPath := c.String(flagOutputSymindex.Name)
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

		// We use the output directory as our tmp space. This way we can avoid copying
		// the output artifacts and instead just move them.
		tmpDir, err := os.MkdirTemp(outputDirPath, "sdk_update.*")
		if err != nil {
			return err
		}

		scratchDir := filepath.Join(tmpDir, "scratch")
		diffDir := filepath.Join(scratchDir, "diff")

		rootDir := fileutil.NewDualPath(filepath.Join(tmpDir, "root"), "/")
		sysrootDir := rootDir.Add("build", board)
		sourceDir := rootDir.Add("mnt/host/source")
		stageDir := rootDir.Add("stage")
		tarballsDir := stageDir.Add("tarballs")
		hostPackagesDir := rootDir.Add("var/lib/portage/pkgs")
		targetPackagesDir := sysrootDir.Add("packages")

		for _, dir := range []string{stageDir.Outside(), tarballsDir.Outside(), hostPackagesDir.Outside(), targetPackagesDir.Outside()} {
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
			"--scratch-dir=" + scratchDir,
			"--overlay=" + stageDir.Inside() + "=" + stageDir.Outside(),
			"--overlay=" + hostPackagesDir.Inside() + "=" + hostPackagesDir.Outside(),
			"--overlay=" + targetPackagesDir.Inside() + "=" + targetPackagesDir.Outside(),
		}

		for _, inputPath := range inputPaths {
			args = append(args, "--overlay=/="+inputPath)
		}

		for _, overlay := range overlays {
			overlayDir := sourceDir.Add(overlay.MountDir)
			args = append(args, "--overlay="+overlayDir.Inside()+"="+overlay.SquashfsPath)
		}

		args = append(args, scriptPath.Inside())

		cmd := exec.CommandContext(ctx, args[0], args[1:]...)
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

		if err := moveDiffDir(diffDir, outputDirPath); err != nil {
			return err
		}

		// Some of the folders in the overlayfs workdir have 000 permissions.
		// We need to grant rw permissions to the directories so `os.RemoveAll`
		// doesn't fail.
		if err := fileutil.RemoveAllWithChmod(tmpDir); err != nil {
			return err
		}

		for _, exclude := range []string{
			filepath.Join("build", board, "tmp"),
			filepath.Join("build", board, "var/cache"),
			hostPackagesDir.Inside(),
			targetPackagesDir.Inside(),
			"run",
			"stage",
			"tmp",
			"var/tmp",
			"var/log",
			"var/cache",
		} {
			path := filepath.Join(outputDirPath, exclude)
			if err := fileutil.RemoveAllWithChmod(path); err != nil {
				return err
			}
		}

		if err := symindex.Generate(outputDirPath, outputSymindexPath); err != nil {
			return err
		}

		return nil
	},
}

// Move the contents of the diff dir into the output path
func moveDiffDir(diffDir string, outputPath string) error {
	es, err := os.ReadDir(diffDir)
	if err != nil {
		return err
	}

	for _, e := range es {
		src := filepath.Join(diffDir, e.Name())
		dest := filepath.Join(outputPath, e.Name())

		if e.IsDir() {
			// For directories, we need o+w (S_IWUSR) permission to rename.
			fi, err := e.Info()
			if err != nil {
				return err
			}
			if perm := fi.Mode().Perm(); perm&unix.S_IWUSR == 0 {
				if err := os.Chmod(src, perm|unix.S_IWUSR); err != nil {
					return err
				}
			}
		}

		if err := os.Rename(src, dest); err != nil {
			return err
		}
	}

	return nil
}

func main() {
	bazelutil.FixRunfilesEnv()
	cliutil.Exit(app.Run(os.Args))
}
