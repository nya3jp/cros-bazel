package main

import (
	"errors"
	"fmt"
	"io/fs"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"text/template"

	"github.com/urfave/cli"

	"cros.local/ebuild/private/common/fileutil"
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
	Name     string
	URL      string
	Filename string
}

type ebuildInfo struct {
	EBuildName  string
	PackageName string
	Category    string
	Dists       []*distEntry
}

func locateDistFile(filename string) (*distEntry, error) {
	for _, distBaseURL := range distBaseURLs {
		url := distBaseURL + filename
		if res, err := http.Head(url); err == nil {
			res.Body.Close()
			name := regexp.MustCompile(`[^A-Za-z0-9]`).ReplaceAllString(filename, "_")
			return &distEntry{
				Name:     name,
				Filename: filename,
				URL:      url,
			}, nil
		}
	}
	return nil, fmt.Errorf("no distfile found for %s", filename)
}

var workspaceTemplate = template.Must(template.New("").Parse(`
http_file(
	name = "{{ .Name }}",
	downloaded_file_path = "{{ .Filename }}",
	urls = ["{{ .URL }}"],
)
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

func updateWorkspace(workspacePath string, dists []*distEntry) error {
	if len(dists) == 0 {
		return nil
	}

	f, err := os.OpenFile(workspacePath, os.O_WRONLY|os.O_APPEND, 0o644)
	if err != nil {
		return err
	}
	defer f.Close()

	for _, dist := range dists {
		if err := workspaceTemplate.Execute(f, dist); err != nil {
			return err
		}
	}
	return nil
}

var flagOverlay = &cli.StringFlag{
	Name:     "overlay",
	Required: true,
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagOverlay,
	},
	Action: func(c *cli.Context) error {
		overlayDir := c.String(flagOverlay.Name)

		workspacePath, err := findWorkspace(overlayDir)
		if err != nil {
			return err
		}

		startDirs := []string(c.Args())
		if len(startDirs) == 0 {
			return errors.New("need arguments")
		}

		var srcDirs []string
		for _, startDir := range startDirs {
			if err := filepath.WalkDir(startDir, func(path string, d fs.DirEntry, err error) error {
				if err != nil {
					return err
				}
				if !d.IsDir() {
					return nil
				}
				if _, err := os.Stat(filepath.Join(path, "Manifest")); err == nil {
					srcDirs = append(srcDirs, path)
					return fs.SkipDir
				}
				return nil
			}); err != nil {
				return err
			}
		}

		for _, srcDir := range srcDirs {
			v := strings.Split(srcDir, "/")
			category := v[len(v)-2]
			packageName := v[len(v)-1]
			dstDir := filepath.Join(overlayDir, category, packageName)
			buildPath := filepath.Join(dstDir, "BUILD")

			if _, err := os.Stat(dstDir); errors.Is(err, os.ErrNotExist) {
				if err := os.MkdirAll(filepath.Dir(dstDir), 0o755); err != nil {
					return err
				}
				if err := fileutil.CopyDir(srcDir, dstDir); err != nil {
					return err
				}
			}

			if _, err := os.Stat(buildPath); err == nil {
				continue
			}

			// Find the best version.
			fis, err := os.ReadDir(dstDir)
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
					return fmt.Errorf("%s: %w", filepath.Join(dstDir, name), err)
				}
				if bestVer == nil || bestVer.Compare(ver) < 0 {
					bestName = name
					bestVer = ver
				}
			}
			if bestName == "" {
				return fmt.Errorf("%s: no ebuild found", dstDir)
			}

			// Read Manifest to get a list of distfiles.
			manifest, err := os.ReadFile(filepath.Join(dstDir, "Manifest"))
			if err != nil {
				return err
			}

			var dists []*distEntry
			for _, line := range strings.Split(string(manifest), "\n") {
				fields := strings.Fields(line)
				if len(fields) < 2 || fields[0] != "DIST" {
					continue
				}
				filename := fields[1]
				dist, err := locateDistFile(filename)
				if err != nil {
					return fmt.Errorf("%s: unable to locate distfile %s: %w", srcDir, filename, err)
				}
				dists = append(dists, dist)
			}

			ebuild := &ebuildInfo{
				EBuildName:  bestName,
				PackageName: packageName,
				Category:    category,
				Dists:       dists,
			}
			log.Printf("Generating: %s", buildPath)
			if err := generateBuild(buildPath, ebuild); err != nil {
				return err
			}
			if err := updateWorkspace(workspacePath, dists); err != nil {
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
