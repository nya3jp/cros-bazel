// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package packages

type Stability string

const (
	StabilityStable  Stability = "stable"
	StabilityTesting Stability = "testing"
	StabilityBroken  Stability = "broken"
)

func SelectByStability(pkgs []*Package) []*Package {
	if len(pkgs) == 0 {
		return nil
	}

	candidates := make(map[Stability][]*Package)
	for _, pkg := range pkgs {
		s := pkg.Stability()
		candidates[s] = append(candidates[s], pkg)
	}

	if stable := candidates[StabilityStable]; len(stable) > 0 {
		return stable
	}
	if testing := candidates[StabilityTesting]; len(testing) > 0 {
		return testing
	}
	return nil
}
