// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package naming

import (
	"errors"
	"fmt"
	"regexp"
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/version"
)

var categoryRe = regexp.MustCompile(`^[A-Za-z0-9_][A-Za-z0-9+_.-]*$`)

func CheckCategory(s string) error {
	if !categoryRe.MatchString(s) {
		return fmt.Errorf("invalid category name %q", s)
	}
	return nil
}

var packageRe = regexp.MustCompile(`^[A-Za-z0-9_][A-Za-z0-9+_-]*$`)

func CheckPackage(s string) error {
	if _, _, err := version.ExtractSuffix(s); err == nil {
		return errors.New("invalid package name: version-like suffix")
	}
	if !packageRe.MatchString(s) {
		return errors.New("invalid package name")
	}
	return nil
}

func CheckCategoryAndPackage(s string) error {
	v := strings.Split(s, "/")
	if len(v) != 2 {
		return errors.New("invalid package name")
	}
	if err := CheckCategory(v[0]); err != nil {
		return err
	}
	return CheckPackage(v[1])
}
