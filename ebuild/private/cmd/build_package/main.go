// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	_ "embed"
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/alessio/shellescape"
	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/urfave/cli/v2"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/cliutil"
	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

const (
	ebuildExt = ".ebuild"
	binaryExt = ".tbz2"
)

//go:embed setup.sh
var setupScript []byte

type ebuildPathInfo struct {
	overlay     string
	category    string
	packageName string
	version     *version.Version
}

// Expects path to be in the following form:
// <overlay>/<category>/<packageName>/<packageName>-<version>.ebuild
// i.e., third_party/chromiumos-overlay/app-accessibility/brltty/brltty-6.3-r6.ebuild
func parseEBuildPath(path string) (*ebuildPathInfo, error) {
	if !strings.HasSuffix(path, ebuildExt) {
		return nil, errors.New("ebuild must have .ebuild suffix")
	}

	rest, ver, err := version.ExtractSuffix(strings.TrimSuffix(path, ebuildExt))
	if err != nil {
		return nil, err
	}

	info := ebuildPathInfo{
		version: ver,
	}

	parts := strings.Split(rest, string(os.PathSeparator))

	if len(parts) < 4 {
		return nil, fmt.Errorf("Unable to parse ebuild path: %s", path)
	}

	info.packageName = parts[len(parts)-2]
	info.category = parts[len(parts)-3]
	info.overlay = filepath.Join(parts[0 : len(parts)-3]...)

	return &info, nil
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
	Usage: "<inside path>=<squashfs file | directory | tar.*>: " +
		"Mounts the file or directory at the specified path. " +
		"Inside path can be absolute or relative to /mnt/host/source/.",
}

var flagInstallTarget = &cli.StringSliceFlag{
	Name:  "install-target",
	Usage: "<binpkg>[:<binpkg>]+: All binpkgs specified will be installed in parallel",
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
			MountDir:     strings.TrimSuffix(v[0], "/"),
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

var app = &cli.App{
	Flags: []cli.Flag{
		flagEBuild,
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
		board := c.String(flagBoard.Name)
		distfileSpecs := c.StringSlice(flagDistfile.Name)
		sdkPaths := c.StringSlice(flagSDK.Name)
		overlays, err := parseOverlaySpecs(c.StringSlice(flagOverlay.Name))
		if err != nil {
			return err
		}
		var targetInstallGroups [][]string
		for _, targetGroupStr := range c.StringSlice(flagInstallTarget.Name) {
			targets := strings.Split(targetGroupStr, ":")
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

		ebuildInfo, err := parseEBuildPath(originalEBuildPath)
		if err != nil {
			return fmt.Errorf("invalid ebuild file name: %w", err)
		}

		tmpDir, err := os.MkdirTemp(".", "build_package.*")
		if err != nil {
			return err
		}
		defer os.RemoveAll(tmpDir)

		scratchDir := filepath.Join(tmpDir, "scratch")
		diffDir := filepath.Join(scratchDir, "diff")

		rootDir := fileutil.NewDualPath(filepath.Join(tmpDir, "root"), "/")
		bazelBuildDir := rootDir.Add("mnt/host/bazel-build")
		sourceDir := rootDir.Add("mnt/host/source")
		ebuildDir := sourceDir.Add("src", ebuildInfo.overlay, ebuildInfo.category, ebuildInfo.packageName)
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
			"--overlay=/=" + rootDir.Outside(),
			// Even though the ebuildDir is inside the rootDir we need to explicitly
			// pass it in because the ebuild overlay squashfs gets mounted on top.
			// If we converted the `overlay` rule to generate using the full root path
			// in the squashfs files we could mount them at / and remove this
			// line.
			"--overlay=" + ebuildDir.Inside() + "=" + ebuildDir.Outside(),
		}

		for _, sdkPath := range sdkPaths {
			args = append(args, "--overlay=/="+sdkPath)
		}

		for _, overlay := range overlays {
			var overlayDir string
			if filepath.IsAbs(overlay.MountDir) {
				overlayDir = overlay.MountDir
			} else {
				overlayDir = sourceDir.Add(overlay.MountDir).Inside()
			}
			args = append(args, "--overlay="+overlayDir+"="+overlay.SquashfsPath)
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
			args = append(args, "ebuild", "--skip-manifest", overlayEbuildPath.Inside(), "clean", "package")
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
				return cliutil.ExitCode(err.ExitCode())
			}
			return err
		}

		if !login {
			// TODO: Normalize timestamps in the archive.
			binaryOutPath := targetPackagesDir.Add(
				ebuildInfo.category,
				strings.TrimSuffix(filepath.Base(originalEBuildPath), ebuildExt)+binaryExt)
			if err := copyFile(filepath.Join(diffDir, binaryOutPath.Inside()), finalOutPath); err != nil {
				return err
			}
		}

		return nil
	},
}

func main() {
	bazelutil.FixRunfilesEnv()
	cliutil.Exit(app.Run(os.Args))
}
