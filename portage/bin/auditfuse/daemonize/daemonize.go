// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package daemonize

import (
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"os/signal"
	"runtime"
	"syscall"

	"golang.org/x/sys/unix"
)

const envName = "AUDITFUSE_DAEMONIZE_STEP"

// Start starts daemonizing the current process.
//
// This function may start the current executable as a subprocess.
//
// If err is not nil, the current process should exit abnormally immediately.
// If exit is true, the current process should exit normally immediately.
// If exit is false, the current process will become a daemon. Perform
// necessary setups and call Stop once it's done. Note that, at this point,
// stdin/stdout are connected to /dev/null and stderr is connected to a pipe
// where you can possibly write error messages.
func Start() (exit bool, err error) {
	step := os.Getenv(envName)
	switch step {
	case "":
		exe, err := os.Executable()
		if err != nil {
			return false, fmt.Errorf("failed to locate the current executable: %w", err)
		}

		r, w, err := os.Pipe()
		if err != nil {
			return false, err
		}

		cmd := exec.Command(exe)
		cmd.Args = os.Args
		cmd.Env = []string{
			fmt.Sprintf("%s=2", envName),
		}
		cmd.Stderr = w
		cmd.SysProcAttr = &syscall.SysProcAttr{
			Setsid: true,
		}
		if err := cmd.Start(); err != nil {
			return false, err
		}

		w.Close()

		c, err := io.Copy(os.Stderr, r)
		if err != nil {
			return false, err
		}

		// Exit abnormally if the daemon process wrote something to stderr.
		if c > 0 {
			return false, errors.New("daemon wrote something to stderr")
		}

		// Exit the current process now.
		return true, nil

	case "2":
		exe, err := os.Executable()
		if err != nil {
			return false, fmt.Errorf("failed to locate the current executable: %w", err)
		}

		// Lock the current thread for pthread_sigmask.
		runtime.LockOSThread()

		unix.Umask(0o022)
		signal.Reset()
		if err := unix.PthreadSigmask(unix.SIG_SETMASK, nil, &unix.Sigset_t{}); err != nil {
			return false, fmt.Errorf("pthread_sigmask: %w", err)
		}

		cmd := exec.Command(exe)
		cmd.Args = os.Args
		cmd.Env = []string{
			fmt.Sprintf("%s=3", envName),
		}
		cmd.Stderr = os.Stderr
		if err := cmd.Start(); err != nil {
			return false, err
		}

		// Exit the current process now.
		return true, nil

	case "3":
		return false, nil

	default:
		return false, fmt.Errorf("invalid %s value: %s", envName, step)
	}
}

// Finish finishes daemonizing the current process.
//
// Call Start before calling Finish. Finish closes stderr (precisely, it is
// connected to stderr) so you cannot report errors once it returns. Finish
// changes the current working directory to /, so make sure to resolve relative
// file paths beforehand.
func Finish() {
	os.Chdir("/")
	// Dup stdout to stderr, which should redirect it to /dev/null.
	// This should trigger the first process to exit.
	unix.Dup2(1, 2)
}
