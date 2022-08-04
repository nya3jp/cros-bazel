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

	sysrootDir = "/build/target/"
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
func (dp dualPath) Inside() string  { return dp.inside }

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
	Name:     "ebuild",
	Required: true,
}

var flagCategory = &cli.StringFlag{
	Name:     "category",
	Required: true,
}

var flagSDK = &cli.StringFlag{
	Name:     "sdk",
	Required: true,
}

var flagOverlay = &cli.StringSliceFlag{
	Name:     "overlay",
	Required: true,
}

var flagDependency = &cli.StringSliceFlag{
	Name: "dependency",
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

var app = &cli.App{
	Flags: []cli.Flag{
		flagEBuild,
		flagCategory,
		flagSDK,
		flagOverlay,
		flagDependency,
		flagDistfile,
		flagFile,
		flagOutput,
	},
	Action: func(c *cli.Context) error {
		originalEBuildPath := c.String(flagEBuild.Name)
		category := c.String(flagCategory.Name)
		distfileSpecs := c.StringSlice(flagDistfile.Name)
		sdkPath := c.String(flagSDK.Name)
		overlayPaths := c.StringSlice(flagOverlay.Name)
		dependencyPaths := c.StringSlice(flagDependency.Name)
		fileSpecs := c.StringSlice(flagFile.Name)
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

		diffDir := filepath.Join(tmpDir, "diff")
		workDir := filepath.Join(tmpDir, "work")

		stageDir := newDualPath(tmpDir, "/").Add("stage")
		inputDir := stageDir.Add("input")
		outputDir := stageDir.Add("output")
		overlaysDir := inputDir.Add("overlays")
		packageDir := overlaysDir.Add(baseNoExt(overlayPaths[0]), category, packageShortName)
		distfilesDir := outputDir.Add("distfiles")
		binaryPkgsDir := outputDir.Add("binpkgs")

		for _, dir := range []string{diffDir, workDir} {
			if err := os.MkdirAll(dir, 0o755); err != nil {
				return err
			}
		}
		for _, dir := range []dualPath{inputDir, outputDir, overlaysDir, packageDir, distfilesDir, binaryPkgsDir} {
			if err := os.MkdirAll(dir.Outside(), 0o755); err != nil {
				return err
			}
		}

		overlayEbuildPath := packageDir.Add(filepath.Base(originalEBuildPath))
		if err := copyFile(originalEBuildPath, overlayEbuildPath.Outside()); err != nil {
			return err
		}

		for _, fileSpec := range fileSpecs {
			v := strings.SplitN(fileSpec, "=", 2)
			if len(v) < 2 {
				return errors.New("invalid file spec")
			}
			src := v[1]
			dst := packageDir.Add(v[0]).Outside()
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
			if err := copyFile(v[1], distfilesDir.Add(v[0]).Outside()); err != nil {
				return err
			}
		}

		args := []string{
			"--diff-dir=" + diffDir,
			"--work-dir=" + workDir,
			// TODO: Consider avoiding runfiles.
			"--init=" + filepath.Join(runfilesDir, "dumb_init/file/downloaded"),
			"--overlay-dir=" + stageDir.Inside() + "=" + stageDir.Outside(),
			"--overlay-dir=" + packageDir.Inside() + "=" + packageDir.Outside(),
			"--overlay-squashfs=/=" + sdkPath,
		}

		var overlayDirsInside []string
		for _, overlayPath := range overlayPaths {
			overlayDir := overlaysDir.Add(baseNoExt(overlayPath))
			args = append(args, "--overlay-squashfs="+overlayDir.Inside()+"="+overlayPath)
			overlayDirsInside = append(overlayDirsInside, overlayDir.Inside())
		}

		for _, dependencyPath := range dependencyPaths {
			args = append(args, "--overlay-squashfs="+sysrootDir+"="+dependencyPath)
		}

		args = append(args,
			"bash",
			"-c",
			// TODO: Can we get rid of mkdir?
			`mkdir -p "${ROOT}"; ln -sf "$0" /etc/portage/make.profile; exec "$@"`,
			// TODO: Avoid hard-coding the default profile.
			"/stage/input/overlays/chromiumos-overlay/profiles/default/linux/amd64/10.0/sdk",
			"ebuild",
			overlayEbuildPath.Inside(),
			"clean",
			"package",
		)
		cmd := exec.Command(
			// TODO: Consider avoiding runfiles.
			filepath.Join(runfilesDir, "rules_ebuild/ebuild/private/run_in_container_/run_in_container"),
			args...)
		cmd.Env = append(
			os.Environ(),
			"PATH="+strings.Join(systemBinPaths, ":"),
			"ROOT="+sysrootDir,
			"SYSROOT="+sysrootDir,
			"FEATURES=digest -sandbox -usersandbox", // TODO: turn on sandbox
			"PORTAGE_USERNAME=root",
			"PORTAGE_GRPNAME=root",
			"PORTDIR="+overlayDirsInside[0],
			"PORTDIR_OVERLAY="+strings.Join(overlayDirsInside[1:], " "),
			"DISTDIR="+distfilesDir.Inside(),
			"PKGDIR="+binaryPkgsDir.Inside(),
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
			strings.TrimSuffix(filepath.Base(originalEBuildPath), ebuildExt)+binaryExt)
		if err := convertTbz2ToSquashfs(filepath.Join(diffDir, binaryOutPath.Inside()), finalOutPath); err != nil {
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
