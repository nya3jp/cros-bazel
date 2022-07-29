package main

import (
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"os/user"
	"path/filepath"
	"strings"

	"github.com/urfave/cli"

	"cros.local/ebuild/private/go/portage/version"
)

const (
	ebuildExt = ".ebuild"
	binaryExt = ".tbz2"
)

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

func copyFile(src, dst string) error {
	cmd := exec.Command("/usr/bin/cp", "--", src, dst)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func main() {
	flagEBuild := &cli.StringFlag{Name: "ebuild", Required: true}
	flagCategory := &cli.StringFlag{Name: "category", Required: true}
	flagDistfiles := &cli.StringSliceFlag{Name: "distfile"}
	flagOutput := &cli.StringFlag{Name: "output", Required: true}

	app := &cli.App{
		Flags: []cli.Flag{
			flagEBuild,
			flagCategory,
			flagDistfiles,
			flagOutput,
		},
		Action: func (c *cli.Context) error {
			originalEBuildPath := c.String(flagEBuild.Name)
			category := c.String(flagCategory.Name)
			distfiles := c.StringSlice(flagDistfiles.Name)
			finalOutPath := c.String(flagOutput.Name)

			packageShortName, _, err := parseEBuildPath(originalEBuildPath)
			if err != nil {
				return fmt.Errorf("invalid ebuild file name: %w", err)
			}

			exe, err := os.Executable()
			if err != nil {
				return err
			}
			runfilesDir := exe + ".runfiles"

			tmpDir, err := os.MkdirTemp("", "build_ebuild.*")
			if err != nil {
				return err
			}
			defer os.RemoveAll(tmpDir)

			rootDir := filepath.Join(tmpDir, "root")
			overlayDir := filepath.Join(tmpDir, "bazel-overlay")
			packageDir := filepath.Join(overlayDir, category, packageShortName)
			distfilesDir := filepath.Join(tmpDir, "distfiles")
			binaryDir := filepath.Join(tmpDir, "binary")

			for _, dir := range []string{packageDir, distfilesDir, binaryDir} {
				if err := os.MkdirAll(dir, 0o777); err != nil {
					return err
				}
			}

			overlayEbuildPath := filepath.Join(packageDir, filepath.Base(originalEBuildPath))
			if err := copyFile(originalEBuildPath, overlayEbuildPath); err != nil {
				return err
			}

			for _, distfile := range distfiles {
				v := strings.SplitN(distfile, "=", 2)
				if len(v) < 2 {
					return errors.New("invalid distfile spec")
				}
				dst := filepath.Join(distfilesDir, v[0])
				src, err := filepath.Abs(v[1])
				if err != nil {
					return err
				}
				fmt.Printf("***** %s -> %s\n", src, dst)
				if _, err := os.Stat(src); err != nil {
					return err
				}
				if err := os.Symlink(src, dst); err != nil {
					return err
				}
			}

			u, err := user.Current()
			if err != nil {
				return err
			}
			g, err := user.LookupGroupId(u.Gid)
			if err != nil {
				return err
			}

			cmd := exec.Command(
				filepath.Join(runfilesDir, "chromiumos_portage_tool", "ebuild"),
				overlayEbuildPath,
				"clean",
				"package",
			)
			cmd.Env = append(
				os.Environ(),
				// TODO: Drop $PATH. Current it's needed for bzip2.
				"PATH=/usr/sbin:/usr/bin:/sbin:/bin",
				"TMPDIR=" + tmpDir,
				"ROOT=" + rootDir,
				"SYSROOT=" + rootDir,
				"FEATURES=digest",
				"PORTAGE_USERNAME=" + u.Username,
				"PORTAGE_GRPNAME=" + g.Name,
				"PORTAGE_TMPDIR=" + tmpDir,
				"PORTDIR=" + overlayDir,
				"DISTDIR=" + distfilesDir,
				"PKGDIR=" + binaryDir,
			)
			cmd.Stdout = os.Stdout
			cmd.Stderr = os.Stderr
			if err := cmd.Run(); err != nil {
				if err, ok := err.(*exec.ExitError); ok {
					os.Exit(err.ExitCode())
				}
				return err
			}

			binaryOutPath := filepath.Join(
				binaryDir,
				category,
				strings.TrimSuffix(filepath.Base(originalEBuildPath), ebuildExt) + binaryExt)
			if err := copyFile(binaryOutPath, finalOutPath); err != nil {
				return err
			}

			return nil
		},
	}

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
