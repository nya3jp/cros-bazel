// Copyright 2022 The Chromium OS Authors. All rights reserved.
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
	"third_party/portage-stable",
	"third_party/chromiumos-overlay",
	"overlays/overlay-arm64-generic",
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
}

type distEntry struct {
	Filename string `json:"filename"`
	URL      string `json:"url"`
	SHA256   string `json:"sha256"`
	Name     string `json:"name"`
}

type packageInfo struct {
	BuildDeps   []string `json:"buildDeps"`
	RuntimeDeps []string `json:"runtimeDeps"`
}

type ebuildInfo struct {
	EBuildName  string
	PackageName string
	Category    string
	Dists       []*distEntry
	PackageInfo *packageInfo
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
		url := distBaseURL + filename
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

var repositoriesTemplate = template.Must(template.New("").Parse(`# Copyright 2022 The Chromium OS Authors. All rights reserved.
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

var buildTemplate = template.Must(template.New("").Parse(`# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild:defs.bzl", "ebuild")

ebuild(
    name = "{{ .PackageName }}",
    src = "{{ .EBuildName }}",
    category = "{{ .Category }}",
    distfiles = {
        {{- range .Dists }}
        "@{{ .Name }}//file": "{{ .Filename }}",
        {{- end }}
    },
    {{- if .PackageInfo.BuildDeps }}
    build_target_deps = [
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

func generateBuild(buildPath string, ebuild *ebuildInfo) error {
	f, err := os.Create(buildPath)
	if err != nil {
		return err
	}
	defer f.Close()

	return buildTemplate.Execute(f, ebuild)
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
		if _, err := os.Stat(filepath.Join(overlayDir, name)); err != nil {
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

		for _, overlayDir := range overlayDirs {
			var knownDists []*distEntry
			distJSONPath := filepath.Join(overlayDir, "distfiles.json")
			if b, err := os.ReadFile(distJSONPath); err == nil {
				if err := json.Unmarshal(b, &knownDists); err != nil {
					return fmt.Errorf("%s: %w", distJSONPath, err)
				}
			}

			knownDistMap := make(map[string]*distEntry)
			for _, dist := range knownDists {
				knownDistMap[dist.Filename] = dist
			}

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

				if _, ok := blockedPackages[packageName]; ok {
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
					for _, fi := range fis {
						name := fi.Name()
						if !strings.HasSuffix(name, ebuildExt) {
							continue
						}
						// TODO: Remove this hack.
						if name == "ncurses-5.9-r99.ebuild" {
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
					}
					if bestName == "" {
						return errors.New("no ebuild found")
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
						if dist, ok := knownDistMap[filename]; ok {
							dists = append(dists, dist)
							continue
						}
						log.Printf("  locating distfile %s...", filename)
						dist, err := locateDistFile(filename)
						if err != nil {
							log.Printf("WARNING: unable to locate distfile %s: %v", filename, err)
							continue
						}
						knownDists = append(knownDists, dist)
						knownDistMap[filename] = dist

						// Update distfiles.json.
						b, err := json.Marshal(knownDists)
						if err != nil {
							return err
						}
						if err := os.WriteFile(distJSONPath, b, 0o644); err != nil {
							return err
						}
					}

					pkgInfo := pkgInfoMap[fmt.Sprintf("%s/%s", category, packageName)]
					if pkgInfo == nil {
						pkgInfo = &packageInfo{}
					}

					ebuild := &ebuildInfo{
						EBuildName:  bestName,
						PackageName: packageName,
						Category:    category,
						Dists:       dists,
						PackageInfo: pkgInfo,
					}
					if err := generateBuild(buildPath, ebuild); err != nil {
						return err
					}
					return nil
				}(); err != nil {
					log.Printf("WARNING: Failed to generate %s: %v", buildPath, err)
				}
			}

			sort.Slice(knownDists, func(i, j int) bool {
				return knownDists[i].Name < knownDists[j].Name
			})
			if err := updateRepositories(filepath.Join(overlayDir, "repositories.bzl"), knownDists); err != nil {
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
