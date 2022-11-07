// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package hooks

import (
	"fmt"
	"math"
	"path/filepath"
	"reflect"
	"strings"
	"unsafe"

	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/cmd/fakefs/fsop"
	"cros.local/bazel/ebuild/private/cmd/fakefs/logging"
	"cros.local/bazel/ebuild/private/cmd/fakefs/ptracearch"
	"cros.local/bazel/ebuild/private/cmd/fakefs/syscallabi"
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

func writeBytes(tid int, ptr uintptr, data []byte) error {
	// Use process_vm_writev(2) instead of ptrace(2) with PTRACE_POKEDATA
	// for much better efficiency.
	if len(data) == 0 {
		return nil
	}
	localIov := []unix.Iovec{{
		Base: &data[0],
		Len:  uint64(len(data)),
	}}
	remoteIov := []unix.RemoteIovec{{
		Base: ptr,
		Len:  int(len(data)),
	}}
	_, err := unix.ProcessVMWritev(tid, localIov, remoteIov, 0)
	return err
}

func writeStruct[T any](tid int, ptr uintptr, data *T) error {
	return writeBytes(tid, ptr, unsafe.Slice((*byte)(unsafe.Pointer(data)), unsafe.Sizeof(*data)))
}

func dirfdPath(tid int, dfd int) string {
	if dfd == unix.AT_FDCWD {
		return fmt.Sprintf("/proc/%d/cwd", tid)
	}
	return fmt.Sprintf("/proc/%d/fd/%d", tid, dfd)
}

// rewritePerThreadPaths rewrites file paths specific to threads.
// TODO: Improve the method to reduce false negatives.
func rewritePerThreadPaths(tid int, path string) string {
	path = filepath.Clean(path)

	// /proc/self/ -> /proc/$tid/
	const procSelf = "/proc/self/"
	if strings.HasPrefix(path, procSelf) {
		path = fmt.Sprintf("/proc/%d/%s", tid, path[len(procSelf):])
	}
	return path
}

func blockSyscallAndReturn(tid int, regs *ptracearch.Regs, ret uint64) func(regs *ptracearch.Regs) {
	// Set the syscall number to -1, which should always fail with ENOSYS.
	regs.Orig_rax = math.MaxUint64
	_ = ptracearch.SetRegs(tid, regs)

	return func(regs *ptracearch.Regs) {
		regs.Rax = ret
		_ = ptracearch.SetRegs(tid, regs)
	}
}

func blockSyscall(tid int, regs *ptracearch.Regs, logger *logging.Logger, err error) func(regs *ptracearch.Regs) {
	errno, ok := err.(unix.Errno)
	if err != nil && !ok {
		logger.Errorf(tid, "%s: %v", syscallabi.Name(int(regs.Orig_rax)), err)
		errno = unix.ENOTRECOVERABLE
	}
	return blockSyscallAndReturn(tid, regs, -uint64(errno))
}

// openat opens a file with arguments intercepted for a tracee thread.
// It returns a file descriptor opened with O_PATH.
func openat(tid int, dfd int, filename string, flags int) (fd int, err error) {
	oflags := unix.O_PATH | unix.O_CLOEXEC
	if flags&unix.AT_SYMLINK_NOFOLLOW != 0 {
		oflags |= unix.O_NOFOLLOW
	}

	// If the file path is absolute, no need to resolve dfd.
	if filepath.IsAbs(filename) {
		return unix.Open(filename, oflags, 0)
	}

	path := dirfdPath(tid, dfd)
	dirfd, err := unix.Open(path, unix.O_PATH|unix.O_CLOEXEC, 0)
	if err != nil {
		return -1, unix.EBADF
	}

	if filename == "" && flags&unix.AT_EMPTY_PATH != 0 {
		return dirfd, nil
	}

	fd, err = unix.Openat(dirfd, filename, oflags, 0)
	_ = unix.Close(dirfd)
	if err != nil {
		return -1, err
	}
	return fd, nil
}

func simulateFstatat(tid int, regs *ptracearch.Regs, logger *logging.Logger, dfd int, filename string, statbuf uintptr, flags int) func(regs *ptracearch.Regs) {
	filename = rewritePerThreadPaths(tid, filename)

	// If the file path is absolute, no need to resolve dfd.
	if filepath.IsAbs(filename) {
		if !fsop.HasOverride(filename, flags&unix.AT_SYMLINK_NOFOLLOW == 0) {
			return nil
		}
	}

	fd, err := openat(tid, dfd, filename, flags)
	if err != nil {
		// Pass through the system call if the target file fails to open.
		return nil
	}
	defer unix.Close(fd)

	var stat unix.Stat_t
	overridden, err := fsop.Fstat(fd, &stat)
	if err != nil {
		return blockSyscall(tid, regs, logger, err)
	}
	if !overridden {
		// Pass through the system call if the file has no override.
		return nil
	}

	err = writeStruct(tid, statbuf, &stat)
	return blockSyscall(tid, regs, logger, err)
}

func simulateStatx(tid int, regs *ptracearch.Regs, logger *logging.Logger, dfd int, filename string, flags int, mask int, statxbuf uintptr) func(regs *ptracearch.Regs) {
	filename = rewritePerThreadPaths(tid, filename)

	// If the file path is absolute, no need to resolve dfd.
	if filepath.IsAbs(filename) {
		if !fsop.HasOverride(filename, flags&unix.AT_SYMLINK_NOFOLLOW == 0) {
			return nil
		}
	}

	fd, err := openat(tid, dfd, filename, flags)
	if err != nil {
		// Pass through the system call if the target file fails to open.
		return nil
	}
	defer unix.Close(fd)

	var statx unix.Statx_t
	overridden, err := fsop.Fstatx(fd, mask, &statx)
	if err != nil {
		return blockSyscall(tid, regs, logger, err)
	}
	if !overridden {
		// Pass through the system call if the file has no override.
		return nil
	}

	err = writeStruct(tid, statxbuf, &statx)
	return blockSyscall(tid, regs, logger, err)
}

func simulateListxattr(tid int, regs *ptracearch.Regs, logger *logging.Logger, filename string, list uintptr, size int, followSymlinks bool) func(regs *ptracearch.Regs) {
	filename = rewritePerThreadPaths(tid, filename)
	if !filepath.IsAbs(filename) {
		filename = fmt.Sprintf("/proc/%d/cwd/%s", tid, filename)
	}
	if !fsop.HasOverride(filename, followSymlinks) {
		return nil
	}

	data, actualSize, err := fsop.Listxattr(filename, size, followSymlinks)
	if err != nil {
		return blockSyscall(tid, regs, logger, err)
	}

	if err := writeBytes(tid, list, data); err != nil {
		return blockSyscall(tid, regs, logger, err)
	}

	return blockSyscallAndReturn(tid, regs, uint64(actualSize))
}

func simulateFlistxattr(tid int, regs *ptracearch.Regs, logger *logging.Logger, fd int, list uintptr, size int) func(regs *ptracearch.Regs) {
	nfd, err := unix.Open(fmt.Sprintf("/proc/%d/fd/%d", tid, fd), unix.O_RDONLY|unix.O_CLOEXEC, 0)
	if err != nil {
		return blockSyscall(tid, regs, logger, unix.EBADF)
	}
	defer unix.Close(nfd)

	if !fsop.FHasOverride(nfd) {
		return nil
	}

	data, actualSize, err := fsop.Flistxattr(nfd, size)
	if err != nil {
		return blockSyscall(tid, regs, logger, err)
	}

	if err := writeBytes(tid, list, data); err != nil {
		return blockSyscall(tid, regs, logger, err)
	}

	return blockSyscallAndReturn(tid, regs, uint64(actualSize))
}

func simulateFchownat(tid int, regs *ptracearch.Regs, logger *logging.Logger, dfd int, filename string, user int, group int, flags int) func(regs *ptracearch.Regs) {
	return blockSyscall(tid, regs, logger, func() error {
		fd, err := openat(tid, dfd, filename, flags)
		if err != nil {
			return err
		}
		defer unix.Close(fd)

		return fsop.Fchown(fd, user, group)
	}())
}

func SyscallList() []int {
	return []int{
		// stat
		unix.SYS_STAT,
		unix.SYS_FSTAT,
		unix.SYS_LSTAT,
		unix.SYS_STATX,
		unix.SYS_NEWFSTATAT,
		// listxattr
		unix.SYS_LISTXATTR,
		unix.SYS_LLISTXATTR,
		unix.SYS_FLISTXATTR,
		// chown
		unix.SYS_CHOWN,
		unix.SYS_LCHOWN,
		unix.SYS_FCHOWN,
		unix.SYS_FCHOWNAT,
	}
}

func OnSyscall(tid int, regs *ptracearch.Regs, logger *logging.Logger) func(regs *ptracearch.Regs) {
	switch regs.Orig_rax {
	case unix.SYS_STAT:
		args := syscallabi.ParseStatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "stat(%q)", filename)
		return simulateFstatat(tid, regs, logger, unix.AT_FDCWD, filename, args.Statbuf, unix.AT_SYMLINK_FOLLOW)

	case unix.SYS_LSTAT:
		args := syscallabi.ParseLstatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "lstat(%q)", filename)
		return simulateFstatat(tid, regs, logger, unix.AT_FDCWD, filename, args.Statbuf, unix.AT_SYMLINK_NOFOLLOW)

	case unix.SYS_FSTAT:
		args := syscallabi.ParseFstatArgs(regs)
		logger.Infof(tid, "fstat(%d)", args.Fd)
		return simulateFstatat(tid, regs, logger, args.Fd, "", args.Statbuf, unix.AT_EMPTY_PATH)

	case unix.SYS_NEWFSTATAT:
		args := syscallabi.ParseNewfstatatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "newfstatat(%d, %q, %#x)", args.Dfd, filename, args.Flag)
		return simulateFstatat(tid, regs, logger, args.Dfd, filename, args.Statbuf, args.Flag)

	case unix.SYS_STATX:
		args := syscallabi.ParseStatxArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "statx(%d, %q, %#x, %#x)", args.Dfd, filename, args.Flags, args.Mask)
		return simulateStatx(tid, regs, logger, args.Dfd, filename, args.Flags, args.Mask, args.Buffer)

	case unix.SYS_LISTXATTR:
		args := syscallabi.ParseListxattrArgs(regs)
		filename, err := readCString(tid, args.Pathname)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "listxattr(%q, %d)", filename, args.Size)
		return simulateListxattr(tid, regs, logger, filename, args.List, args.Size, true)

	case unix.SYS_LLISTXATTR:
		args := syscallabi.ParseLlistxattrArgs(regs)
		filename, err := readCString(tid, args.Pathname)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "llistxattr(%q, %d)", filename, args.Size)
		return simulateListxattr(tid, regs, logger, filename, args.List, args.Size, false)

	case unix.SYS_FLISTXATTR:
		args := syscallabi.ParseFlistxattrArgs(regs)
		logger.Infof(tid, "flistxattr(%d, %d)", args.Fd, args.Size)
		return simulateFlistxattr(tid, regs, logger, args.Fd, args.List, args.Size)

	case unix.SYS_CHOWN:
		args := syscallabi.ParseChownArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "chown(%q, %d, %d)", filename, args.Owner, args.Group)
		return simulateFchownat(tid, regs, logger, unix.AT_FDCWD, filename, args.Owner, args.Group, unix.AT_SYMLINK_FOLLOW)

	case unix.SYS_LCHOWN:
		args := syscallabi.ParseLchownArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "lchown(%q, %d, %d)", filename, args.Owner, args.Group)
		return simulateFchownat(tid, regs, logger, unix.AT_FDCWD, filename, args.Owner, args.Group, unix.AT_SYMLINK_NOFOLLOW)

	case unix.SYS_FCHOWN:
		args := syscallabi.ParseFchownArgs(regs)
		logger.Infof(tid, "fchown(%d, %d, %d)", args.Fd, args.Owner, args.Group)
		return simulateFchownat(tid, regs, logger, args.Fd, "", args.Owner, args.Group, unix.AT_EMPTY_PATH)

	case unix.SYS_FCHOWNAT:
		args := syscallabi.ParseFchownatArgs(regs)
		filename, err := readCString(tid, args.Filename)
		if err != nil {
			return blockSyscall(tid, regs, logger, fmt.Errorf("failed to read filename: %w", err))
		}
		logger.Infof(tid, "fchownat(%d, %q, %d, %d, %#x)", args.Dfd, filename, args.User, args.Group, args.Flag)
		return simulateFchownat(tid, regs, logger, args.Dfd, filename, args.User, args.Group, args.Flag)

	default:
		return nil
	}
}
