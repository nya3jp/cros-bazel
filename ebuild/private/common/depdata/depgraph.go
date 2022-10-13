// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package depdata

type PackageInfoMap map[string][]*PackageInfo

func (g PackageInfoMap) FixupForJSON() {
	for _, infos := range g {
		for _, info := range infos {
			info.FixupForJSON()
		}
	}
}

type PackageInfo struct {
	Version     string              `json:"version"`
	BuildDeps   []string            `json:"buildDeps"`
	LocalSrc    []string            `json:"localSrc"`
	RuntimeDeps []string            `json:"runtimeDeps"`
	SrcUris     map[string]*URIInfo `json:"srcUris"`
}

func (pi *PackageInfo) FixupForJSON() {
	if pi.BuildDeps == nil {
		pi.BuildDeps = []string{}
	}
	if pi.LocalSrc == nil {
		pi.LocalSrc = []string{}
	}
	if pi.RuntimeDeps == nil {
		pi.RuntimeDeps = []string{}
	}
	if pi.SrcUris == nil {
		pi.SrcUris = make(map[string]*URIInfo)
	}
}

type URIInfo struct {
	Uris      []string `json:"uris"`
	Size      int      `json:"size"`
	Integrity string   `json:"integrity"`
	// TODO: Remove when we can use integrity
	SHA256 string `json:"SHA256"`
	SHA512 string `json:"SHA512"`
}
