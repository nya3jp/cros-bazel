// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makechroot

import (
	"fmt"
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
