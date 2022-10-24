// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package hooks

import (
	"fmt"
	"math"
	"reflect"
	"unsafe"

	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/cmd/fakefs/fsop"
	"cros.local/bazel/ebuild/private/cmd/fakefs/logging"
	"cros.local/bazel/ebuild/private/cmd/fakefs/ptracearch"
	"cros.local/bazel/ebuild/private/cmd/fakefs/syscallabi"
	"cros.local/bazel/ebuild/private/cmd/fakefs/tracee"
	"cros.local/bazel/ebuild/private/cmd/fakefs/tracer"
)

func readCString(tid int, ptr uintptr) (string, error) {
	// Use process_vm_readv(2) instead of ptrace(2) with PTRACE_PEEKDATA
	// for much better efficiency.
	var str []byte

	// Always assume that the page size is 4096 bytes.
	// Even if huge pages are enabled, the page size should be multiple of
	// 4096 bytes, so it's fine for our purpose.
	const pageSize = 4096
	buf := make([]byte, pageSize)

	for {
		nextSize := pageSize - (ptr % pageSize)
		localIov := []unix.Iovec{{
			Base: (*byte)((unsafe.Pointer)((*reflect.SliceHeader)((unsafe.Pointer)(&buf)).Data)),
			Len:  uint64(nextSize),
		}}
		remoteIov := []unix.RemoteIovec{{
			Base: ptr,
			Len:  int(nextSize),
		}}

		readSize, err := unix.ProcessVMReadv(tid, localIov, remoteIov, 0)
		if err != nil {
			return "", err
		}

		for _, b := range buf[:readSize] {
			if b == 0 {
				return string(str), nil
			}
			str = append(str, b)
		}
		ptr += uintptr(readSize)
	}
}

func writeData[T any](tid int, ptr uintptr, data *T) error {
	// Use process_vm_writev(2) instead of ptrace(2) with PTRACE_POKEDATA
	// for much better efficiency.
	size := unsafe.Sizeof(*data)
	localIov := []unix.Iovec{{
		Base: (*byte)(unsafe.Pointer(data)),
		Len:  uint64(size),
	}}
	remoteIov := []unix.RemoteIovec{{
		Base: ptr,
		Len:  int(size),
	}}
	_, err := unix.ProcessVMWritev(tid, localIov, remoteIov, 0)
	return err
}

func dirfdPath(tid int, dfd int) string {
	if dfd == unix.AT_FDCWD {
		return fmt.Sprintf("/proc/%d/cwd", tid)
	}
	return fmt.Sprintf("/proc/%d/fd/%d", tid, dfd)
}

func blockSyscall(tid int, regs *ptracearch.Regs, logger *logging.ThreadLogger, err error) func(regs *ptracearch.Regs) {
	errno, ok := err.(unix.Errno)
	if err != nil && !ok {
		logger.Printf("! %s: %v", syscallabi.Name(int(regs.Orig_rax)), err)
		errno = unix.ENOTRECOVERABLE
	}

	// Set the syscall number to -1, which should always fail with ENOSYS.
	regs.Orig_rax = math.MaxUint64
	_ = ptracearch.SetRegs(tid, regs)

	return func(regs *ptracearch.Regs) {
		// Override the system call return value with errno.
		regs.Rax = -uint64(errno)
		_ = ptracearch.SetRegs(tid, regs)
	}
}

// openat opens a file with arguments intercepted for a tracee thread.
// It returns a file descriptor opened with O_PATH.
func openat(tid int, dfd int, filename string, flags int) (fd int, err error) {
	path := dirfdPath(tid, dfd)
	dirfd, err := unix.Open(path, unix.O_PATH|unix.O_CLOEXEC, 0)
	if err != nil {
		return -1, err
	}

	if filename == "" && flags&unix.AT_EMPTY_PATH != 0 {
		return dirfd, nil
	}

	oflags := unix.O_PATH | unix.O_CLOEXEC
	if flags&unix.AT_SYMLINK_NOFOLLOW != 0 {
		oflags |= unix.O_NOFOLLOW
	}

	fd, err = unix.Openat(dirfd, filename, oflags, 0)
	_ = unix.Close(dirfd)
	if err != nil {
		return -1, err
	}
	return fd, nil
}

func simulateFstatat(tid int, regs *ptracearch.Regs, logger *logging.ThreadLogger, dfd int, filename string, statbuf uintptr, flags int) func(regs *ptracearch.Regs) {
	return blockSyscall(tid, regs, logger, func() error {
		fd, err := openat(tid, dfd, filename, flags)
		if err != nil {
			return err
		}
		defer unix.Close(fd)

		var stat unix.Stat_t
		if err := fsop.Fstat(fd, &stat); err != nil {
			return err
		}

		return writeData(tid, statbuf, &stat)
	}())
}

func simulateStatx(tid int, regs *ptracearch.Regs, logger *logging.ThreadLogger, dfd int, filename string, flags int, mask int, statxbuf uintptr) func(regs *ptracearch.Regs) {
	return blockSyscall(tid, regs, logger, func() error {
		fd, err := openat(tid, dfd, filename, flags)
		if err != nil {
			return err
		}
		defer unix.Close(fd)

		var statx unix.Statx_t
		if err := fsop.Fstatx(fd, mask, &statx); err != nil {
			return err
		}

		return writeData(tid, statxbuf, &statx)
	}())
}

func simulateFchownat(tid int, regs *ptracearch.Regs, logger *logging.ThreadLogger, dfd int, filename string, user int, group int, flags int) func(regs *ptracearch.Regs) {
	return blockSyscall(tid, regs, logger, func() error {
		fd, err := openat(tid, dfd, filename, flags)
		if err != nil {
			return err
		}
		defer unix.Close(fd)

		return fsop.Fchown(fd, user, group)
	}())
}

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
	switch regs.Orig_rax {
	case unix.SYS_STAT:
		args := syscallabi.ParseStatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Printf("# stat(%q)", filename)
		return simulateFstatat(tid, regs, logger, unix.AT_FDCWD, filename, args.Statbuf, unix.AT_SYMLINK_FOLLOW)

	case unix.SYS_LSTAT:
		args := syscallabi.ParseLstatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Printf("# lstat(%q)", filename)
		return simulateFstatat(tid, regs, logger, unix.AT_FDCWD, filename, args.Statbuf, unix.AT_SYMLINK_NOFOLLOW)

	case unix.SYS_FSTAT:
		args := syscallabi.ParseFstatArgs(regs)
		logger.Printf("# fstat(%q)", args.Fd)
		return simulateFstatat(tid, regs, logger, args.Fd, "", args.Statbuf, unix.AT_EMPTY_PATH)

	case unix.SYS_NEWFSTATAT:
		args := syscallabi.ParseNewfstatatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Printf("# newfstatat(%d, %q, %#x)", args.Dfd, filename, args.Flag)
		return simulateFstatat(tid, regs, logger, args.Dfd, filename, args.Statbuf, args.Flag)

	case unix.SYS_STATX:
		args := syscallabi.ParseStatxArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Printf("# statx(%d, %q, %#x, %#x)", args.Dfd, filename, args.Flags, args.Mask)
		return simulateStatx(tid, regs, logger, args.Dfd, filename, args.Flags, args.Mask, args.Buffer)

	case unix.SYS_CHOWN:
		args := syscallabi.ParseChownArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Printf("# chown(%q, %d, %d)", filename, args.Owner, args.Group)
		return simulateFchownat(tid, regs, logger, unix.AT_FDCWD, filename, args.Owner, args.Group, unix.AT_SYMLINK_FOLLOW)

	case unix.SYS_LCHOWN:
		args := syscallabi.ParseLchownArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Printf("# lchown(%q, %d, %d)", filename, args.Owner, args.Group)
		return simulateFchownat(tid, regs, logger, unix.AT_FDCWD, filename, args.Owner, args.Group, unix.AT_SYMLINK_NOFOLLOW)

	case unix.SYS_FCHOWN:
		args := syscallabi.ParseFchownArgs(regs)
		logger.Printf("# fchown(%d, %d, %d)", args.Fd, args.Owner, args.Group)
		return simulateFchownat(tid, regs, logger, args.Fd, "", args.Owner, args.Group, unix.AT_EMPTY_PATH)

	case unix.SYS_FCHOWNAT:
		args := syscallabi.ParseFchownatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Printf("# fchownat(%d, %q, %d, %d, %#x)", args.Dfd, filename, args.User, args.Group, args.Flag)
		return simulateFchownat(tid, regs, logger, args.Dfd, filename, args.User, args.Group, args.Flag)

	case unix.SYS_CHMOD, unix.SYS_FCHMOD, unix.SYS_FCHMODAT:
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
