// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package tracee

import (
	"fmt"
	"os"
	"os/exec"
	"unsafe"

	seccomp "github.com/elastic/go-seccomp-bpf"
	"golang.org/x/net/bpf"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/cmd/fakefs/hooks"
)

func setUpSeccompBPF() error {
	program, err := hooks.SeccompBPF()
	if err != nil {
		return err
	}

	rawProgram, err := bpf.Assemble(program)
	if err != nil {
		return err
	}

	var filters []unix.SockFilter
	for _, inst := range rawProgram {
		filters = append(filters, unix.SockFilter{
			Code: inst.Op,
			Jt:   inst.Jt,
			Jf:   inst.Jf,
			K:    inst.K,
		})
	}
	filterProgram := &unix.SockFprog{
		Len:    uint16(len(filters)),
		Filter: &filters[0],
	}

	if err := seccomp.SetNoNewPrivs(); err != nil {
		return fmt.Errorf("prctl(PR_SET_NO_NEW_PRIVS): %w", err)
	}

	const seccompSetModeFilter = 0x1
	if _, _, errno := unix.Syscall(unix.SYS_SECCOMP, seccompSetModeFilter, uintptr(seccomp.FilterFlagTSync), uintptr(unsafe.Pointer(filterProgram))); errno != 0 {
		return fmt.Errorf("seccomp(SECCOMP_SET_MODE_FILTER): %w", errno)
	}

	return nil
}

func Run(args []string) error {
	if err := setUpSeccompBPF(); err != nil {
		return err
	}

	// Stop the process to give the tracee a chance to call PTRACE_SEIZE.
	// Note that we don't use PTRACE_TRACEME since PTRACE_SEIZE has improved
	// behavior.
	pid := unix.Getpid()
	unix.Kill(pid, unix.SIGSTOP)

	path, err := exec.LookPath(args[0])
	if err != nil {
		return err
	}

	return unix.Exec(path, args, os.Environ())
}
