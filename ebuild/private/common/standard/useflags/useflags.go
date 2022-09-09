// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package useflags

import (
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/makevars"
	"cros.local/bazel/ebuild/private/common/standard/profile"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Context struct {
	makeConfSource string
	profileSource  string
	overrides      *profile.Overrides
}

func NewContext(makeConfVars makevars.Vars, profile *profile.ParsedProfile) *Context {
	return &Context{
		makeConfSource: makeConfVars["USE"],
		profileSource:  profile.Vars()["USE"],
		overrides:      profile.Overrides(),
	}
}

func (c *Context) ComputeForPackage(packageName string, ver *version.Version, ebuildVars makevars.Vars) map[string]struct{} {
	po := c.overrides.ForPackage(packageName, ver)
	combined := strings.Join([]string{parseIUSE(ebuildVars["IUSE"]), c.profileSource, c.makeConfSource, po.Use()}, " ")
	finalized := makevars.FinalizeIncrementalVar(combined)

	use := make(map[string]struct{})
	for _, token := range strings.Fields(finalized) {
		use[token] = struct{}{}
	}
	return use
}

func parseIUSE(s string) string {
	var use []string
	for _, u := range strings.Fields(s) {
		if strings.HasPrefix(u, "+") {
			use = append(use, u[1:])
		}
	}
	return strings.Join(use, " ")
}
