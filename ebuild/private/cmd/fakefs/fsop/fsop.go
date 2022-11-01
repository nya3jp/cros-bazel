// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Package fsop implements filesystem system calls to simulate privileged
// operations such as chown/chmod for unprivileged users.
package fsop

import (
	"errors"
	"fmt"

	"golang.org/x/sys/unix"
)

const xattrKeyOverride = "user.fakefs.override"

var errNoOverride = errors.New("no override")

func readOverrideData(fd int) (*overrideData, error) {
	buf := make([]byte, 64)
	size, err := unix.Fgetxattr(fd, xattrKeyOverride, buf)
	if err == unix.ENODATA || err == unix.ENOTSUP {
		return nil, errNoOverride
	}
	if err != nil {
		return nil, err
	}
	return parseOverrideData(buf[:size])
}

func writeOverrideData(fd int, data *overrideData) error {
	return unix.Fsetxattr(fd, xattrKeyOverride, data.Marshal(), 0)
}

// upgradeFd upgrades a file descriptor opened with O_PATH to a regular file
// descriptor.
func upgradeFd(fd int) (int, error) {
	return unix.Open(fmt.Sprintf("/proc/self/fd/%d", fd), unix.O_RDONLY|unix.O_CLOEXEC, 0)
}

// HasOverride returns if a file has an override xattr.
// If the file does not exist, it is considered that an override xattr is
// missing.
func HasOverride(path string, followSymlinks bool) bool {
	var err error
	if followSymlinks {
		_, err = unix.Getxattr(path, xattrKeyOverride, nil)
	} else {
		_, err = unix.Lgetxattr(path, xattrKeyOverride, nil)
	}
	return err == nil || err == unix.ERANGE
}

// FHasOverride returns if a file has an override xattr.
func FHasOverride(fd int) bool {
	_, err := unix.Fgetxattr(fd, xattrKeyOverride, nil)
	return err == nil || err == unix.ERANGE
}

// Fstat returns stat_t for a given file descriptor.
// If a file pointed by fd is a regular file or a directory, it considers xattrs
// to override file metadata. Otherwise it behaves like normal fstat(2).
// fd can be a file descriptor opened with O_PATH.
func Fstat(fd int, stat *unix.Stat_t) (overridden bool, err error) {
	// Use fstatat(2) instead of fstat(2) to support file descriptors opened
	// with O_PATH.
	if err := unix.Fstatat(fd, "", stat, unix.AT_EMPTY_PATH); err != nil {
		return false, err
	}

	switch stat.Mode & unix.S_IFMT {
	case unix.S_IFREG, unix.S_IFDIR:
		ufd, err := upgradeFd(fd)
		if err != nil {
			return false, err
		}
		defer unix.Close(ufd)

		data, err := readOverrideData(ufd)
		if err == errNoOverride {
			return false, nil
		}
		if err != nil {
			return false, err
		}

		stat.Uid = uint32(data.Uid)
		stat.Gid = uint32(data.Gid)
		return true, nil

	default:
		return false, nil
	}
}

// Fstatx returns statx_t for a given file descriptor.
// If a file pointed by fd is a regular file or a directory, it considers xattrs
// to override file metadata. Otherwise it behaves like normal statx(2).
// fd can be a file descriptor opened with O_PATH.
func Fstatx(fd int, mask int, statx *unix.Statx_t) (overridden bool, err error) {
	// Always request the mode field.
	// It is fine for statx(2) to return non-requested fields and thus its
	// mask field differs from the requested mask.
	mask |= unix.STATX_MODE

	// TODO: Pass through AT_STATX_* flags.
	if err := unix.Statx(fd, "", unix.AT_EMPTY_PATH, mask|unix.STATX_MODE, statx); err != nil {
		return false, err
	}

	switch statx.Mode & unix.S_IFMT {
	case unix.S_IFREG, unix.S_IFDIR:
		ufd, err := upgradeFd(fd)
		if err != nil {
			return false, err
		}
		defer unix.Close(ufd)

		data, err := readOverrideData(ufd)
		if err == errNoOverride {
			return false, nil
		}
		if err != nil {
			return false, err
		}

		if statx.Mask&unix.STATX_UID != 0 {
			statx.Uid = uint32(data.Uid)
		}
		if statx.Mask&unix.STATX_GID != 0 {
			statx.Gid = uint32(data.Gid)
		}
		return true, nil

	default:
		return false, nil
	}
}

func doListxattr(cap int, listxattr func([]byte) (int, error)) (keys []byte, size int, err error) {
	unfiltered := make([]byte, cap)
	size, err = listxattr(unfiltered)
	if err != nil {
		return nil, 0, err
	}

	// If cap is 0, listxattr(2) family returns the required size without storing
	// results.
	if cap == 0 {
		return nil, size, nil
	}

	unfiltered = unfiltered[:size]

	var filtered []byte
	var key []byte
	for _, b := range unfiltered {
		if b != 0 {
			key = append(key, b)
			continue
		}
		if string(key) != xattrKeyOverride {
			filtered = append(filtered, append(key, 0)...)
		}
		key = nil
	}
	if len(key) > 0 {
		return nil, 0, fmt.Errorf("listxattr result not null-terminated")
	}
	return filtered, len(filtered), nil
}

// Listxattr enumerates xattrs of a file, hiding fakefs-specific entries.
func Listxattr(path string, cap int, followSymlinks bool) (keys []byte, size int, err error) {
	return doListxattr(cap, func(buf []byte) (int, error) {
		if followSymlinks {
			return unix.Listxattr(path, buf)
		}
		return unix.Llistxattr(path, buf)
	})
}

// Flistxattr enumerates xattrs of a file, hiding fakefs-specific entries.
func Flistxattr(fd int, cap int) (keys []byte, size int, err error) {
	return doListxattr(cap, func(buf []byte) (int, error) {
		return unix.Flistxattr(fd, buf)
	})
}

// Fchown changes ownership of a given file.
// If a file pointed by fd is a regular file or a directory, it sets xattrs
// to override file metadata. Otherwise it fails if ownership is being changed.
// fd can be a file descriptor opened with O_PATH.
func Fchown(fd int, uid int, gid int) error {
	// TODO: Consider locking the file to avoid races.
	// TODO: Avoid upgrading the file descriptor twice.
	var stat unix.Stat_t
	if _, err := Fstat(fd, &stat); err != nil {
		return err
	}

	if uid < 0 {
		uid = int(stat.Uid)
	}
	if gid < 0 {
		gid = int(stat.Gid)
	}

	switch stat.Mode & unix.S_IFMT {
	case unix.S_IFREG, unix.S_IFDIR:
		ufd, err := upgradeFd(fd)
		if err != nil {
			return err
		}
		defer unix.Close(ufd)

		data := &overrideData{
			Uid: uid,
			Gid: gid,
		}
		if err := writeOverrideData(ufd, data); err != nil {
			return err
		}

	default:
		if uid != int(stat.Uid) || gid != int(stat.Gid) {
			return errors.New("cannot change ownership of non-regular files")
		}
	}
	return nil
}
