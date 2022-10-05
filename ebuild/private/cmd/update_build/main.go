// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"log"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"text/template"

	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/common/standard/version"
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

var distBaseURLs = []string{
	"https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/",
	"https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/",
	"https://commondatastorage.googleapis.com/chromeos-localmirror/lvfs/",
	"https://storage.googleapis.com/chromeos-localmirror/secureshell/distfiles/",
	"https://storage.googleapis.com/chromeos-localmirror/secureshell/releases/",
	"https://storage.googleapis.com/chromium-nodejs/14.15.4/",
	"https://storage.googleapis.com/chromium-nodejs/16.13.0",
}

// The following packages don't exist in the mirrors above, but instead
// need to be pulled from the SRC_URI. We should probably mirror these so our
// build isn't dependent on external hosts.
var manualDistfileMap = map[string]string{
	"iproute2-5.16.0.tar.gz": "http://www.kernel.org/pub/linux/utils/net/iproute2/iproute2-5.16.0.tar.gz",
}

type distEntry struct {
	Filename string `json:"filename"`
	URL      string `json:"url"`
	SHA256   string `json:"sha256"`
	Name     string `json:"name"`
}

type packageInfo struct {
	BuildDeps   []string `json:"buildDeps"`
	LocalSrc    []string `json:"localSrc"`
	RuntimeDeps []string `json:"runtimeDeps"`
}

type ebuildInfo struct {
	EBuildName  string
	PackageName string
	Category    string
	Dists       []*distEntry
	PackageInfo *packageInfo
}

type packageGroup struct {
	PackageName string
	Packages    []string
}

func getSHA256(url string) (string, error) {
	res, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer res.Body.Close()

	if res.StatusCode/100 != 2 {
		return "", fmt.Errorf("http status %d", res.StatusCode)
	}

	hasher := sha256.New()
	if _, err := io.Copy(hasher, res.Body); err != nil {
		return "", err
	}
	return hex.EncodeToString(hasher.Sum(nil)), nil
}

func locateDistFile(filename string) (*distEntry, error) {
	for _, distBaseURL := range distBaseURLs {
		var url string
		if url = manualDistfileMap[filename]; url == "" {
			url = distBaseURL + filename
		}
		// TODO: This SHA should come from the manifest
		if sha256, err := getSHA256(url); err == nil {
			name := regexp.MustCompile(`[^A-Za-z0-9]`).ReplaceAllString(filename, "_")
			return &distEntry{
				Filename: filename,
				URL:      url,
				SHA256:   sha256,
				Name:     name,
			}, nil
		}
	}
	return nil, fmt.Errorf("no distfile found for %s", filename)
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
        "{{ . }}",
        {{- end }}
    ],
    visibility = ["//visibility:public"],
)
`))

func generateBuild(buildPath string, fn func(f *os.File) error) error {
	f, err := os.Create(buildPath)
	if err != nil {
		return err
	}
	defer f.Close()

	return fn(f)
}

func updateRepositories(bzlPath string, dists []*distEntry) error {
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

func packageNamesToLabels(names []string, overlayDirs []string) ([]string, error) {
	var labels []string
	for _, name := range names {
		label, err := packageNameToLabel(name, overlayDirs)
		if err != nil {
			return nil, err
		}
		labels = append(labels, label)
	}
	return labels, nil
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

		var pkgInfoMap map[string]*packageInfo
		if packageInfoPath != "" {
			b, err := os.ReadFile(packageInfoPath)
			if err != nil {
				return err
			}
			if err := json.Unmarshal(b, &pkgInfoMap); err != nil {
				return err
			}

			// Rewrite package names to labels.
			for _, pkgInfo := range pkgInfoMap {
				pkgInfo.BuildDeps, err = packageNamesToLabels(pkgInfo.BuildDeps, overlayDirs)
				if err != nil {
					return err
				}
				pkgInfo.RuntimeDeps, err = packageNamesToLabels(pkgInfo.RuntimeDeps, overlayDirs)
				if err != nil {
					return err
				}
			}
		}

		cachedDists := make(map[string]*distEntry)
		distJSONPath := filepath.Join(workspaceDir, "bazel/data/distfiles.json")
		if b, err := os.ReadFile(distJSONPath); err == nil {
			if err := json.Unmarshal(b, &cachedDists); err != nil {
				return fmt.Errorf("%s: %w", distJSONPath, err)
			}
		}

		for _, overlayDir := range overlayDirs {
			var overlayDists []*distEntry

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
				v := strings.Split(ebuildDir, "/")
				category := v[len(v)-2]
				packageName := v[len(v)-1]
				buildPath := filepath.Join(ebuildDir, "BUILD.bazel")

				pkgInfo := pkgInfoMap[fmt.Sprintf("%s/%s", category, packageName)]
				_, blocked := blockedPackages[packageName]
				if pkgInfo == nil || blocked {
					if err := os.Remove(buildPath); err != nil && !errors.Is(err, os.ErrNotExist) {
						return err
					}
					continue
				}

				log.Printf("Generating: %s", buildPath)

				if err := func() error {
					// Find the best version.
					fis, err := os.ReadDir(ebuildDir)
					if err != nil {
						return err
					}

					var bestName string
					var bestVer *version.Version

					var allVersions []*version.Version

					for _, fi := range fis {
						name := fi.Name()
						if !strings.HasSuffix(name, ebuildExt) {
							continue
						}
						// TODO: Remove this hack.
						if name == "ncurses-5.9-r99.ebuild" ||
							// There are no 9999 lacros distfiles
							name == "chromeos-lacros-9999.ebuild" {
							continue
						}
						_, ver, err := version.ExtractSuffix(strings.TrimSuffix(name, ebuildExt))
						if err != nil {
							return fmt.Errorf("%s: %w", filepath.Join(ebuildDir, name), err)
						}
						if bestVer == nil || bestVer.Compare(ver) < 0 {
							bestName = name
							bestVer = ver
						}
						if info, err := fi.Info(); err == nil {
							// Skip the revbump symlinks
							if info.Mode()&fs.ModeSymlink == 0 {
								allVersions = append(allVersions, ver)
							}
						} else {
							return err
						}
					}
					if bestName == "" {
						return errors.New("no ebuild found")
					}

					if category != "dev-rust" {
						// We only want to generate 1 ebuild for non rust packages
						allVersions = []*version.Version{bestVer}
					} else {
						// If we have cros-workon rust ebuilds only use those.
						// HACK: We should use the eclass to determine if it's a cros-workon
						for _, ver := range allVersions {
							if ver.Major() == "9999" {
								allVersions = []*version.Version{ver}
								break
							}
						}
					}

					// Read Manifest to get a list of distfiles.
					manifest, err := os.ReadFile(filepath.Join(ebuildDir, "Manifest"))
					if err != nil && !errors.Is(err, os.ErrNotExist) {
						return err
					}

					var dists []*distEntry
					for _, line := range strings.Split(string(manifest), "\n") {
						fields := strings.Fields(line)
						if len(fields) < 2 || fields[0] != "DIST" {
							continue
						}
						filename, err := url.PathUnescape(fields[1])
						if err != nil {
							return err
						}
						if strings.Contains(filename, "/") {
							// This is likely a Go dep, skip for now.
							continue
						}

						// Check the cache first.
						if dist, ok := cachedDists[filename]; ok {
							if dist == nil {
								log.Printf("WARNING: unable to locate distfile %s: negative-cached", filename)
								continue
							}
							dists = append(dists, dist)
							continue
						}

						log.Printf("  locating distfile %s...", filename)
						dist, err := locateDistFile(filename)
						if err != nil {
							log.Printf("WARNING: unable to locate distfile %s: %v", filename, err)
						} else {
							dists = append(dists, dist)
						}

						cachedDists[filename] = dist

						// Update cache.
						b, err := json.MarshalIndent(cachedDists, "", "  ")
						if err != nil {
							return err
						}
						if err := os.WriteFile(distJSONPath, b, 0o644); err != nil {
							return err
						}
					}

					if err := generateBuild(buildPath, func(f *os.File) error {
						if err := buildHeaderTemplate.Execute(f, nil); err != nil {
							return err
						}

						var targetNames []string

						for _, ver := range allVersions {
							ebuildName := fmt.Sprintf("%s-%s.ebuild", packageName, ver)

							var localPackageName string
							if len(allVersions) == 1 {
								localPackageName = packageName
							} else {
								localPackageName = fmt.Sprintf("%s-%s", packageName, ver)
								targetNames = append(targetNames, ":"+localPackageName)
							}

							ebuild := &ebuildInfo{
								EBuildName:  ebuildName,
								PackageName: localPackageName,
								Category:    category,
								Dists:       dists,
								PackageInfo: pkgInfo,
							}

							if err := ebuildTemplate.Execute(f, ebuild); err != nil {
								return err
							}
						}

						if len(targetNames) > 0 {
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
						return err
					}

					overlayDists = append(overlayDists, dists...)
					return nil
				}(); err != nil {
					log.Printf("WARNING: Failed to generate %s: %v", buildPath, err)
				}
			}

			sort.Slice(overlayDists, func(i, j int) bool {
				return overlayDists[i].Name < overlayDists[j].Name
			})
			if err := updateRepositories(filepath.Join(overlayDir, "repositories.bzl"), overlayDists); err != nil {
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
