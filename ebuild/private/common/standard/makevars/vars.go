// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package makevars

import (
	"fmt"
	"io"
	"sort"
	"strings"

	"github.com/alessio/shellescape"
)

type Vars map[string]string

func (v Vars) Copy() Vars {
	u := make(Vars)
	for key, value := range v {
		u[key] = value
	}
	return u
}

func (v Vars) CopyNoIncrementalVars() Vars {
	u := make(Vars)
	for key, value := range v {
		if isIncrementalVar(key) {
			continue
		}
		u[key] = value
	}
	return u
}

func (v Vars) Environ() []string {
	names := make([]string, 0, len(v))
	for name := range v {
		names = append(names, name)
	}
	sort.Strings(names)

	env := make([]string, 0, len(v))
	for _, name := range names {
		env = append(env, fmt.Sprintf("%s=%s", name, v[name]))
	}
	return env
}

func (v Vars) Dump(w io.Writer) {
	names := make([]string, 0, len(v))
	for name := range v {
		names = append(names, name)
	}
	sort.Strings(names)

	for _, name := range names {
		fmt.Fprintf(w, "%s=%s\n", shellescape.Quote(name), shellescape.Quote(v[name]))
	}
}

func (v Vars) GetAsList(key string) []string {
	return strings.Fields(v[key])
}

func (v Vars) GetAsSet(key string) map[string]struct{} {
	set := make(map[string]struct{})
	for _, e := range v.GetAsList(key) {
		set[e] = struct{}{}
	}
	return set
}

func (v Vars) Merge(nv Vars) {
	for key, newValue := range nv {
		if isIncrementalVar(key) {
			v[key] = v[key] + " " + newValue
		} else {
			v[key] = newValue
		}
	}
}

func Merge(varsList ...Vars) Vars {
	merged := make(Vars)
	for _, vars := range varsList {
		merged.Merge(vars)
	}
	return merged
}

var incrementalVarNames = map[string]struct{}{
	"USE":                   {},
	"USE_EXPAND":            {},
	"USE_EXPAND_HIDDEN":     {},
	"CONFIG_PROTECT":        {},
	"CONFIG_PROTECT_MASK":   {},
	"IUSE_IMPLICIT":         {},
	"USE_EXPAND_IMPLICIT":   {},
	"USE_EXPAND_UNPREFIXED": {},
	"ENV_UNSET":             {},
	// USE_EXPAND_VALUES_* are handled separately.
}

func isIncrementalVar(name string) bool {
	// TODO: Treat variables mentioned in USE_EXPAND and its family as incremental.
	if _, ok := incrementalVarNames[name]; ok {
		return true
	}
	if strings.HasPrefix(name, "USE_EXPAND_VALUES_") {
		return true
	}
	return false
}

func FinalizeIncrementalVar(value string) string {
	tokenSet := make(map[string]struct{})

	for _, token := range strings.Fields(value) {
		if token == "-*" {
			tokenSet = make(map[string]struct{})
			continue
		}
		if strings.HasPrefix(token, "-") {
			delete(tokenSet, token[1:])
			continue
		}
		tokenSet[token] = struct{}{}
	}

	var tokens []string
	for token := range tokenSet {
		tokens = append(tokens, token)
	}
	sort.Strings(tokens)
	return strings.Join(tokens, " ")
}
