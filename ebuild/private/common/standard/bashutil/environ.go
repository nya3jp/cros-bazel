// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package bashutil

import (
	"mvdan.cc/sh/v3/expand"
)

// Environ implements string-only expand.Environ on map[string]string.
type Environ map[string]string

var _ expand.Environ = Environ{}

func (e Environ) Get(name string) expand.Variable {
	value, ok := e[name]
	if !ok {
		return expand.Variable{}
	}
	return expand.Variable{
		Local: true,
		Kind:  expand.String,
		Str:   value,
	}
}

func (e Environ) Each(f func(name string, v expand.Variable) bool) {
	for name := range e {
		if !f(name, e.Get(name)) {
			return
		}
	}
}

func (e Environ) Set(name string, v expand.Variable) {
	if v.Kind != expand.String {
		// Silently ignore non-string variables.
		return
	}
	e[name] = v.Str
}
