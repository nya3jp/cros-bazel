// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package tracee

import (
	"os"
	"os/exec"

	seccomp "github.com/elastic/go-seccomp-bpf"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/cmd/fakefs/hooks"
	"cros.local/bazel/ebuild/private/cmd/fakefs/syscallabi"
)

func Run(args []string) error {
	var names []string
	for _, nr := range hooks.SyscallList() {
		names = append(names, syscallabi.Name(nr))
	}

	// Set seccomp filter.
	filter := seccomp.Filter{
		NoNewPrivs: true,
		Flag:       seccomp.FilterFlagTSync,
		Policy: seccomp.Policy{
			DefaultAction: seccomp.ActionAllow,
			Syscalls: []seccomp.SyscallGroup{{
				Action: seccomp.ActionTrace,
				Names:  names,
			}},
		},
	}
	if err := seccomp.LoadFilter(filter); err != nil {
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
