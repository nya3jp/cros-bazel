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
	"github.com/urfave/cli"

	"cros.local/ebuild/private/portage/version"
	"cros.local/ebuild/private/portage/xpak"
)

const (
	ebuildExt = ".ebuild"
	binaryExt = ".tbz2"

	sysrootDir = "/build/target/"
)

//go:embed build_ebuild_setup.sh
var setupScript []byte

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

var flagInstallTarget = &cli.StringSliceFlag{
	Name: "install-target",
}

var flagInstallHost = &cli.StringSliceFlag{
	Name: "install-host",
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

var flagRunInContainer = &cli.StringFlag{
	Name:     "run-in-container",
	Required: true,
}

var flagDumbInit = &cli.StringFlag{
	Name:     "dumb-init",
	Required: true,
}

var flagSquashfuse = &cli.StringFlag{
	Name:     "squashfuse",
	Required: true,
}

var flagLogin = &cli.BoolFlag{
	Name: "login",
}

func preparePackages(installPaths []string, dir dualPath) ([]string, error) {
	var atoms []string

	for _, installPath := range installPaths {
		xp, err := xpak.Read(installPath)
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

var app = &cli.App{
	Flags: []cli.Flag{
		flagEBuild,
		flagCategory,
		flagSDK,
		flagOverlay,
		flagInstallTarget,
		flagInstallHost,
		flagDistfile,
		flagFile,
		flagOutput,
		flagRunInContainer,
		flagDumbInit,
		flagSquashfuse,
		flagLogin,
	},
	Action: func(c *cli.Context) error {
		originalEBuildPath := c.String(flagEBuild.Name)
		category := c.String(flagCategory.Name)
		distfileSpecs := c.StringSlice(flagDistfile.Name)
		sdkPath := c.String(flagSDK.Name)
		overlayPaths := c.StringSlice(flagOverlay.Name)
		targetInstallPaths := c.StringSlice(flagInstallTarget.Name)
		hostInstallPaths := c.StringSlice(flagInstallHost.Name)
		fileSpecs := c.StringSlice(flagFile.Name)
		finalOutPath := c.String(flagOutput.Name)
		runInContainerPath := c.String(flagRunInContainer.Name)
		dumbInitPath := c.String(flagDumbInit.Name)
		squashfusePath := c.String(flagSquashfuse.Name)
		login := c.Bool(flagLogin.Name)

		if !login {
			log.Print("HINT: To debug this build environment, run the Bazel build with --spawn_strategy=standalone, and run the command printed below:")
			pwd, _ := os.Getwd()
			log.Printf("( cd %s && %s --login )", shellescape.Quote(pwd), shellescape.QuoteCommand(os.Args))
		}

		packageShortName, _, err := parseEBuildPath(originalEBuildPath)
		if err != nil {
			return fmt.Errorf("invalid ebuild file name: %w", err)
		}

		tmpDir, err := os.MkdirTemp("", "build_ebuild.*")
		if err != nil {
			return err
		}
		defer os.RemoveAll(tmpDir)

		diffDir := filepath.Join(tmpDir, "diff")
		workDir := filepath.Join(tmpDir, "work")

		stageDir := newDualPath(tmpDir, "/").Add("stage")
		overlaysDir := stageDir.Add("overlays")
		ebuildDir := overlaysDir.Add(baseNoExt(overlayPaths[0]), category, packageShortName)
		distfilesDir := stageDir.Add("distfiles")
		packagesDir := stageDir.Add("packages")
		targetPackagesDir := packagesDir.Add("target")
		hostPackagesDir := packagesDir.Add("host")
		extrasDir := stageDir.Add("extras")

		for _, dir := range []string{diffDir, workDir} {
			if err := os.MkdirAll(dir, 0o755); err != nil {
				return err
			}
		}
		for _, dir := range []dualPath{stageDir, overlaysDir, ebuildDir, distfilesDir, packagesDir, targetPackagesDir, hostPackagesDir, extrasDir} {
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
			if err := copyFile(v[1], distfilesDir.Add(v[0]).Outside()); err != nil {
				return err
			}
		}

		args := []string{
			runInContainerPath,
			"--diff-dir=" + diffDir,
			"--work-dir=" + workDir,
			"--dumb-init=" + dumbInitPath,
			"--squashfuse=" + squashfusePath,
			"--overlay-dir=" + stageDir.Inside() + "=" + stageDir.Outside(),
			"--overlay-dir=" + ebuildDir.Inside() + "=" + ebuildDir.Outside(),
			"--overlay-squashfs=/=" + sdkPath,
		}

		var overlayDirsInside []string
		for _, overlayPath := range overlayPaths {
			overlayDir := overlaysDir.Add(baseNoExt(overlayPath))
			args = append(args, "--overlay-squashfs="+overlayDir.Inside()+"="+overlayPath)
			overlayDirsInside = append(overlayDirsInside, overlayDir.Inside())
		}

		hostInstallAtoms, err := preparePackages(hostInstallPaths, hostPackagesDir)
		if err != nil {
			return err
		}

		targetInstallAtoms, err := preparePackages(targetInstallPaths, targetPackagesDir)
		if err != nil {
			return err
		}

		setupPath := stageDir.Add("setup.sh")
		if err := os.WriteFile(setupPath.Outside(), setupScript, 0o755); err != nil {
			return err
		}

		args = append(args, setupPath.Inside())
		if !login {
			args = append(args, "fakeroot", "ebuild", overlayEbuildPath.Inside(), "clean", "package")
		}
		cmd := exec.Command(args[0], args[1:]...)
		cmd.Env = append(
			os.Environ(),
			"PATH=/usr/sbin:/usr/bin:/sbin:/bin",
			"FEATURES=digest -sandbox -usersandbox", // TODO: turn on sandbox
			"PORTAGE_USERNAME=root",
			"PORTAGE_GRPNAME=root",
			"ROOT=/build/target/",
			"SYSROOT=/build/target/",
			"PORTAGE_CONFIGROOT=/build/target/",
			// TODO: Dump PORTDIRs in make.conf.
			"PORTDIR="+overlayDirsInside[0],
			"PORTDIR_OVERLAY="+strings.Join(overlayDirsInside[1:], " "),
			"DISTDIR="+distfilesDir.Inside(),
			"PKGDIR="+targetPackagesDir.Inside(),
			"PKGDIR_HOST="+hostPackagesDir.Inside(),
			"INSTALL_ATOMS_HOST="+strings.Join(hostInstallAtoms, " "),
			"INSTALL_ATOMS_TARGET="+strings.Join(targetInstallAtoms, " "),
		)
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
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
