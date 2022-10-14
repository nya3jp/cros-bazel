// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This is an implementation of ver_test required in EAPI 7+.

package main

import (
	"errors"
	"fmt"
	"log"
	"os"

	"cros.local/bazel/ebuild/private/common/standard/version"
)

func main() {
	if err := func() error {
		args := os.Args[1:]
		if len(args) == 2 {
			args = append([]string{os.Getenv("PVR")}, args...)
		}
		if len(args) != 3 {
			return errors.New("needs exactly 3 arguments")
		}

		lhs, err := version.Parse(args[0])
		if err != nil {
			return fmt.Errorf("failed to parse lhs version: %w", err)
		}

		rhs, err := version.Parse(args[2])
		if err != nil {
			return fmt.Errorf("failed to parse rhs version: %w", err)
		}

		cmp := lhs.Compare(rhs)

		op := args[1]
		var ok bool
		switch op {
		case "-eq":
			ok = cmp == 0
		case "-ne":
			ok = cmp != 0
		case "-gt":
			ok = cmp > 0
		case "-ge":
			ok = cmp >= 0
		case "-lt":
			ok = cmp < 0
		case "-le":
			ok = cmp <= 0
		default:
			return fmt.Errorf("unsupported operator: %s", op)
		}
		if !ok {
			os.Exit(1)
		}
		return nil
	}(); err != nil {
		log.Printf("ERROR: %v", err)
		os.Exit(2)
	}
}
