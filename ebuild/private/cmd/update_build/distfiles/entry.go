// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package distfiles

import (
	"sort"
)

type Entry struct {
	Filename string `json:"filename"`
	URL      string `json:"url"`
	SHA256   string `json:"sha256"`
	Name     string `json:"name"`
}

func Unique(list []*Entry) []*Entry {
	sort.Slice(list, func(i, j int) bool {
		return list[i].Name < list[j].Name
	})

	var uniqueList []*Entry
	var previousItem *Entry
	for _, dep := range list {
		if previousItem != nil && dep.Name == previousItem.Name {
			continue
		}
		previousItem = dep
		uniqueList = append(uniqueList, dep)
	}

	return uniqueList
}
