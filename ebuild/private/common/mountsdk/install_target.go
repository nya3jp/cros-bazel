package mountsdk

import (
	"fmt"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"github.com/urfave/cli/v2"
)

const binaryExt = ".tbz2"

var FlagInstallTarget = &cli.StringSliceFlag{
	Name:  "install-target",
	Usage: "<binpkg>[:<binpkg>]+: All binpkgs specified will be installed in parallel",
}

func preparePackages(installPaths []string, dir string) (mounts []makechroot.BindMount, atoms []string, err error) {
	for _, installPath := range installPaths {
		xp, err := binarypackage.ReadXpak(installPath)
		if err != nil {
			return nil, nil, fmt.Errorf("reading %s: %w", filepath.Base(installPath), err)
		}
		category := strings.TrimSpace(string(xp["CATEGORY"]))
		pf := strings.TrimSpace(string(xp["PF"]))

		mounts = append(mounts, makechroot.BindMount{
			Source:    installPath,
			MountPath: filepath.Join(dir, category, pf+binaryExt),
		})

		atoms = append(atoms, fmt.Sprintf("=%s/%s", category, pf))
	}

	return mounts, atoms, nil
}

func preparePackageGroups(installGroups [][]string, dir string) (mounts []makechroot.BindMount, atomGroups [][]string, err error) {
	for _, installGroup := range installGroups {
		packageMounts, atoms, err := preparePackages(installGroup, dir)
		if err != nil {
			return nil, nil, err
		}
		mounts = append(mounts, packageMounts...)
		atomGroups = append(atomGroups, atoms)
	}

	return mounts, atomGroups, nil
}

func AddInstallTargetsToConfig(installTargetsUnparsed []string, targetPackagesDir string, cfg *Config) (extraEnv []string, err error) {
	var targetInstallGroups [][]string
	for _, targetGroupStr := range installTargetsUnparsed {
		targets := strings.Split(targetGroupStr, ":")
		targetInstallGroups = append(targetInstallGroups, targets)
	}
	packageMounts, targetInstallAtomGroups, err := preparePackageGroups(targetInstallGroups, targetPackagesDir)
	if err != nil {
		return nil, err
	}
	cfg.BindMounts = append(cfg.BindMounts, packageMounts...)

	for i, atomGroup := range targetInstallAtomGroups {
		extraEnv = append(extraEnv,
			fmt.Sprintf("INSTALL_ATOMS_TARGET_%d=%s", i,
				strings.Join(atomGroup, " ")))
	}
	return extraEnv, nil
}
