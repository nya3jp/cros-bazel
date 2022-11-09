// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package cliutil

import (
	"errors"
	"fmt"
	"log"
	"os"
)

// ExitCode is an error value that instructs the program to exit with a certain
// exit code.
// The program must call cliutil.Exit in its main function to handle ExitCode
// errors.
type ExitCode int

func (e ExitCode) Error() string {
	return fmt.Sprintf("exit code %d", int(e))
}

// Exit terminates the program by calling os.Exit. If err contains ExitCode,
// it calls os.Exit with the specified exit code. Otherwise it prints an error
// message and calls os.Exit(1).
//
// The function never returns. Beware that deferred function calls are not
// triggered.
func Exit(err error) {
	var code ExitCode
	if errors.As(err, &code) {
		os.Exit(int(code))
	}
	if err != nil {
		log.Printf("FATAL: %v", err)
		os.Exit(1)
	}
	os.Exit(0)
}
