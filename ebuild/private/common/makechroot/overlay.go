// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makechroot

import (
	"fmt"
	"path/filepath"
	"strings"
)

type OverlayInfo struct {
	MountDir  string
	ImagePath string
}

func ParseOverlaySpecs(specs []string) ([]OverlayInfo, error) {
	var overlays []OverlayInfo
	for _, spec := range specs {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid overlay spec: %s", spec)
		}
		mountDir := v[0]
		if mountDir != "/" {
			mountDir = strings.TrimSuffix(mountDir, "/")
		}
		overlays = append(overlays, OverlayInfo{
			MountDir:  mountDir,
			ImagePath: v[1],
		})
	}
	return overlays, nil
}

type BindMount struct {
	MountPath string
	Source    string
}

func ParseBindMountSpec(specs []string) ([]BindMount, error) {
	// Bind-mounts work the same as overlay, so we can just use their parsing
	// mechanism.
	overlays, err := ParseOverlaySpecs(specs)
	if err != nil {
		return nil, fmt.Errorf("invalid bind-mount: %v", err)
	}

	var mounts []BindMount
	for _, overlay := range overlays {
		path, err := filepath.Abs(overlay.ImagePath)
		if err != nil {
			return nil, err
		}

		mounts = append(mounts, BindMount{MountPath: overlay.MountDir, Source: path})
	}
	return mounts, nil
}