// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package ptracearch

import "golang.org/x/sys/unix"

type Regs = unix.PtraceRegsAmd64

func GetRegs(tid int, regs *Regs) error {
	return unix.PtraceGetRegsAmd64(tid, regs)
}

func SetRegs(tid int, regs *Regs) error {
	return unix.PtraceSetRegsAmd64(tid, regs)
}
