// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package dependency

import (
	"fmt"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/naming"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

type TargetPackage struct {
	Name     string
	Version  *version.Version
	MainSlot string
	Uses     map[string]bool
}

type VersionOperator string

const (
	OpNone         VersionOperator = ""
	OpLessEqual    VersionOperator = "<="
	OpLess         VersionOperator = "<"
	OpExactEqual   VersionOperator = "="
	OpRoughEqual   VersionOperator = "~"
	OpGreaterEqual VersionOperator = ">="
	OpGreater      VersionOperator = ">"
)

var versionOperators = []VersionOperator{
	OpLessEqual,
	OpLess,
	OpExactEqual,
	OpRoughEqual,
	OpGreaterEqual,
	OpGreater,
}

type Atom struct {
	name     string
	op       VersionOperator
	ver      *version.Version
	wildcard bool
	slotDep  string
	useDeps  []*UseDependency
}

func NewAtom(packageName string, op VersionOperator, ver *version.Version, wildcard bool, slotDep string, useDeps []*UseDependency) *Atom {
	return &Atom{
		name:     packageName,
		op:       op,
		ver:      ver,
		wildcard: wildcard,
		slotDep:  slotDep,
		useDeps:  useDeps,
	}
}

func NewSimpleAtom(packageName string) *Atom {
	return NewAtom(packageName, OpNone, nil, false, "", nil)
}

func ParseAtom(atomStr string) (*Atom, error) {
	rest := atomStr

	var useDeps []*UseDependency
	if strings.HasSuffix(rest, "]") {
		v := strings.SplitN(strings.TrimSuffix(rest, "]"), "[", 2)
		if len(v) != 2 {
			return nil, fmt.Errorf("%s: invalid use dependencies", atomStr)
		}
		for _, u := range strings.Split(v[1], ",") {
			useDeps = append(useDeps, &UseDependency{raw: u})
		}
		rest = v[0]
	}

	slotDep := ""
	if v := strings.SplitN(rest, ":", 2); len(v) == 2 {
		slotDep = v[1]
		rest = v[0]
	}

	op, rest, err := trimVersionOperator(rest)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", atomStr, err)
	}

	var ver *version.Version
	wildcard := false
	if op != OpNone {
		if op == OpExactEqual && strings.HasSuffix(rest, "*") {
			rest = strings.TrimSuffix(rest, "*")
			wildcard = true
		}

		rest, ver, err = version.ExtractSuffix(rest)
		if err != nil {
			return nil, fmt.Errorf("%s: %w", atomStr, err)
		}
	}

	if err := naming.CheckCategoryAndPackage(rest); err != nil {
		return nil, fmt.Errorf("%s: %w", atomStr, err)
	}

	return &Atom{
		name:     rest,
		op:       op,
		ver:      ver,
		wildcard: wildcard,
		slotDep:  slotDep,
		useDeps:  useDeps,
	}, nil
}

func trimVersionOperator(s string) (op VersionOperator, rest string, err error) {
	for _, op := range versionOperators {
		if strings.HasPrefix(s, string(op)) {
			return op, strings.TrimPrefix(s, string(op)), nil
		}
	}
	return OpNone, s, nil
}

func (a *Atom) PackageName() string              { return a.name }
func (a *Atom) PackageCategory() string          { return strings.Split(a.name, "/")[0] }
func (a *Atom) VersionOperator() VersionOperator { return a.op }
func (a *Atom) Version() *version.Version        { return a.ver }
func (a *Atom) Wildcard() bool                   { return a.wildcard }
func (a *Atom) SlotDep() string                  { return a.slotDep }
func (a *Atom) UseDeps() []*UseDependency        { return a.useDeps }

func (a *Atom) Match(t *TargetPackage) bool {
	if t.Name != a.name {
		return false
	}
	if a.slotDep != "" && a.slotDep != "*" && a.slotDep != "=" {
		// TODO: Consider subslot dependencies as well.
		wantMainSlot := strings.Split(strings.TrimSuffix(a.slotDep, "="), "/")[0]
		if t.MainSlot != wantMainSlot {
			return false
		}
	}
	// TODO: Consider use dependency.
	switch a.op {
	case OpNone:
		return true
	case OpLess:
		return t.Version.Compare(a.ver) < 0
	case OpLessEqual:
		return t.Version.Compare(a.ver) <= 0
	case OpExactEqual:
		if a.wildcard {
			return t.Version.HasPrefix(a.ver)
		}
		return t.Version.Compare(a.ver) == 0
	case OpRoughEqual:
		return t.Version.DropRevision().Compare(a.ver) == 0
	case OpGreaterEqual:
		return t.Version.Compare(a.ver) >= 0
	case OpGreater:
		return t.Version.Compare(a.ver) > 0
	default:
		panic(fmt.Sprintf("unknown version operator %s", string(a.op)))
	}
}

func (a *Atom) String() string {
	s := string(a.op) + a.name
	if a.op != OpNone {
		s += "-" + a.ver.String()
		if a.wildcard {
			s += "*"
		}
	}
	if a.slotDep != "" {
		s += ":" + a.slotDep
	}
	if len(a.useDeps) > 0 {
		var substrings []string
		for _, useDep := range a.useDeps {
			substrings = append(substrings, useDep.String())
		}
		s += fmt.Sprintf("[%s]", strings.Join(substrings, ","))
	}
	return s
}

type UseDependency struct {
	raw string
}

func (u *UseDependency) String() string {
	return u.raw
}
