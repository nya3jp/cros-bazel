// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makechroot

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/tar"
)

type BindMount struct {
	MountPath string
	Source    string
}

func ParseBindMountSpec(specs []string) ([]BindMount, error) {
	var mounts []BindMount
	for _, spec := range specs {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid bind-mount spec: %s", spec)
		}
		mounts = append(mounts, BindMount{
			MountPath: v[0],
			Source:    v[1],
		})
	}
	return mounts, nil
}

type LayerType int

const (
	LayerDir LayerType = iota
	LayerSquashfs
	LayerTar
)

func DetectLayerType(layerPath string) (LayerType, error) {
	layerPath, err := filepath.EvalSymlinks(layerPath)
	if err != nil {
		return -1, err
	}

	fileInfo, err := os.Stat(layerPath)
	if err != nil {
		return -1, err
	}

	if fileInfo.IsDir() {
		return LayerDir, nil
	} else if strings.HasSuffix(layerPath, ".squashfs") {
		return LayerSquashfs, nil
	} else if tar.IsTar(layerPath) {
		return LayerTar, nil
	} else {
		return -1, fmt.Errorf("unsupported file type: %s", layerPath)
	}
}
