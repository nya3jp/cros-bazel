// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package depdata

import (
	"encoding/json"
	"os"
)

type PackageInfoMap map[string]*PackageInfo

func (g PackageInfoMap) fixupForJSON() {
	for _, info := range g {
		info.fixupForJSON()
	}
}

type PackageInfo struct {
	Name        string              `json:"name"`
	MainSlot    string              `json:"mainSlot"`
	EBuildPath  string              `json:"ebuildPath"`
	Version     string              `json:"version"`
	BuildDeps   []string            `json:"buildDeps"`
	LocalSrc    []string            `json:"localSrc"`
	RuntimeDeps []string            `json:"runtimeDeps"`
	SrcUris     map[string]*URIInfo `json:"srcUris"`
	PostDeps    []string            `json:"postDeps,omitempty"`
}

func (pi *PackageInfo) fixupForJSON() {
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

func Load(path string) (PackageInfoMap, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	var infoMap PackageInfoMap
	if err := json.NewDecoder(f).Decode(&infoMap); err != nil {
		return nil, err
	}
	return infoMap, nil
}

func Save(path string, infoMap PackageInfoMap) error {
	infoMap.fixupForJSON()

	f, err := os.Create(path)
	if err != nil {
		return err
	}

	enc := json.NewEncoder(f)
	enc.SetEscapeHTML(false)
	enc.SetIndent("", "  ")
	if err := enc.Encode(infoMap); err != nil {
		_ = f.Close()
		return err
	}
	return f.Close()
}
