// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/cliutil"
	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"cros.local/bazel/ebuild/private/common/processes"
	"cros.local/bazel/ebuild/private/common/standard/version"
	"github.com/bazelbuild/rules_go/go/runfiles"
	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/mountsdk"
)

const (
	ebuildExt = ".ebuild"
	binaryExt = ".tbz2"
)

const mainScript = "/mnt/host/bazel-build/build_package.sh"

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

var flagOutput = &cli.StringFlag{
	Name:     "output",
	Required: true,
}

var flagXpak = &cli.StringSliceFlag{
	Name: "xpak",
	Usage: "<XPAK key>=[?]<output file>: Write the XPAK key from the binpkg " +
		"to the specified file. If =? is used then an empty file is created if " +
		"XPAK key doesn't exist.",
}

var flagOutputFile = &cli.StringSliceFlag{
	Name: "output-file",
	Usage: "<inside path>=<outside path>: Extracts a file from the binpkg " +
		"and writes it to the outside path",
}

var flagSysrootFile = &cli.StringSliceFlag{
	Name: "sysroot-file",
	Usage: "<inside path>=<outside path>: Copies the outside file into the " +
		"sysroot.",
}

type sysrootFileSpec struct {
	SysrootPath string
	SrcPath     string
}

// Spec format: <inside path>=<outside path>
func parseSysrootFileSpecs(specs []string) ([]sysrootFileSpec, error) {
	var sysrootFiles []sysrootFileSpec
	for _, spec := range specs {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid sysroot spec: %s", spec)
		}
		if !filepath.IsAbs(v[0]) {
			return nil, fmt.Errorf("invalid sysroot spec: %s, %s must be absolute", spec, v[0])
		}
		sysrootFiles = append(sysrootFiles, sysrootFileSpec{
			SysrootPath: v[0],
			SrcPath:     v[1],
		})
	}
	return sysrootFiles, nil
}

func installSysrootFiles(sysrootFiles []sysrootFileSpec, sysroot string) error {
	for _, sysrootFileSpec := range sysrootFiles {
		// TODO: Maybe we can hard link or bindmount the files to save the copy
		// cost?
		dest := filepath.Join(sysroot, sysrootFileSpec.SysrootPath)
		destDir := filepath.Dir(dest)
		if err := os.MkdirAll(destDir, 0o755); err != nil {
			return err
		}

		if err := fileutil.Copy(sysrootFileSpec.SrcPath, dest); err != nil {
			return fmt.Errorf("Failed to copy %s to %s: %w", sysrootFileSpec.SrcPath, dest, err)
		}
	}

	return nil
}

func extractBinaryPackageFiles(binPkgPath string,
	xpakSpecs []binarypackage.XpakSpec,
	outputFileSpecs []binarypackage.OutputFileSpec) error {
	if len(xpakSpecs) == 0 && len(outputFileSpecs) == 0 {
		return nil
	}

	binPkg, err := binarypackage.BinaryPackage(binPkgPath)
	if err != nil {
		return err
	}
	defer binPkg.Close()

	if err = binarypackage.ExtractXpakFiles(binPkg, xpakSpecs); err != nil {
		return err
	}

	if err = binarypackage.ExtractOutFiles(binPkg, outputFileSpecs); err != nil {
		return err
	}

	return nil
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

var app = &cli.App{
	Flags: append(mountsdk.CLIFlags,
		flagBoard,
		flagEBuild,
		flagFile,
		flagDistfile,
		flagOutput,
		flagOutputFile,
		flagXpak,
		flagSysrootFile,
	),
	Action: func(c *cli.Context) error {
		finalOutPath := c.String(flagOutput.Name)
		board := c.String(flagBoard.Name)
		ebuildSource := c.String(flagEBuild.Name)
		fileSpecs := c.StringSlice(flagFile.Name)
		distfileSpecs := c.StringSlice(flagDistfile.Name)

		xpakSpecs, err := binarypackage.ParseXpakSpecs(c.StringSlice(flagXpak.Name))
		if err != nil {
			return err
		}

		outputFileSpecs, err := binarypackage.ParseOutputFileSpecs(c.StringSlice(flagOutputFile.Name))
		if err != nil {
			return err
		}

		sysrtootFileSpecs, err := parseSysrootFileSpecs(c.StringSlice(flagSysrootFile.Name))
		if err != nil {
			return err
		}

		// We need "supports-graceful-termination" execution requirement in the
		// build action to let Bazel send SIGTERM instead of SIGKILL.
		ctx, cancel := signal.NotifyContext(c.Context, unix.SIGINT, unix.SIGTERM)
		defer cancel()
		c.Context = ctx

		cfg, err := mountsdk.GetMountConfigFromCLI(c)
		if err != nil {
			return err
		}

		runScript, err := runfiles.Rlocation("chromiumos/bazel/ebuild/private/cmd/build_package/build_package.sh")
		if err != nil {
			return err
		}
		cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
			Source:    runScript,
			MountPath: mainScript,
		})

		v := strings.SplitN(ebuildSource, "=", 2)
		if len(v) < 2 {
			return errors.New("invalid ebuild cfg")
		}

		ebuildMetadata, err := ParseEbuildMetadata(v[0])
		if err != nil {
			return fmt.Errorf("invalid ebuild file name: %w", err)
		}

		ebuildFile := makechroot.BindMount{
			Source:    v[1],
			MountPath: filepath.Join(mountsdk.SourceDir, "src", ebuildMetadata.Overlay, ebuildMetadata.Category, ebuildMetadata.PackageName, filepath.Base(ebuildSource)),
		}
		cfg.Remounts = append(cfg.Remounts, filepath.Dir(ebuildFile.MountPath))

		cfg.BindMounts = append(cfg.BindMounts, ebuildFile)
		for _, fileSpec := range fileSpecs {
			v := strings.SplitN(fileSpec, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid file cfg")
			}
			cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
				Source:    v[1],
				MountPath: filepath.Join(filepath.Dir(ebuildFile.MountPath), v[0]),
			})
		}

		for _, distfileSpec := range distfileSpecs {
			v := strings.SplitN(distfileSpec, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid distfile cfg")
			}
			cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
				Source:    v[1],
				MountPath: filepath.Join("/var/cache/distfiles", v[0]),
			})
		}

		targetPackagesDir := filepath.Join("/build", board, "packages")

		if err := mountsdk.RunInSDK(cfg, func(s *mountsdk.MountedSDK) error {
			overlayEbuildPath := s.RootDir.Add(ebuildFile.MountPath)
			for _, dir := range []string{targetPackagesDir, "/var/lib/portage/pkgs"} {
				if err := os.MkdirAll(s.RootDir.Add(dir).Outside(), 0o755); err != nil {
					return err
				}
			}

			sysroot := s.RootDir.Add("build").Add(board).Outside()

			if err := installSysrootFiles(sysrtootFileSpecs, sysroot); err != nil {
				return err
			}

			cmd := s.Command(mainScript, "ebuild", "--skip-manifest", overlayEbuildPath.Inside(), "clean", "package")
			cmd.Env = append(cmd.Env, fmt.Sprintf("BOARD=%s", board))

			if err := processes.Run(ctx, cmd); err != nil {
				return err
			}

			// TODO: Normalize timestamps in the archive.
			binaryOutPath := filepath.Join(targetPackagesDir,
				ebuildMetadata.Category,
				strings.TrimSuffix(filepath.Base(ebuildSource), ebuildExt)+binaryExt)

			if err := fileutil.Copy(filepath.Join(s.DiffDir, binaryOutPath), finalOutPath); err != nil {
				return err
			}

			if err := extractBinaryPackageFiles(finalOutPath, xpakSpecs, outputFileSpecs); err != nil {
				return err
			}

			return nil
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
