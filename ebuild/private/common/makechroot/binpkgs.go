package makechroot

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"cros.local/rules_ebuild/ebuild/private/common/fileutil"
	"cros.local/rules_ebuild/ebuild/private/common/portage/xpak"
)

func CopyBinaryPackages(packagesDir string, packagePaths []string) (atoms []string, err error) {
	const binaryExt = ".tbz2"

	for _, packagePath := range packagePaths {
		xp, err := xpak.Read(packagePath)
		if err != nil {
			return nil, fmt.Errorf("reading %s: %w", filepath.Base(packagePath), err)
		}
		category := strings.TrimSpace(string(xp["CATEGORY"]))
		pf := strings.TrimSpace(string(xp["PF"]))

		categoryDir := filepath.Join(packagesDir, category)
		if err := os.MkdirAll(categoryDir, 0o755); err != nil {
			return nil, err
		}

		copyPath := filepath.Join(categoryDir, pf+binaryExt)
		if err := fileutil.Copy(packagePath, copyPath); err != nil {
			return nil, err
		}

		atoms = append(atoms, fmt.Sprintf("=%s/%s", category, pf))
	}

	return atoms, nil
}
