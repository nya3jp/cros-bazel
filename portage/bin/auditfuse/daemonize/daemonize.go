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
// necessary setups and call Stop once it's done.
//
// Daemonization fails if a daemon process exits without calling Finish.
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
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.ExtraFiles = []*os.File{
			w, // Set the writer end of the pipe to FD 3.
		}
		cmd.SysProcAttr = &syscall.SysProcAttr{
			Setsid: true,
		}
		if err := cmd.Start(); err != nil {
			return false, err
		}

		w.Close()

		b, _ := io.ReadAll(r)
		r.Close()

		if string(b) != "ok" {
			return false, errors.New("failed to start daemon")
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
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.ExtraFiles = []*os.File{
			os.NewFile(3, "<pipe>"),
		}
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
// Call Finish in the daemon process once initialization is complete. It
// communicates the initialization success to the first process, and let it exit
// normally.
// Finish changes the current working directory to /, so make sure to resolve
// relative file paths beforehand.
func Finish() {
	os.Chdir("/")

	// Writing "ok" and closing the pipe tells success to the first process.
	w := os.NewFile(3, "<pipe>")
	io.WriteString(w, "ok")
	w.Close()
}
