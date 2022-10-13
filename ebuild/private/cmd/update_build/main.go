// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"crypto/sha256"
	"crypto/sha512"
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

	"cros.local/bazel/ebuild/private/common/depdata"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"github.com/urfave/cli"
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
	"https://storage.googleapis.com/chromium-nodejs/16.13.0/",
}

var allowedHosts = map[string]struct{}{
	"commondatastorage.googleapis.com": {},
	"storage.googleapis.com":           {},
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

type ebuildInfo struct {
	EBuildName  string
	PackageName string
	Category    string
	Dists       []*distEntry
	PackageInfo *depdata.PackageInfo
}

type packageGroup struct {
	PackageName string
	Packages    []string
}

func getSHA(url string) (string, string, error) {
	log.Printf("Trying: %s", url)
	res, err := http.Get(url)
	if err != nil {
		return "", "", err
	}
	defer res.Body.Close()

	if res.StatusCode/100 != 2 {
		return "", "", fmt.Errorf("http status %d", res.StatusCode)
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return "", "", err
	}

	sha256Hash := sha256.Sum256(body)
	sha512Hash := sha512.Sum512(body)

	// Need to unsize the slice
	sha256HashHex := hex.EncodeToString(sha256Hash[:])
	sha512HashHex := hex.EncodeToString(sha512Hash[:])

	return sha256HashHex, sha512HashHex, nil
}

func locateDistFile(filename string, info *depdata.URIInfo) (*distEntry, error) {
	// Once we use bazel 6.0 we can just set integrity value on the
	// http_file.
	var sha256 string
	var sha512 string

	goodUri, ok := manualDistfileMap[filename]
	if !ok {
		for _, uri := range info.Uris {
			parsedUri, err := url.ParseRequestURI(uri)
			if err != nil {
				return nil, err
			}

			if parsedUri.Scheme == "gs" {
				log.Printf("Got gs: %s", parsedUri)
				parsedUri.Scheme = "https"
				parsedUri.Path = filepath.Join(parsedUri.Host, parsedUri.Path)
				parsedUri.Host = "commondatastorage.googleapis.com"
				uri = parsedUri.String()
			}

			if _, ok := allowedHosts[parsedUri.Host]; !ok {
				continue
			}

			goodUri = uri
			sha256 = info.SHA256
			break
		}
	}

	if goodUri != "" && sha256 == "" {
		// We don't have a SHA256 in the Manifest, Download the file and use
		// the SHA512 in the Manifest to verify the file, then compute the
		// SHA256 from the downloaded file.
		var err error
		sha256, sha512, err = getSHA(goodUri)
		if err != nil {
			return nil, err
		}

		if info.SHA512 != "" && sha512 != info.SHA512 {
			return nil, fmt.Errorf("SHA512 doesn't match for file %s: %s != %s", filename, info.SHA512, sha512)
		}
	} else if goodUri == "" {
		for _, distBaseURL := range distBaseURLs {
			url := distBaseURL + filename

			var err error
			sha256, sha512, err = getSHA(url)
			if err != nil {
				continue
			}

			if info.SHA512 != "" && sha512 != info.SHA512 {
				return nil, fmt.Errorf("SHA512 doesn't match for file %s: %s != %s", filename, info.SHA512, sha512)
			}

			goodUri = url
			break
		}
	}

	log.Printf("Found: %s w/ sha256 %s", goodUri, sha256)

	if goodUri != "" && sha256 != "" {
		name := regexp.MustCompile(`[^A-Za-z0-9]`).ReplaceAllString(filename, "_")
		return &distEntry{
			Filename: filename,
			URL:      goodUri,
			SHA256:   sha256,
			Name:     name,
		}, nil
	} else {
		return nil, fmt.Errorf("no distfile found for %s", filename)
	}
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

func getDistEntries(cachedDists map[string]*distEntry, distJSONPath string, pkgInfo *depdata.PackageInfo) ([]*distEntry, error) {
	var dists []*distEntry
	for filename, srcInfo := range pkgInfo.SrcUris {
		// Check the cache first.
		if dist, ok := cachedDists[filename]; ok {
			if dist != nil {
				dists = append(dists, dist)
				continue
			}
		}

		log.Printf("  locating distfile %s...", filename)
		dist, err := locateDistFile(filename, srcInfo)
		if err != nil {
			log.Fatalf("WARNING: unable to locate distfile %s: %v", filename, err)
			// TODO: Do we want to support negative caching?
			continue
		} else {
			dists = append(dists, dist)
		}

		cachedDists[filename] = dist

		// Update cache.
		b, err := json.MarshalIndent(cachedDists, "", "  ")
		if err != nil {
			return nil, err
		}
		if err := os.WriteFile(distJSONPath, b, 0o644); err != nil {
			return nil, err
		}
	}

	sort.Slice(dists, func(i, j int) bool {
		return dists[i].Name < dists[j].Name
	})

	return dists, nil
}

func uniqueDists(list []*distEntry) []*distEntry {
	sort.Slice(list, func(i, j int) bool {
		return list[i].Name < list[j].Name
	})

	var uniqueList []*distEntry
	var previousItem *distEntry
	for _, dep := range list {
		if previousItem != nil && dep.Name == previousItem.Name {
			continue
		}
		previousItem = dep
		uniqueList = append(uniqueList, dep)
	}

	return uniqueList
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

				pkgInfos := pkgInfoMap[fmt.Sprintf("%s/%s", category, packageName)]
				_, blocked := blockedPackages[packageName]
				if pkgInfos == nil || blocked {
					if err := os.Remove(buildPath); err != nil && !errors.Is(err, os.ErrNotExist) {
						return err
					}
					continue
				}

				log.Printf("Generating: %s", buildPath)

				if err := func() error {

					var ebuildInfos []ebuildInfo

					// Keep the previous behavior where rust pkgs were sorted ascending
					sort.Slice(pkgInfos, func(i, j int) bool {
						return pkgInfos[i].Version < pkgInfos[j].Version
					})

					for _, pkgInfo := range pkgInfos {
						dists, err := getDistEntries(cachedDists, distJSONPath, pkgInfo)
						if err != nil {
							return err
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
						overlayDists = append(overlayDists, dists...)
					}

					if err := generateBuild(buildPath, func(f *os.File) error {
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
						return err
					}
					return nil
				}(); err != nil {
					log.Printf("WARNING: Failed to generate %s: %v", buildPath, err)
				}
			}

			overlayDists = uniqueDists(overlayDists)

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
