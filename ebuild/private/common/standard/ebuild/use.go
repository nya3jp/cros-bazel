// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package ebuild

import (
	"sort"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/makevars"
)

func computeUseFlags(pkg *config.Package, config config.Source, metadata Metadata) (map[string]bool, error) {
	env := make(makevars.Vars)
	varsList, err := config.EvalPackageVars(pkg, env)
	if err != nil {
		return nil, err
	}

	varsList = append([]makevars.Vars{
		{"USE": parseIUSEToUSE(metadata["IUSE"])},
	}, varsList...)

	vars := makevars.Finalize(varsList)

	masks := make(map[string]bool)
	forces := make(map[string]bool)
	if err := config.UseMasksAndForces(pkg, masks, forces); err != nil {
		return nil, err
	}

	// TODO: Hide USE flags not declared in IUSE.
	// For now, we don't filter them as we don't parse USE_EXPAND yet and
	// thus don't know the effective IUSE.
	uses := make(map[string]bool)
	for _, u := range strings.Fields(vars["USE"]) {
		if masks[u] {
			continue
		}
		uses[u] = true
	}
	for u, ok := range forces {
		if !ok || masks[u] {
			continue
		}
		uses[u] = true
	}

	return uses, nil
}

func parseIUSEToUSE(iuse string) string {
	var uses []string
	for _, use := range strings.Fields(iuse) {
		if strings.HasPrefix(use, "+") {
			uses = append(uses, strings.TrimPrefix(use, "+"))
		}
	}
	sort.Strings(uses)
	return strings.Join(uses, " ")
}
