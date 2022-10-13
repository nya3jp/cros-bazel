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

	"github.com/alessio/shellescape"
	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"cros.local/bazel/ebuild/private/common/runfiles"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

const (
	ebuildExt = ".ebuild"
	binaryExt = ".tbz2"
)

//go:embed setup.sh
var setupScript []byte

func parseEBuildPath(path string) (packageShortName string, ver *version.Version, err error) {
	const versionSep = "-"

	if !strings.HasSuffix(path, ebuildExt) {
		return "", nil, errors.New("ebuild must have .ebuild suffix")
	}

	rest, ver, err := version.ExtractSuffix(strings.TrimSuffix(path, ebuildExt))
	if err != nil {
		return "", nil, err
	}
	if !strings.HasSuffix(rest, versionSep) {
		return "", nil, errors.New("package name and version must be separated by a hyphen")
	}
	packageShortName = filepath.Base(strings.TrimSuffix(rest, versionSep))
	return packageShortName, ver, nil
}

func baseNoExt(path string) string {
	return strings.SplitN(filepath.Base(path), ".", 2)[0]
}

func runCommand(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func copyFile(src, dst string) error {
	return runCommand("/usr/bin/cp", "--", src, dst)
}

var flagEBuild = &cli.StringFlag{
	Name:     "ebuild",
	Required: true,
}

var flagCategory = &cli.StringFlag{
	Name:     "category",
	Required: true,
}

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagSDK = &cli.StringSliceFlag{
	Name:     "sdk",
	Required: true,
}

var flagOverlay = &cli.StringSliceFlag{
	Name:     "overlay",
	Required: true,
}

var flagInstallTarget = &cli.StringSliceFlag{
	Name:  "install-target",
	Usage: "<binpkg>[,<binpkg>]+: All binpkgs specified will be installed in parallel",
}

var flagDistfile = &cli.StringSliceFlag{
	Name: "distfile",
}

var flagFile = &cli.StringSliceFlag{
	Name: "file",
}

var flagOutput = &cli.StringFlag{
	Name:     "output",
	Required: true,
}

var flagLogin = &cli.BoolFlag{
	Name: "login",
}

type overlayInfo struct {
	MountDir     string
	SquashfsPath string
}

func parseOverlaySpecs(specs []string) ([]overlayInfo, error) {
	var overlays []overlayInfo
	for _, spec := range specs {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid overlay spec: %s", spec)
		}
		overlays = append(overlays, overlayInfo{
			MountDir:     strings.Trim(v[0], "/"),
			SquashfsPath: v[1],
		})
	}
	return overlays, nil
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
		if err := copyFile(installPath, copyPath.Outside()); err != nil {
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

// The --sdk inputs can be two types:
//  1. A symlink to a directory or symlink to a file in the real execroot
//  2. A directory tree with symlinks pointing to the files in the real exec root
//
// This function undose the latter case. Bazel should be giving us a symlink to
// the directory, instead of creating a symlink tree. We don't want to use the
// symlink tree because that would require bind mounting the whole execroot
// inside the container. Otherwise we couldn't resolve the symlinks.
//
// This method will find the first symlink in the symlink forest which will be
// pointing to the real execroot. It then calculates the folder that should have
// been passed in by bazel.
func resolveSdkPaths(sdkPath string) (string, error) {
	log.Println("Sdk Path: ", sdkPath)
	info, err := os.Lstat(sdkPath)
	if err != nil {
		return "", err
	}

	log.Println("Mode:", info.Mode())

	if info.Mode()&fs.ModeSymlink != 0 {
		// Resolve the symlink so we always return an absolute path.
		pointsTo, err := os.Readlink(sdkPath)
		if err != nil {
			return "", err
		}
		return pointsTo, nil
	}

	if !info.IsDir() {
		return "", fmt.Errorf("Can't resolve sdk path %s %s", info.Mode(), sdkPath)
	}

	done := fmt.Errorf("Done")

	var resolvedSdkPath string
	err = filepath.WalkDir(sdkPath, func(path string, info fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		if info.IsDir() {
			return nil
		}

		fileInfo, err := info.Info()
		if err != nil {
			return err
		}

		if fileInfo.Mode().Type()&fs.ModeSymlink == 0 {
			return fmt.Errorf("Expected %s to be a symlink", path)
		}

		target, err := os.Readlink(path)
		if err != nil {
			return err
		}

		insidePath := strings.TrimPrefix(path, sdkPath)

		resolvedSdkPath = strings.TrimSuffix(target, insidePath)

		return done
	})

	if err != nil && err != done {
		return "", err
	}

	return resolvedSdkPath, nil
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagEBuild,
		flagCategory,
		flagBoard,
		flagSDK,
		flagOverlay,
		flagInstallTarget,
		flagDistfile,
		flagFile,
		flagOutput,
		flagLogin,
	},
	Action: func(c *cli.Context) error {
		originalEBuildPath := c.String(flagEBuild.Name)
		category := c.String(flagCategory.Name)
		board := c.String(flagBoard.Name)
		distfileSpecs := c.StringSlice(flagDistfile.Name)
		sdkPaths := c.StringSlice(flagSDK.Name)
		overlays, err := parseOverlaySpecs(c.StringSlice(flagOverlay.Name))
		if err != nil {
			return err
		}
		var targetInstallGroups [][]string
		for _, targetGroupStr := range c.StringSlice(flagInstallTarget.Name) {
			targets := strings.Split(targetGroupStr, ",")
			targetInstallGroups = append(targetInstallGroups, targets)
		}
		fileSpecs := c.StringSlice(flagFile.Name)
		finalOutPath := c.String(flagOutput.Name)
		login := c.Bool(flagLogin.Name)

		if !login {
			log.Print("HINT: To debug this build environment, run the Bazel build with --spawn_strategy=standalone, and run the command printed below:")
			pwd, _ := os.Getwd()
			log.Printf("( cd %s && %s --login )", shellescape.Quote(pwd), shellescape.QuoteCommand(os.Args))
		}

		runInContainerPath, ok := bazel.FindBinary("bazel/ebuild/private/cmd/run_in_container", "run_in_container")
		if !ok {
			return errors.New("run_in_container not found")
		}

		packageShortName, _, err := parseEBuildPath(originalEBuildPath)
		if err != nil {
			return fmt.Errorf("invalid ebuild file name: %w", err)
		}

		tmpDir, err := os.MkdirTemp("", "build_package.*")
		if err != nil {
			return err
		}
		defer os.RemoveAll(tmpDir)

		scratchDir := filepath.Join(tmpDir, "scratch")
		diffDir := filepath.Join(scratchDir, "diff")

		rootDir := fileutil.NewDualPath(filepath.Join(tmpDir, "root"), "/")
		bazelBuildDir := rootDir.Add("mnt/host/bazel-build")
		sourceDir := rootDir.Add("mnt/host/source")
		// TODO: Choose a right overlay.
		ebuildDir := sourceDir.Add(overlays[0].MountDir, category, packageShortName)
		distDir := rootDir.Add("var/cache/distfiles")
		hostPackagesDir := rootDir.Add("var/lib/portage/pkgs")
		targetPackagesDir := rootDir.Add("build").Add(board).Add("packages")

		for _, dir := range []fileutil.DualPath{bazelBuildDir, sourceDir, ebuildDir, distDir, hostPackagesDir, targetPackagesDir} {
			if err := os.MkdirAll(dir.Outside(), 0o755); err != nil {
				return err
			}
		}

		overlayEbuildPath := ebuildDir.Add(filepath.Base(originalEBuildPath))
		if err := copyFile(originalEBuildPath, overlayEbuildPath.Outside()); err != nil {
			return err
		}

		for _, fileSpec := range fileSpecs {
			v := strings.SplitN(fileSpec, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid file spec")
			}
			src := v[1]
			dst := ebuildDir.Add(v[0]).Outside()
			if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
				return err
			}
			if err := copyFile(src, dst); err != nil {
				return err
			}
		}

		for _, distfileSpec := range distfileSpecs {
			v := strings.SplitN(distfileSpec, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid distfile spec")
			}
			if err := copyFile(v[1], distDir.Add(v[0]).Outside()); err != nil {
				return err
			}
		}

		args := []string{
			runInContainerPath,
			"--scratch-dir=" + scratchDir,
			"--overlay=" + bazelBuildDir.Inside() + "=" + bazelBuildDir.Outside(),
			"--overlay=" + ebuildDir.Inside() + "=" + ebuildDir.Outside(),
			"--overlay=" + distDir.Inside() + "=" + distDir.Outside(),
			"--overlay=" + hostPackagesDir.Inside() + "=" + hostPackagesDir.Outside(),
			"--overlay=" + targetPackagesDir.Inside() + "=" + targetPackagesDir.Outside(),
		}

		for _, sdkPath := range sdkPaths {
			resolvedSdkPath, err := resolveSdkPaths(sdkPath)
			if err != nil {
				return err
			}
			args = append(args, "--overlay=/="+resolvedSdkPath)
		}

		for _, overlay := range overlays {
			overlayDir := sourceDir.Add(overlay.MountDir)
			args = append(args, "--overlay="+overlayDir.Inside()+"="+overlay.SquashfsPath)
		}

		targetInstallAtomGroups, err := preparePackageGroups(targetInstallGroups, targetPackagesDir)
		if err != nil {
			return err
		}

		setupPath := bazelBuildDir.Add("setup.sh")
		if err := os.WriteFile(setupPath.Outside(), setupScript, 0o755); err != nil {
			return err
		}

		args = append(args, setupPath.Inside())
		if !login {
			args = append(args, "fakeroot", "ebuild", "--skip-manifest", overlayEbuildPath.Inside(), "clean", "package")
		}
		cmd := exec.Command(args[0], args[1:]...)
		cmd.Env = append(
			os.Environ(),
			"PATH=/usr/sbin:/usr/bin:/sbin:/bin",
			"BOARD="+board,
		)
		for i, atomGroup := range targetInstallAtomGroups {
			installString := fmt.Sprintf("INSTALL_ATOMS_TARGET_%d=%s", i, strings.Join(atomGroup, " "))
			cmd.Env = append(cmd.Env, installString)
		}
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			if err, ok := err.(*exec.ExitError); ok {
				os.Exit(err.ExitCode())
			}
			return err
		}

		if !login {
			// TODO: Normalize timestamps in the archive.
			binaryOutPath := targetPackagesDir.Add(
				category,
				strings.TrimSuffix(filepath.Base(originalEBuildPath), ebuildExt)+binaryExt)
			if err := copyFile(filepath.Join(diffDir, binaryOutPath.Inside()), finalOutPath); err != nil {
				return err
			}
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
