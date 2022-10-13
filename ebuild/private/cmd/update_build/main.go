// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"text/template"

	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/cmd/update_build/distfiles"
	"cros.local/bazel/ebuild/private/common/depdata"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
)

const ebuildExt = ".ebuild"

var overlayRelDirs = []string{
	// The order matters; the first one has the highest priority.
	"overlays/overlay-arm64-generic",
	"third_party/chromiumos-overlay",
	"third_party/portage-stable",
}

// TODO: Remove this blocklist.
var blockedPackages = map[string]struct{}{
	"fzf":   {}, // tries to access goproxy.io
	"jlink": {}, // distfile unavailable due to restricted license
	"shfmt": {}, // tries to access goproxy.io
}

type ebuildInfo struct {
	EBuildName  string
	PackageName string
	Category    string
	Dists       []*distfiles.Entry
	PackageInfo *depdata.PackageInfo
}

type packageGroup struct {
	PackageName string
	Packages    []string
}

var repositoriesTemplate = template.Must(template.New("").Parse(`# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def dependencies():
{{- range . }}
    http_file(
        name = "{{ .Name }}",
        downloaded_file_path = "{{ .Filename }}",
        sha256 = "{{ .SHA256 }}",
        urls = ["{{ .URL }}"],
    )
{{- end }}
`))

var buildHeaderTemplate = template.Must(template.New("").Parse(`# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild:defs.bzl", "ebuild")
`))

var ebuildTemplate = template.Must(template.New("").Parse(`
ebuild(
    name = "{{ .PackageName }}",
    ebuild = "{{ .EBuildName }}",
    category = "{{ .Category }}",
    distfiles = {
        {{- range .Dists }}
        "@{{ .Name }}//file": "{{ .Filename }}",
        {{- end }}
    },
    {{- if .PackageInfo.LocalSrc }}
    srcs = [
        {{- range .PackageInfo.LocalSrc }}
        "{{ . }}",
        {{- end }}
    ],
    {{- end }}
    {{- if .PackageInfo.BuildDeps }}
    build_deps = [
        {{- range .PackageInfo.BuildDeps }}
        "{{ . }}",
        {{- end }}
    ],
    {{- end }}
    {{- if .PackageInfo.RuntimeDeps }}
    runtime_deps = [
        {{- range .PackageInfo.RuntimeDeps }}
        "{{ . }}",
        {{- end }}
    ],
    {{- end }}
    files = glob(["files/**"]),
    visibility = ["//visibility:public"],
)
`))

var packageGroupTemplate = template.Must(template.New("").Parse(`
load("//bazel/ebuild:defs.bzl", "package_set")

package_set(
    name = "{{ .PackageName }}",
    deps = [
        {{- range .Packages }}
        ":{{ . }}",
        {{- end }}
    ],
    visibility = ["//visibility:public"],
)
`))

func writeToFile(buildPath string, fn func(f *os.File) error) error {
	f, err := os.Create(buildPath)
	if err != nil {
		return err
	}
	defer f.Close()

	return fn(f)
}

func generateRepositories(bzlPath string, dists []*distfiles.Entry) error {
	if len(dists) == 0 {
		return nil
	}

	f, err := os.Create(bzlPath)
	if err != nil {
		return err
	}
	defer f.Close()

	return repositoriesTemplate.Execute(f, dists)
}

func packageNameToLabel(name string, overlayDirs []string) (string, error) {
	for _, overlayDir := range overlayDirs {
		fis, err := os.ReadDir(filepath.Join(overlayDir, name))
		if err != nil {
			continue
		}
		hasEBuild := false
		for _, fi := range fis {
			if strings.HasSuffix(fi.Name(), ".ebuild") {
				hasEBuild = true
			}
		}
		if !hasEBuild {
			continue
		}
		v := strings.Split(overlayDir, "/")
		return fmt.Sprintf("//%s/%s/%s", v[len(v)-2], v[len(v)-1], name), nil
	}
	return "", fmt.Errorf("%s not found in overlays", name)
}

func packageAtomsToLabels(atoms []string, overlayDirs []string) ([]string, error) {
	var labels []string
	for _, atromStr := range atoms {
		atom, err := dependency.ParseAtom(atromStr)
		if err != nil {
			return nil, err
		}

		label, err := packageNameToLabel(atom.PackageName(), overlayDirs)
		if err != nil {
			return nil, err
		}

		if atom.Version() != nil {
			name := strings.Split(atom.PackageName(), "/")[1]
			label = fmt.Sprintf("%s:%s-%s", label, name, atom.Version())
		}

		labels = append(labels, label)
	}
	return labels, nil
}

func getDistEntries(cache *distfiles.Cache, pkgInfo *depdata.PackageInfo) ([]*distfiles.Entry, error) {
	var dists []*distfiles.Entry
	for filename, srcInfo := range pkgInfo.SrcUris {
		// Check the cache first.
		if dist, ok := cache.Get(filename); ok {
			dists = append(dists, dist)
			continue
		}

		log.Printf("  locating distfile %s...", filename)
		dist, err := distfiles.Locate(filename, srcInfo)
		if err != nil {
			log.Fatalf("WARNING: unable to locate distfile %s: %v", filename, err)
			// TODO: Do we want to support negative caching?
			continue
		} else {
			dists = append(dists, dist)
		}

		cache.Set(filename, dist)
	}

	sort.Slice(dists, func(i, j int) bool {
		return dists[i].Name < dists[j].Name
	})

	return dists, nil
}

func generatePackage(ebuildDir string, pkgInfoMap depdata.PackageInfoMap, cache *distfiles.Cache) ([]*distfiles.Entry, error) {
	v := strings.Split(ebuildDir, "/")
	category := v[len(v)-2]
	packageName := v[len(v)-1]
	buildPath := filepath.Join(ebuildDir, "BUILD.bazel")

	pkgInfos := pkgInfoMap[fmt.Sprintf("%s/%s", category, packageName)]
	_, blocked := blockedPackages[packageName]
	if pkgInfos == nil || blocked {
		if err := os.Remove(buildPath); err != nil && !errors.Is(err, os.ErrNotExist) {
			return nil, err
		}
		return nil, nil
	}

	log.Printf("Generating: %s", buildPath)

	var ebuildInfos []ebuildInfo

	// Keep the previous behavior where rust pkgs were sorted ascending
	sort.Slice(pkgInfos, func(i, j int) bool {
		return pkgInfos[i].Version < pkgInfos[j].Version
	})

	var allDists []*distfiles.Entry

	for _, pkgInfo := range pkgInfos {
		dists, err := getDistEntries(cache, pkgInfo)
		if err != nil {
			return nil, err
		}

		ebuildName := fmt.Sprintf("%s-%s.ebuild", packageName, pkgInfo.Version)

		var localPackageName string
		if len(pkgInfos) == 1 {
			localPackageName = packageName
		} else {
			localPackageName = fmt.Sprintf("%s-%s", packageName, pkgInfo.Version)
		}

		ebuild := ebuildInfo{
			EBuildName:  ebuildName,
			PackageName: localPackageName,
			Category:    category,
			Dists:       dists,
			PackageInfo: pkgInfo,
		}

		ebuildInfos = append(ebuildInfos, ebuild)
		allDists = append(allDists, dists...)
	}

	if err := writeToFile(buildPath, func(f *os.File) error {
		if err := buildHeaderTemplate.Execute(f, nil); err != nil {
			return err
		}

		for _, ebuild := range ebuildInfos {
			if err := ebuildTemplate.Execute(f, ebuild); err != nil {
				return err
			}
		}

		if len(ebuildInfos) > 1 {
			var targetNames []string

			for _, ebuild := range ebuildInfos {
				targetNames = append(targetNames, ebuild.PackageName)
			}

			packageGroup := packageGroup{
				PackageName: packageName,
				Packages:    targetNames,
			}
			if err := packageGroupTemplate.Execute(f, packageGroup); err != nil {
				return err
			}
		}

		return nil
	}); err != nil {
		return nil, err
	}
	return allDists, nil
}

func generateOverlay(overlayDir string, pkgInfoMap depdata.PackageInfoMap, cache *distfiles.Cache) error {
	var overlayDists []*distfiles.Entry

	var ebuildDirs []string
	if err := filepath.WalkDir(overlayDir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !d.IsDir() {
			return nil
		}

		fis, err := os.ReadDir(path)
		if err != nil {
			return err
		}
		for _, fi := range fis {
			if strings.HasSuffix(fi.Name(), ebuildExt) {
				ebuildDirs = append(ebuildDirs, path)
				return fs.SkipDir
			}
		}
		return nil
	}); err != nil {
		return err
	}

	for _, ebuildDir := range ebuildDirs {
		dists, err := generatePackage(ebuildDir, pkgInfoMap, cache)
		if err != nil {
			return err
		}
		overlayDists = append(overlayDists, dists...)
	}

	overlayDists = distfiles.Unique(overlayDists)

	if err := generateRepositories(filepath.Join(overlayDir, "repositories.bzl"), overlayDists); err != nil {
		return err
	}
	return nil
}

func generate(overlayDirs []string, pkgInfoMap depdata.PackageInfoMap, cache *distfiles.Cache) error {
	for _, overlayDir := range overlayDirs {
		if err := generateOverlay(overlayDir, pkgInfoMap, cache); err != nil {
			return err
		}
	}
	return nil
}

var flagPackageInfoFile = &cli.StringFlag{
	Name: "package-info-file",
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagPackageInfoFile,
	},
	Action: func(c *cli.Context) error {
		packageInfoPath := c.String(flagPackageInfoFile.Name)

		workspaceDir := os.Getenv("BUILD_WORKSPACE_DIRECTORY")

		var overlayDirs []string
		for _, overlayRelDir := range overlayRelDirs {
			overlayDir := filepath.Join(workspaceDir, overlayRelDir)
			overlayDirs = append(overlayDirs, overlayDir)
		}

		var pkgInfoMap depdata.PackageInfoMap
		if packageInfoPath != "" {
			b, err := os.ReadFile(packageInfoPath)
			if err != nil {
				return err
			}
			if err := json.Unmarshal(b, &pkgInfoMap); err != nil {
				return err
			}

			// Rewrite package names to labels.
			for _, pkgInfos := range pkgInfoMap {
				for _, pkgInfo := range pkgInfos {
					pkgInfo.BuildDeps, err = packageAtomsToLabels(pkgInfo.BuildDeps, overlayDirs)
					if err != nil {
						return err
					}
					pkgInfo.RuntimeDeps, err = packageAtomsToLabels(pkgInfo.RuntimeDeps, overlayDirs)
					if err != nil {
						return err
					}
				}
			}
		}

		cache, err := distfiles.NewCache(filepath.Join(workspaceDir, "bazel/data/distfiles.json"))
		if err != nil {
			return err
		}

		return generate(overlayDirs, pkgInfoMap, cache)
	},
}

func main() {
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
