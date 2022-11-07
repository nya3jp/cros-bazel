// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"errors"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"text/template"

	"github.com/urfave/cli/v2"

	"cros.local/bazel/ebuild/cmd/update_build/distfiles"
	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/commonflags"
	"cros.local/bazel/ebuild/private/common/depdata"
)

const ebuildExt = ".ebuild"

var manuallyCreatedBuildFiles = map[string]struct{}{
	"chromeos-fonts": {},
}

var overlayRelDirs = []string{
	// The order matters; the first one has the highest priority.
	"overlays/overlay-arm64-generic",
	"third_party/chromiumos-overlay",
	"third_party/portage-stable",
}

type ebuildInfo struct {
	EBuildName  string
	PackageName string
	Category    string
	Dists       []*distfiles.Entry
	PackageInfo *depdata.PackageInfo
	PostDeps    []string
}

type packageGroup struct {
	PackageName string
	Packages    []string
}

type repositoriesTemplateVars struct {
	Dists []*distfiles.Entry
}

var repositoriesTemplate = template.Must(template.New("").Parse(`# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def dependencies():
{{- range .Dists }}
    http_file(
        name = "{{ .Name }}",
        downloaded_file_path = "{{ .Filename }}",
        sha256 = "{{ .SHA256 }}",
        urls = ["{{ .URL }}"],
    )
{{- end }}
`))

type buildTemplateVars struct {
	EBuilds []ebuildInfo
}

var buildTemplate = template.Must(template.New("").Parse(`# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild:defs.bzl", "ebuild", "package_set")

{{ range .EBuilds -}}
ebuild(
    name = "{{ .PackageInfo.MainSlot }}",
    ebuild = "{{ .EBuildName }}",
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
    files = glob(["files/**", "*.bashrc"]),
    visibility = ["//visibility:public"],
)

package_set(
    name = "{{ .PackageInfo.MainSlot }}_package_set",
    deps = [
        ":{{ .PackageInfo.MainSlot }}",
        {{- range .PostDeps }}
        "{{ . }}",
        {{- end }}
    ],
    visibility = ["//visibility:public"],
)
{{ end -}}
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

	return repositoriesTemplate.Execute(f, &repositoriesTemplateVars{
		Dists: dists,
	})
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

func generatePackage(ebuildDir string, pkgInfos []*depdata.PackageInfo, postDepsMap map[string][]string, cache *distfiles.Cache) ([]*distfiles.Entry, error) {
	v := strings.Split(ebuildDir, "/")
	category := v[len(v)-2]
	packageName := v[len(v)-1]
	buildPath := filepath.Join(ebuildDir, "BUILD.bazel")

	if _, ok := manuallyCreatedBuildFiles[packageName]; ok {
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
		label := fmt.Sprintf("//%s:%s", filepath.Dir(pkgInfo.EBuildPath), pkgInfo.MainSlot)

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
			PostDeps:    postDepsMap[label],
		}

		ebuildInfos = append(ebuildInfos, ebuild)
		allDists = append(allDists, dists...)
	}

	if err := writeToFile(buildPath, func(f *os.File) error {
		return buildTemplate.Execute(f, &buildTemplateVars{
			EBuilds: ebuildInfos,
		})
	}); err != nil {
		return nil, err
	}
	return allDists, nil
}

func cleanOverlay(overlayDir string) error {
	return filepath.WalkDir(overlayDir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !d.IsDir() {
			return nil
		}

		if _, ok := manuallyCreatedBuildFiles[d.Name()]; ok {
			return fs.SkipDir
		}

		fis, err := os.ReadDir(path)
		if err != nil {
			return err
		}
		for _, fi := range fis {
			if strings.HasSuffix(fi.Name(), ebuildExt) {
				if err := os.Remove(filepath.Join(path, "BUILD.bazel")); err != nil && !errors.Is(err, fs.ErrNotExist) {
					return err
				}
				return fs.SkipDir
			}
		}
		return nil
	})
}

func generateOverlay(overlayDir string, workspaceDir string, pkgInfoMap depdata.PackageInfoMap, postDepsMap map[string][]string, cache *distfiles.Cache) error {
	if err := cleanOverlay(overlayDir); err != nil {
		return err
	}

	pkgInfoByDir := make(map[string][]*depdata.PackageInfo)
	for _, info := range pkgInfoMap {
		ebuildDir := filepath.Dir(filepath.Join(workspaceDir, info.EBuildPath))
		if !strings.HasPrefix(ebuildDir, overlayDir+"/") {
			continue
		}
		pkgInfoByDir[ebuildDir] = append(pkgInfoByDir[ebuildDir], info)
	}

	var overlayDists []*distfiles.Entry

	for ebuildDir, infos := range pkgInfoByDir {
		dists, err := generatePackage(ebuildDir, infos, postDepsMap, cache)
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

func generate(workspaceDir string, pkgInfoMap depdata.PackageInfoMap, postDepsMap map[string][]string, cache *distfiles.Cache) error {
	for _, overlayRelDir := range overlayRelDirs {
		overlayDir := filepath.Join(workspaceDir, overlayRelDir)
		if err := generateOverlay(overlayDir, workspaceDir, pkgInfoMap, postDepsMap, cache); err != nil {
			return err
		}
	}
	return nil
}

func propagatePostDeps(pkgInfoMap depdata.PackageInfoMap) map[string][]string {
	postDepsMap := make(map[string][]string)

	var dfs func(label string) // for recursive calls
	dfs = func(label string) {
		if _, ok := postDepsMap[label]; ok {
			return
		}

		info := pkgInfoMap[label]

		postDepSet := make(map[string]struct{})
		for _, postDep := range info.PostDeps {
			postDepSet[postDep] = struct{}{}
		}

		for _, runtimeDep := range info.RuntimeDeps {
			dfs(runtimeDep)
			for _, postDep := range postDepsMap[runtimeDep] {
				postDepSet[postDep] = struct{}{}
			}
		}

		// Remove self-dependencies.
		delete(postDepSet, label)

		var postDeps []string
		for postDep := range postDepSet {
			postDeps = append(postDeps, postDep)
		}
		sort.Strings(postDeps)
		postDepsMap[label] = postDeps
	}

	for label := range pkgInfoMap {
		dfs(label)
	}

	return postDepsMap
}

var app = &cli.App{
	Flags: []cli.Flag{
		commonflags.DepsJSON,
		commonflags.DistfilesJSON,
	},
	Action: func(c *cli.Context) error {
		depsJSONPath := c.String(commonflags.DepsJSON.Name)
		distfilesJSONPath := c.String(commonflags.DistfilesJSON.Name)

		workspaceDir := bazelutil.WorkspaceDir()

		pkgInfoMap, err := depdata.Load(depsJSONPath)
		if err != nil {
			return err
		}

		postDepsMap := propagatePostDeps(pkgInfoMap)

		cache, err := distfiles.NewCache(distfilesJSONPath)
		if err != nil {
			return err
		}

		return generate(workspaceDir, pkgInfoMap, postDepsMap, cache)
	},
}

func main() {
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
