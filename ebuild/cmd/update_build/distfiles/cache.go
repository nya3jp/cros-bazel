// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package distfiles

import (
	"encoding/json"
	"fmt"
	"os"
)

type Cache struct {
	path  string
	dists map[string]*Entry
}

func NewCache(path string) (*Cache, error) {
	dists := make(map[string]*Entry)

	if b, err := os.ReadFile(path); err == nil {
		if err := json.Unmarshal(b, &dists); err != nil {
			return nil, fmt.Errorf("%s: %w", path, err)
		}
	}

	return &Cache{
		path:  path,
		dists: dists,
	}, nil
}

func (dc *Cache) Get(filename string) (*Entry, bool) {
	dist, ok := dc.dists[filename]
	return dist, ok
}

func (dc *Cache) Set(filename string, dist *Entry) error {
	dc.dists[filename] = dist

	b, err := json.MarshalIndent(dc.dists, "", "  ")
	if err != nil {
		return err
	}
	if err := os.WriteFile(dc.path, b, 0o644); err != nil {
		return err
	}
	return nil
}
