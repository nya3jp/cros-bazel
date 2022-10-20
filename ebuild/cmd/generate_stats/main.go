// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"fmt"
	"log"
	"sort"

	"os"
	"path/filepath"
	"strings"

	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/common/depdata"
)

func buildArtifactPath(info *depdata.PackageInfo, workspaceDir string) string {
	return filepath.Join(workspaceDir, "bazel-bin", strings.Replace(info.EBuildPath, ".ebuild", ".tbz2", 1))
}

func transitiveBuildDeps(pkgInfoMap depdata.PackageInfoMap, startInfo *depdata.PackageInfo, workspaceDir string) []string {
	var queue []string
	seen := make(map[string]struct{})

	for _, label := range startInfo.BuildDeps {
		queue = append(queue, label)
		seen[label] = struct{}{}
	}

	for len(queue) > 0 {
		current := queue[0]
		queue = queue[1:]
		info := pkgInfoMap[current]
		for _, next := range info.RuntimeDeps {
			if _, ok := seen[next]; !ok {
				queue = append(queue, next)
				seen[next] = struct{}{}
			}
		}
	}

	var labels []string
	for label := range seen {
		labels = append(labels, label)
	}
	sort.Strings(labels)
	return labels
}

func printStats(pkgInfoMap depdata.PackageInfoMap, workspaceDir string) error {
	goods := 0
	bads := 0
	skips := 0
	statusMap := make(map[string]string)

	for label, info := range pkgInfoMap {
		if _, err := os.Stat(buildArtifactPath(info, workspaceDir)); err == nil {
			goods++
			statusMap[label] = "✅"
		} else {
			skip := false
			for _, l := range transitiveBuildDeps(pkgInfoMap, info, workspaceDir) {
				if _, err := os.Stat(buildArtifactPath(pkgInfoMap[l], workspaceDir)); err != nil {
					skip = true
					break
				}
			}
			if skip {
				skips++
				statusMap[label] = "⌛"
			} else {
				bads++
				statusMap[label] = "🔥"
			}
		}
	}

	var sortedLabels []string
	for label := range pkgInfoMap {
		sortedLabels = append(sortedLabels, label)
	}
	sort.Strings(sortedLabels)

	for _, label := range sortedLabels {
		fmt.Printf("%s,%s\n", label, statusMap[label])
	}
	log.Printf("Good: %d, Bad: %d, Skipped: %d", goods, bads, skips)
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
		workspaceDir := os.Getenv("BUILD_WORKSPACE_DIRECTORY")

		packageInfoPath := c.String(flagPackageInfoFile.Name)
		if packageInfoPath == "" {
			packageInfoPath = filepath.Join(workspaceDir, "bazel/data/deps.json")
		}

		pkgInfoMap, err := depdata.Load(packageInfoPath)
		if err != nil {
			return err
		}

		if err := printStats(pkgInfoMap, workspaceDir); err != nil {
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
