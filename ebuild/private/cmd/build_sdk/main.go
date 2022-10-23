// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	_ "embed"
	"errors"
	"fmt"
	"io/fs"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/urfave/cli/v2"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/makechroot"
)

//go:embed setup.sh
var setupScript []byte

var flagInput = &cli.StringSliceFlag{
	Name:     "input",
	Required: true,
}

var flagOutputDir = &cli.StringFlag{
	Name:     "output-dir",
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
		flagInput,
		flagOutputDir,
		flagOutputSquashfs,
		flagBoard,
		flagOverlay,
		flagInstallHost,
		flagInstallTarget,
		flagInstallTarball,
	},
	Action: func(c *cli.Context) error {
		inputPaths := c.StringSlice(flagInput.Name)
		outputDirPath := c.String(flagOutputDir.Name)
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

		// We use the output directory as our tmp space. This way we can avoid copying
		// the output artifacts and instead just move them.
		tmpDir, err := os.MkdirTemp(outputDirPath, "build_sdk.*")
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

		if err := moveDiffDir(diffDir, outputDirPath); err != nil {
			return err
		}

		// Some of the folders in the overlayfs workdir have 000 permissions.
		// We need to grant rw permissions to the directories so `os.RemoveAll`
		// doesn't fail.
		if err := fixChmod(tmpDir); err != nil {
			return err
		}

		if err := os.RemoveAll(tmpDir); err != nil {
			return err
		}

		for _, exclude := range []string{
			filepath.Join("build", board, "tmp"),
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
			os.RemoveAll(path)
		}

		if err := moveSymlinks(outputDirPath, outputSquashfsPath); err != nil {
			return err
		}

		return nil
	},
}

// Move the contents of the diff dir into the output path
func moveDiffDir(diffDir string, outputPath string) error {
	diffDirHandle, err := os.Open(diffDir)
	if err != nil {
		return err
	}
	defer diffDirHandle.Close()

	filesInfo, err := diffDirHandle.Readdir(0)
	if err != nil {
		return err
	}

	for _, fileInfo := range filesInfo {
		src := filepath.Join(diffDir, fileInfo.Name())
		dest := filepath.Join(outputPath, fileInfo.Name())
		if err := os.Rename(src, dest); err != nil {
			return err
		}
	}

	return nil
}

// Ensure we have o+rwx to each directory. Otherwise os.RemoveAll() fails.
func fixChmod(path string) error {
	err := filepath.WalkDir(path, func(path string, info fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() {
			return nil
		}

		fileInfo, err := info.Info()
		if err != nil {
			return err
		}

		if fileInfo.Mode().Perm()&0700 == 0700 {
			return nil
		}

		if err := os.Chmod(path, 0700); err != nil {
			return err
		}

		return nil
	})

	return err
}

// Until https://github.com/bazelbuild/bazel/issues/15454 is fixed we
// cant use absolute paths in symlinks or symlinks that point to the base layer.
// As a work around we extract all the absolute and dangling symlinks into
// their own layer.
func moveSymlinks(outputPath string, squashfsPath string) error {
	var symlinks []string

	err := filepath.Walk(outputPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if info.Mode()&fs.ModeSymlink == 0 {
			return nil
		}

		pointsTo, err := os.Readlink(path)
		if err != nil {
			return err
		}

		if strings.HasPrefix(pointsTo, "/") {
			symlinks = append(symlinks, path)
			return nil
		}

		parentDir := filepath.Dir(path)
		absolutePath := filepath.Join(parentDir, pointsTo)

		if !strings.HasPrefix(absolutePath, outputPath) {
			return fmt.Errorf("Symlink %s points to %s which is below the output base %s", path, pointsTo, outputPath)
		}

		if _, err := os.Stat(absolutePath); err == nil {
			// Symlink points to a file on this layer, leave it
			return nil
		} else if errors.Is(err, fs.ErrNotExist) {
			// Symlink is pointing to something in the base layer
			symlinks = append(symlinks, path)
			return nil
		} else {
			// Some unknown stat error
			return err
		}

		return nil
	})

	args := []string{"/usr/bin/mksquashfs"}
	for _, symlink := range symlinks {
		args = append(args, strings.TrimPrefix(symlink, outputPath+"/"))
	}
	absoluteSquashfsPath, err := filepath.Abs(squashfsPath)
	if err != nil {
		return err
	}
	args = append(args,
		absoluteSquashfsPath,
		"-all-time",
		"0",
		// TODO: Avoid -all-root.
		"-all-root",
		"-no-strip",
	)

	cmd := exec.Command(args[0], args[1:]...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Dir = outputPath

	if err := cmd.Run(); err != nil {
		return err
	}

	// Clear the symlinks from the output directory
	for _, symlink := range symlinks {
		if err := os.Remove(symlink); err != nil {
			return err
		}
	}

	return err
}

func main() {
	bazelutil.FixRunfilesEnv()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
