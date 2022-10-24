// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package tracer

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
	"unsafe"

	"golang.org/x/sys/unix"
)

func exitCode(ws unix.WaitStatus) int {
	if ws.Signaled() {
		return 128 + int(ws.Signal())
	}
	if ws.Exited() {
		return ws.ExitStatus()
	}
	return 0
}

func ptraceSeize(tid int, options uint) error {
	if _, _, errno := unix.RawSyscall6(
		unix.SYS_PTRACE,
		unix.PTRACE_SEIZE,
		uintptr(tid),
		0,
		uintptr(options),
		0,
		0); errno != 0 {
		return errno
	}
	return nil
}

func ptraceListen(tid int) error {
	if _, _, errno := unix.RawSyscall6(
		unix.SYS_PTRACE,
		unix.PTRACE_LISTEN,
		uintptr(tid),
		0,
		0,
		0,
		0); errno != 0 {
		return errno
	}
	return nil
}

func ptraceGetSigInfo(tid int, si *unix.Siginfo) error {
	if _, _, errno := unix.RawSyscall6(
		unix.SYS_PTRACE,
		unix.PTRACE_GETSIGINFO,
		uintptr(tid),
		0,
		uintptr(unsafe.Pointer(si)),
		0,
		0); errno != 0 {
		return errno
	}
	return nil
}

func lookupPidByTid(tid int) (pid int, err error) {
	path := fmt.Sprintf("/proc/%d/status", tid)
	f, err := os.Open(path)
	if err != nil {
		return 0, err
	}
	defer f.Close()

	sc := bufio.NewScanner(f)
	for sc.Scan() {
		line := sc.Text()
		key, value, ok := strings.Cut(line, ":")
		if ok && key == "Tgid" {
			pid, err := strconv.Atoi(strings.TrimSpace(value))
			if err != nil {
				return 0, fmt.Errorf("failed to parse %s: %w", path, err)
			}
			return pid, nil
		}
		if strings.HasPrefix(line, "Tgid:") {
			strings.Cut(line, ":")
		}
	}
	return 0, fmt.Errorf("failed to parse %s: tgid not found", path)
}
