// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package golden

import (
	"encoding/json"
	"os"
)

type Package struct {
	Name    string          `json:"name"`
	Version string          `json:"version"`
	Uses    map[string]bool `json:"uses"`
}

func Load(path string) ([]*Package, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	var pkgs []*Package
	if err := json.NewDecoder(f).Decode(&pkgs); err != nil {
		return nil, err
	}
	return pkgs, nil
}

func Save(path string, pkgs []*Package) error {
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()

	enc := json.NewEncoder(f)
	enc.SetEscapeHTML(false)
	enc.SetIndent("", "  ")
	return enc.Encode(pkgs)
}
