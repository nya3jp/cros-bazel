package main

import (
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/urfave/cli"

	"cros.local/ebuild/private/portage/version"
)

const (
	ebuildExt = ".ebuild"
	binaryExt = ".tbz2"
)

var systemBinPaths = []string{
	"/usr/sbin",
	"/usr/bin",
	"/sbin",
	"/bin",
}

type dualPath struct {
	outside, inside string
}

func newDualPath(outside, inside string) dualPath {
	return dualPath{outside: outside, inside: inside}
}

func (dp dualPath) Outside() string { return dp.outside }
func (dp dualPath) Inside() string { return dp.inside }

func (dp dualPath) Add(components ...string) dualPath {
	return newDualPath(
		filepath.Join(append([]string{dp.outside}, components...)...),
		filepath.Join(append([]string{dp.inside}, components...)...))
}

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

func runCommand(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func copyFile(src, dst string) error {
	return runCommand("/usr/bin/cp", "--", src, dst)
}

func convertTbz2ToSquashfs(src, dst string) error {
	r, w, err := os.Pipe()
	if err != nil {
		return err
	}
	defer r.Close()
	defer w.Close()

	bzcat := exec.Command("/usr/bin/bzcat", src)
	bzcat.Stdout = w
	bzcat.Stderr = os.Stderr
	if err := bzcat.Start(); err != nil {
		return err
	}
	defer func() {
		bzcat.Process.Kill()
		bzcat.Wait()
	}()

	w.Close()

	mksquashfs := exec.Command("/usr/bin/mksquashfs", "-", dst, "-tar")
	mksquashfs.Stdin = r
	mksquashfs.Stderr = os.Stderr
	return mksquashfs.Run()
}

var flagEBuild = &cli.StringFlag{
	Name: "ebuild",
	Required: true,
}

var flagCategory = &cli.StringFlag{
	Name: "category",
	Required: true,
}

var flagDistfile = &cli.StringSliceFlag{
	Name: "distfile",
}

var flagEclass = &cli.StringSliceFlag{
	Name: "eclass",
}

var flagOverlaySquashfs = &cli.StringSliceFlag{
	Name: "overlay-squashfs",
}

var flagOutput = &cli.StringFlag{
	Name: "output",
	Required: true,
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagEBuild,
		flagCategory,
		flagDistfile,
		flagEclass,
		flagOverlaySquashfs,
		flagOutput,
	},
	Action: func (c *cli.Context) error {
		originalEBuildPath := c.String(flagEBuild.Name)
		category := c.String(flagCategory.Name)
		distfiles := c.StringSlice(flagDistfile.Name)
		eclasses := c.StringSlice(flagEclass.Name)
		images := c.StringSlice(flagOverlaySquashfs.Name)
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

		storeDir := filepath.Join(tmpDir, "store")

		const stageMountPoint = "/stage"
		stageDir := newDualPath(filepath.Join(tmpDir, "stage"), stageMountPoint)
		overlayDir := stageDir.Add("bazel-overlay")
		eclassDir := overlayDir.Add("eclass")
		packageDir := overlayDir.Add(category, packageShortName)
		distfilesDir := stageDir.Add("distfiles")
		binaryPkgsDir := stageDir.Add("binpkgs")

		if err := os.Mkdir(storeDir, 0o755); err != nil {
			return err
		}
		for _, dir := range []dualPath{stageDir, overlayDir, eclassDir, packageDir, distfilesDir, binaryPkgsDir} {
			if err := os.MkdirAll(dir.Outside(), 0o755); err != nil {
				return err
			}
		}

		overlayEbuildPath := packageDir.Add(filepath.Base(originalEBuildPath))
		if err := copyFile(originalEBuildPath, overlayEbuildPath.Outside()); err != nil {
			return err
		}

		for _, distfile := range distfiles {
			v := strings.SplitN(distfile, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid distfile spec")
			}
			if err := copyFile(v[1], distfilesDir.Add(v[0]).Outside()); err != nil {
				return err
			}
		}

		for _, eclass := range eclasses {
			if err := copyFile(eclass, eclassDir.Add(filepath.Base(eclass)).Outside()); err != nil {
				return err
			}
		}

		args := []string{
			"--store=" + storeDir,
			"--init=" + filepath.Join(runfilesDir, "dumb_init/file/downloaded"),
			"--bind-mount=" + filepath.Join("/host", stageDir.Outside()) + ":" + stageMountPoint,
		}
		for _, image := range images {
			args = append(args, "--overlay-squashfs=" + image)
		}
		args = append(args,
			"ebuild",
			overlayEbuildPath.Inside(),
			"clean",
			"package",
		)
		cmd := exec.Command(
			filepath.Join(runfilesDir, "rules_ebuild/ebuild/private/run_in_container_/run_in_container"),
			args...)
		cmd.Env = append(
			os.Environ(),
			"PATH=" + strings.Join(systemBinPaths, ":"),
			"ROOT=/",
			"SYSROOT=/",
			"FEATURES=digest -sandbox -usersandbox",  // TODO: turn on sandbox
			"PORTAGE_USERNAME=root",
			"PORTAGE_GRPNAME=root",
			"PORTDIR=" + overlayDir.Inside(),
			"DISTDIR=" + distfilesDir.Inside(),
			"PKGDIR=" + binaryPkgsDir.Inside(),
		)
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			if err, ok := err.(*exec.ExitError); ok {
				os.Exit(err.ExitCode())
			}
			return err
		}

		binaryOutPath := binaryPkgsDir.Add(
			category,
			strings.TrimSuffix(filepath.Base(originalEBuildPath), ebuildExt) + binaryExt)
		if err := convertTbz2ToSquashfs(binaryOutPath.Outside(), finalOutPath); err != nil {
			return fmt.Errorf("converting tbz2 to squashfs: %w", err)
		}

		return nil
	},
}

func main() {
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
