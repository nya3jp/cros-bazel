// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package exit

import (
	"errors"
	"fmt"
	"log"
	"os"
)

// Code is an error value that instructs the program to exit with a certain
// exit code.
// The program must call Exit in its main function to handle Code errors.
type Code int

func (e Code) Error() string {
	return fmt.Sprintf("exit code %d", int(e))
}

// Exit terminates the program by calling os.Exit. If err contains Code,
// it calls os.Exit with the specified exit code. Otherwise it prints an error
// message and calls os.Exit(1).
//
// The function never returns. Beware that deferred function calls are not
// triggered.
func Exit(err error) {
	var code Code
	if errors.As(err, &code) {
		os.Exit(int(code))
	}
	if err != nil {
		log.Printf("FATAL: %v", err)
		os.Exit(1)
	}
	os.Exit(0)
}
