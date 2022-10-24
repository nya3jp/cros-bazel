// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package hooks

import (
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/cmd/fakefs/logging"
	"cros.local/bazel/ebuild/private/cmd/fakefs/ptracearch"
	"cros.local/bazel/ebuild/private/cmd/fakefs/tracee"
	"cros.local/bazel/ebuild/private/cmd/fakefs/tracer"
)

type Hook struct{}

var _ tracee.Hook = Hook{}
var _ tracer.Hook = Hook{}

func (Hook) SyscallList() []int {
	return []int{
		// stat
		unix.SYS_STAT,
		unix.SYS_FSTAT,
		unix.SYS_LSTAT,
		unix.SYS_STATX,
		unix.SYS_NEWFSTATAT,
		// chmod
		unix.SYS_CHMOD,
		unix.SYS_FCHMOD,
		unix.SYS_FCHMODAT,
		// chown
		unix.SYS_CHOWN,
		unix.SYS_LCHOWN,
		unix.SYS_FCHOWN,
		unix.SYS_FCHOWNAT,
	}
}

func (Hook) Syscall(tid int, regs *ptracearch.Regs, logger *logging.ThreadLogger) func(regs *ptracearch.Regs) {
	// TODO: Simulate syscalls instead of just blocking them.

	switch regs.Orig_rax {
	case unix.SYS_CHMOD, unix.SYS_FCHMOD, unix.SYS_FCHMODAT, unix.SYS_CHOWN, unix.SYS_LCHOWN, unix.SYS_FCHOWN, unix.SYS_FCHOWNAT:
		return func(regs *ptracearch.Regs) {
			// Pretend to succeed.
			regs.Rax = 0
			_ = ptracearch.SetRegs(tid, regs)
		}

	default:
		return nil
	}
}

func New() Hook {
	return Hook{}
}
