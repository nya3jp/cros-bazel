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

	"cros.local/ebuild/private/common/portage/version"
)

const ebuildExt = ".ebuild"

var distBaseURLs = []string{
	"https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/",
	"https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/",
}

func findWorkspace(dir string) (string, error) {
	dir, err := filepath.Abs(dir)
	if err != nil {
		return "", err
	}

	for {
		path := filepath.Join(dir, "WORKSPACE")
		if _, err := os.Stat(path); err == nil {
			return path, nil
		}
		dir = filepath.Dir(dir)
		if dir == "/" {
			return "", fmt.Errorf("WORKSPACE not found above %s", dir)
		}
	}
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

var repositoriesTemplate = template.Must(template.New("").Parse(`load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

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

var buildTemplate = template.Must(template.New("").Parse(`load("//ebuild:defs.bzl", "ebuild")

ebuild(
    name = "{{ .PackageName }}",
    src = "{{ .EBuildName }}",
    category = "{{ .Category }}",
    distfiles = {
{{- range .Dists }}
        "@{{ .Name }}//file": "{{ .Filename }}",
{{- end }}
    },
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
	f, err := os.Create(bzlPath)
	if err != nil {
		return err
	}
	defer f.Close()

	return repositoriesTemplate.Execute(f, dists)
}

var app = &cli.App{
	Flags: []cli.Flag{},
	Action: func(c *cli.Context) error {
		if len(c.Args()) != 1 {
			return errors.New("need exactly one directory")
		}
		topDir := c.Args()[0]

		var knownDists []*distEntry
		distJSONPath := filepath.Join(topDir, "distfiles.json")
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
		if err := filepath.WalkDir(topDir, func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if !d.IsDir() {
				return nil
			}
			if _, err := os.Stat(filepath.Join(path, "Manifest")); err == nil {
				ebuildDirs = append(ebuildDirs, path)
				return fs.SkipDir
			}
			return nil
		}); err != nil {
			return err
		}

		for _, ebuildDir := range ebuildDirs {
			v := strings.Split(ebuildDir, "/")
			category := v[len(v)-2]
			packageName := v[len(v)-1]

			if category == "dev-embedded" && packageName == "jlink" {
				// Skip restricted packages.
				continue
			}

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
				return fmt.Errorf("%s: no ebuild found", ebuildDir)
			}

			// Read Manifest to get a list of distfiles.
			manifest, err := os.ReadFile(filepath.Join(ebuildDir, "Manifest"))
			if err != nil {
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
				dist, err := locateDistFile(filename)
				if err != nil {
					return fmt.Errorf("%s: unable to locate distfile %s: %w", ebuildDir, filename, err)
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

			ebuild := &ebuildInfo{
				EBuildName:  bestName,
				PackageName: packageName,
				Category:    category,
				Dists:       dists,
			}
			buildPath := filepath.Join(ebuildDir, "BUILD")
			log.Printf("Generating: %s", buildPath)
			if err := generateBuild(buildPath, ebuild); err != nil {
				return err
			}
		}

		sort.Slice(knownDists, func(i, j int) bool {
			return knownDists[i].Name < knownDists[j].Name
		})
		if err := updateRepositories(filepath.Join(topDir, "repositories.bzl"), knownDists); err != nil {
			return err
		}
		return nil
	},
}

func main() {
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
