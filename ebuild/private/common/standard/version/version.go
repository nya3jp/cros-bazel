// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package version

import (
	"errors"
	"fmt"
	"regexp"
	"strings"
)

// Version represents a version of a package.
type Version struct {
	Main     []string
	Letter   string
	Suffixes []*Suffix
	Revision string
}

func (v *Version) Copy() *Version {
	copy := *v
	for i, suffix := range copy.Suffixes {
		copy.Suffixes[i] = suffix.Copy()
	}
	return &copy
}

func (v *Version) ImplicitRevision() string {
	if v.Revision == "" {
		return "0"
	}
	return v.Revision
}

func (v *Version) DropRevision() *Version {
	copy := v.Copy()
	copy.Revision = ""
	return copy
}

func (v *Version) Major() string {
	if len(v.Main) > 0 {
		return v.Main[0]
	}

	return "0"
}

func (v *Version) String() string {
	var w strings.Builder
	for i, n := range v.Main {
		if i > 0 {
			w.WriteString(".")
		}
		w.WriteString(n)
	}
	fmt.Fprintf(&w, v.Letter)
	for _, s := range v.Suffixes {
		w.WriteString(string(s.Label))
		if s.Number != "" {
			w.WriteString(s.Number)
		}
	}
	if v.Revision != "" {
		w.WriteString("-r")
		w.WriteString(v.Revision)
	}
	return w.String()
}

func (v *Version) Compare(o *Version) int {
	// Compare main.
	if cmp := compareStringInt(v.Main[0], o.Main[0]); cmp != 0 {
		return cmp
	}
	for i := 1; i < len(v.Main) && i < len(o.Main); i++ {
		a := v.Main[i]
		b := o.Main[i]
		if strings.HasPrefix(a, "0") || strings.HasPrefix(b, "0") {
			a0 := strings.TrimRight(a, "0")
			b0 := strings.TrimRight(b, "0")
			if cmp := strings.Compare(a0, b0); cmp != 0 {
				return cmp
			}
		} else {
			if cmp := compareStringInt(a, b); cmp != 0 {
				return cmp
			}
		}
	}
	if len(v.Main) != len(o.Main) {
		if len(v.Main) < len(o.Main) {
			return -1
		}
		return 1
	}

	// Compare letter.
	if cmp := strings.Compare(v.Letter, o.Letter); cmp != 0 {
		return cmp
	}

	// Compare suffixes.
	for i := 0; i < len(v.Suffixes) && i < len(o.Suffixes); i++ {
		if cmp := v.Suffixes[i].Compare(o.Suffixes[i]); cmp != 0 {
			return cmp
		}
	}
	if len(v.Suffixes) > len(o.Suffixes) {
		if v.Suffixes[len(v.Suffixes)-1].Label == SuffixP {
			return 1
		}
		return -1
	}
	if len(v.Suffixes) < len(o.Suffixes) {
		if o.Suffixes[len(o.Suffixes)-1].Label == SuffixP {
			return -1
		}
		return 1
	}

	// Compare revision.
	return compareStringInt(v.Revision, o.Revision)
}

func (v *Version) HasPrefix(prefix *Version) bool {
	copy := v.Copy()

	func() {
		if prefix.Revision != "" {
			return
		}
		copy.Revision = ""

		if len(copy.Suffixes) > len(prefix.Suffixes) {
			copy.Suffixes = copy.Suffixes[:len(prefix.Suffixes)]
		}
		if len(prefix.Suffixes) > 0 {
			return
		}

		if prefix.Letter != "" {
			return
		}
		copy.Letter = ""

		if len(copy.Main) > len(prefix.Main) {
			copy.Main = copy.Main[:len(prefix.Main)]
		}
	}()

	return copy.Compare(prefix) == 0
}

type Suffix struct {
	Label  SuffixLabel
	Number string
}

func (s *Suffix) Copy() *Suffix {
	copy := *s
	return &copy
}

func (s *Suffix) Compare(o *Suffix) int {
	if cmp := s.Label.Compare(o.Label); cmp != 0 {
		return cmp
	}
	return compareStringInt(s.Number, o.Number)
}

type SuffixLabel string

const (
	SuffixAlpha SuffixLabel = "_alpha"
	SuffixBeta  SuffixLabel = "_beta"
	SuffixPre   SuffixLabel = "_pre"
	SuffixRC    SuffixLabel = "_rc"
	SuffixP     SuffixLabel = "_p"
)

func (l SuffixLabel) Compare(o SuffixLabel) int {
	lp := l.priority()
	op := o.priority()
	if lp < op {
		return -1
	}
	if lp > op {
		return 1
	}
	return 0
}

func (l SuffixLabel) priority() int {
	switch l {
	case SuffixAlpha:
		return 1
	case SuffixBeta:
		return 2
	case SuffixPre:
		return 3
	case SuffixRC:
		return 4
	case SuffixP:
		return 5
	default:
		panic(fmt.Sprintf("unknown version suffix label %s", string(l)))
	}
}

func compareStringInt(a, b string) int {
	a = strings.TrimLeft(a, "0")
	b = strings.TrimLeft(b, "0")
	if len(a) != len(b) {
		if len(a) < len(b) {
			return -1
		}
		return 1
	}
	return strings.Compare(a, b)
}

var (
	mainRe     = regexp.MustCompile(`([0-9]+(?:\.[0-9]+)*)$`)
	letterRe   = regexp.MustCompile(`([a-z])$`)
	suffixRe   = regexp.MustCompile(`(_(?:alpha|beta|pre|rc|p))(\d*)$`)
	revisionRe = regexp.MustCompile(`-r(\d+)$`)
)

// ExtractSuffix trims a Portage package version suffix from a string.
//
// Examples:
//
//	"net-misc/curl-7.78.0-r1" => ("net-misc/curl-", "7.78.0-r1")
//	"curl-7.78.0-r1" => ("curl-", "7.78.0-r1")
//	"7.78.0-r1" => ("", "7.78.0-r1")
func ExtractSuffix(s string) (prefix string, ver *Version, err error) {
	revision := ""
	if m := revisionRe.FindStringSubmatch(s); m != nil {
		revision = m[1]
		s = s[:len(s)-len(m[0])]
	}

	var suffixes []*Suffix
	for {
		m := suffixRe.FindStringSubmatch(s)
		if m == nil {
			break
		}

		suffixes = append([]*Suffix{{
			Label:  SuffixLabel(m[1]),
			Number: m[2],
		}}, suffixes...)
		s = s[:len(s)-len(m[0])]
	}

	var letter string
	if m := letterRe.FindStringSubmatch(s); m != nil {
		letter = m[1]
		s = s[:len(s)-len(m[0])]
	}

	m := mainRe.FindStringSubmatch(s)
	if m == nil {
		return "", nil, errors.New("invalid version: main part")
	}
	main := strings.Split(m[1], ".")
	s = s[:len(s)-len(m[0])]

	v := &Version{
		Main:     main,
		Letter:   letter,
		Suffixes: suffixes,
		Revision: revision,
	}
	return s, v, nil
}

// Parse parses a Portage package version string.
func Parse(s string) (*Version, error) {
	rest, ver, err := ExtractSuffix(s)
	if err != nil {
		return nil, err
	}
	if rest != "" {
		return nil, errors.New("invalid version: excess prefix")
	}
	return ver, nil
}
