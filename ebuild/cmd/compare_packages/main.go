// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"archive/tar"
	"bytes"
	"compress/bzip2"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/urfave/cli/v2"

	"cros.local/bazel/ebuild/cmd/compare_packages/equery"
	"cros.local/bazel/ebuild/cmd/compare_packages/golden"
	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/fakechroot"
	"cros.local/bazel/ebuild/private/common/portage"
	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/packages"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

func regenerateGolden(workspaceDir string, board string, goldenPath string) error {
	client := equery.NewClient(workspaceDir, board)

	pkgs, err := client.ListInstalledPackages()
	if err != nil {
		return err
	}

	var infos []*golden.Package
	for _, pkg := range pkgs {
		infos = append(infos, &golden.Package{
			Name:    pkg.Name,
			Version: pkg.Version.String(),
			Uses:    pkg.Uses,
		})
	}

	if err := golden.Save(goldenPath, infos); err != nil {
		return err
	}
	return nil
}

func normalizeHeader(h *tar.Header) error {
	epoch := time.Unix(0, 0)
	h.ModTime = epoch
	h.AccessTime = epoch
	h.ChangeTime = epoch
	for _, key := range []string{"atime", "ctime", "mtime"} {
		delete(h.PAXRecords, key)
	}
	return nil
}

var ignoreFeatures = map[string]struct{}{
	"fakeroot":        {},
	"ipc-sandbox":     {},
	"mount-sandbox":   {},
	"network-sandbox": {},
	"pid-sandbox":     {},
	"sandbox":         {},
	"usersandbox":     {},
}

func normalizeXPAK(xpak binarypackage.XPAK) error {
	xpak["BUILD_TIME"] = []byte("0\n")
	var features []string
	for _, name := range strings.Fields(string(xpak["FEATURES"])) {
		if _, ok := ignoreFeatures[name]; ok {
			continue
		}
		features = append(features, name)
	}
	sort.Strings(features)
	xpak["FEATURES"] = []byte(strings.Join(features, " "))
	env, err := io.ReadAll(bzip2.NewReader(bytes.NewReader(xpak["environment.bz2"])))
	if err != nil {
		return err
	}
	delete(xpak, "environment.bz2")
	xpak["environment"] = env
	return nil
}

func parseBinaryPackage(path string) (files []*tar.Header, xpak binarypackage.XPAK, err error) {
	if err := func() error {
		f, err := binarypackage.BinaryPackage(path)
		if err != nil {
			return err
		}
		defer f.Close()

		raw, err := f.TarballReader()
		if err != nil {
			return err
		}
		defer raw.Close()

		zstd := exec.Command("zstd", "-d")
		zstd.Stdin = raw
		stdout, err := zstd.StdoutPipe()
		if err != nil {
			return err
		}
		zstd.Stderr = os.Stderr
		if err := zstd.Start(); err != nil {
			return err
		}
		defer func() {
			zstd.Process.Kill()
			zstd.Wait()
		}()

		t := tar.NewReader(stdout)
		for {
			file, err := t.Next()
			if err == io.EOF {
				break
			}
			if err != nil {
				return err
			}
			if err := normalizeHeader(file); err != nil {
				return err
			}
			files = append(files, file)
		}

		sort.Slice(files, func(i, j int) bool {
			return files[i].Name < files[j].Name
		})

		xpak, err = f.Xpak()
		if err != nil {
			return err
		}
		if err := normalizeXPAK(xpak); err != nil {
			return err
		}
		return nil
	}(); err != nil {
		return nil, nil, err
	}
	return files, xpak, err
}

func comparePackageUse(want *golden.Package, pkg *packages.Package) error {
	pkgUses := pkg.Uses()
	gotUses := make(map[string]bool)
	for use := range want.Uses {
		gotUses[use] = pkgUses[use]
	}

	if diff := cmp.Diff(gotUses, want.Uses); diff != "" {
		fmt.Printf("USE flags mismatch (-got +want):\n%s", diff)
	}
	return nil
}

func comparePackageContents(want *golden.Package, pkg *packages.Package, pkgDir, workspaceDir string) error {
	const srcDir = "/mnt/host/source/src"
	if !strings.HasPrefix(pkg.Path(), srcDir+"/") {
		return fmt.Errorf("%s is not under %s", pkg.Path(), srcDir)
	}

	gotPath := filepath.Join(workspaceDir, "bazel-bin", strings.TrimSuffix(strings.TrimPrefix(pkg.Path(), srcDir+"/"), ".ebuild")+".tbz2")
	wantPath := filepath.Join(pkgDir, fmt.Sprintf("%s-%s.tbz2", pkg.Name(), pkg.Version().String()))

	if _, err := os.Stat(gotPath); errors.Is(err, fs.ErrNotExist) {
		fmt.Printf("WARNING: skipped content comparison: %v\n", err)
		return nil
	} else if err != nil {
		return err
	}
	if _, err := os.Stat(wantPath); err != nil {
		return err
	}

	gotFiles, gotXPAK, err := parseBinaryPackage(gotPath)
	if err != nil {
		return err
	}
	wantFiles, wantXPAK, err := parseBinaryPackage(wantPath)
	if err != nil {
		return err
	}

	if diff := cmp.Diff(gotFiles, wantFiles); diff != "" {
		fmt.Printf("Files mismatch (-got +want):\n%s", diff)
	}

	gotXPAKStr := make(map[string]string)
	wantXPAKStr := make(map[string]string)
	for key, value := range gotXPAK {
		gotXPAKStr[key] = string(value)
	}
	for key, value := range wantXPAK {
		wantXPAKStr[key] = string(value)
	}
	if diff := cmp.Diff(gotXPAKStr, wantXPAKStr); diff != "" {
		fmt.Printf("XPAK mismatch (-got +want):\n%s", diff)
	}
	return nil
}

func comparePackage(want *golden.Package, resolver *portage.Resolver, workspaceDir string) error {
	fmt.Printf("--- %s-%s\n", want.Name, want.Version)

	ver, err := version.Parse(want.Version)
	if err != nil {
		return err
	}

	pkgs, err := resolver.Packages(dependency.NewAtom(want.Name, dependency.OpExactEqual, ver, false, "", nil))
	if err != nil {
		return err
	}
	if len(pkgs) == 0 {
		return fmt.Errorf("no package found for %s-%s", want.Name, want.Version)
	}
	if len(pkgs) >= 2 {
		return fmt.Errorf("multiple packages found for %s-%s", want.Name, want.Version)
	}
	pkg := pkgs[0]

	if err := comparePackageUse(want, pkg); err != nil {
		return err
	}

	if err := comparePackageContents(want, pkg, resolver.BinaryPackageDir(), workspaceDir); err != nil {
		return err
	}

	return nil
}

var flagBoard = &cli.StringFlag{
	Name:  "board",
	Value: "arm64-generic",
}

var flagRegenerateGolden = &cli.BoolFlag{
	Name: "regenerate-golden",
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagBoard,
		flagRegenerateGolden,
	},
	Action: func(c *cli.Context) error {
		board := c.String(flagBoard.Name)
		regenGolden := c.Bool(flagRegenerateGolden.Name)

		if err := fakechroot.Enter(); err != nil {
			return err
		}

		workspaceDir := bazelutil.WorkspaceDir()
		goldenPath := filepath.Join(workspaceDir, "bazel", "data", "packages.golden.json")

		if regenGolden {
			log.Print("Regenerating golden data. This will take quite a long time...")
			if err := regenerateGolden(workspaceDir, board, goldenPath); err != nil {
				return err
			}
		}

		wants, err := golden.Load(goldenPath)
		if err != nil {
			return err
		}

		resolver, err := portage.NewResolver(filepath.Join("/build", board), portage.NewHackSource())
		if err != nil {
			return err
		}

		for _, want := range wants {
			if err := comparePackage(want, resolver, workspaceDir); err != nil {
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
