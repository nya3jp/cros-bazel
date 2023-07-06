// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package syscallabi

import "cros.local/bazel/portage/bin/fakefs/ptracearch"

// See man 2 syscall for the system call calling convention.

func ParseStatArgs(regs *ptracearch.Regs) StatArgs {
	return StatArgs{uintptr(regs.Rdi), uintptr(regs.Rsi)}
}

func ParseLstatArgs(regs *ptracearch.Regs) LstatArgs {
	return LstatArgs{uintptr(regs.Rdi), uintptr(regs.Rsi)}
}

func ParseFstatArgs(regs *ptracearch.Regs) FstatArgs {
	return FstatArgs{int(int32(regs.Rdi)), uintptr(regs.Rsi)}
}

func ParseNewfstatatArgs(regs *ptracearch.Regs) NewfstatatArgs {
	return NewfstatatArgs{int(int32(regs.Rdi)), uintptr(regs.Rsi), uintptr(regs.Rdx), int(int32(regs.R10))}
}

func ParseStatxArgs(regs *ptracearch.Regs) StatxArgs {
	return StatxArgs{int(int32(regs.Rdi)), uintptr(regs.Rsi), int(int32(regs.Rdx)), int(int32(regs.R10)), uintptr(regs.R8)}
}

func ParseChownArgs(regs *ptracearch.Regs) ChownArgs {
	return ChownArgs{uintptr(regs.Rdi), int(int32(regs.Rsi)), int(int32(regs.Rdx))}
}

func ParseLchownArgs(regs *ptracearch.Regs) LchownArgs {
	return LchownArgs{uintptr(regs.Rdi), int(int32(regs.Rsi)), int(int32(regs.Rdx))}
}

func ParseFchownArgs(regs *ptracearch.Regs) FchownArgs {
	return FchownArgs{int(int32(regs.Rdi)), int(int32(regs.Rsi)), int(int32(regs.Rdx))}
}

func ParseFchownatArgs(regs *ptracearch.Regs) FchownatArgs {
	return FchownatArgs{int(int32(regs.Rdi)), uintptr(regs.Rsi), int(int32(regs.Rdx)), int(int32(regs.R10)), int(int32(regs.R8))}
}

func ParseListxattrArgs(regs *ptracearch.Regs) ListxattrArgs {
	return ListxattrArgs{uintptr(regs.Rdi), uintptr(regs.Rsi), int(regs.Rdx)}
}

func ParseLlistxattrArgs(regs *ptracearch.Regs) LlistxattrArgs {
	return LlistxattrArgs{uintptr(regs.Rdi), uintptr(regs.Rsi), int(regs.Rdx)}
}

func ParseFlistxattrArgs(regs *ptracearch.Regs) FlistxattrArgs {
	return FlistxattrArgs{int(int32(regs.Rdi)), uintptr(regs.Rsi), int(regs.Rdx)}
}
