// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Package fsop implements filesystem system calls to simulate privileged
// operations such as chown/chmod for unprivileged users.
package fsop

import (
	"errors"
	"fmt"
	"os"

	"golang.org/x/sys/unix"
)

const xattrKeyOverride = "user.fakefs.override"

var errNoOverride = errors.New("no override")

func readOverrideData(f *os.File) (*overrideData, error) {
	buf := make([]byte, 64)
	size, err := unix.Fgetxattr(int(f.Fd()), xattrKeyOverride, buf)
	if err == unix.ENODATA {
		return nil, errNoOverride
	}
	if err != nil {
		return nil, err
	}
	return parseOverrideData(buf[:size])
}

func writeOverrideData(f *os.File, data *overrideData) error {
	return unix.Fsetxattr(int(f.Fd()), xattrKeyOverride, data.Marshal(), 0)
}

// upgradeFd upgrades a file descriptor opened with O_PATH to a regular file
// descriptor.
func upgradeFd(fd int) (*os.File, error) {
	return os.Open(fmt.Sprintf("/proc/self/fd/%d", fd))
}

// Fstat returns stat_t for a given file descriptor.
// If a file pointed by fd is a regular file or a directory, it considers xattrs
// to override file metadata. Otherwise it behaves like normal fstat(2).
// fd can be a file descriptor opened with O_PATH.
func Fstat(fd int, stat *unix.Stat_t) error {
	// Use fstatat(2) instead of fstat(2) to support file descriptors opened
	// with O_PATH.
	if err := unix.Fstatat(fd, "", stat, unix.AT_EMPTY_PATH); err != nil {
		return err
	}

	switch stat.Mode & unix.S_IFMT {
	case unix.S_IFREG, unix.S_IFDIR:
		f, err := upgradeFd(fd)
		if err != nil {
			return err
		}
		defer f.Close()

		data, err := readOverrideData(f)
		if err == errNoOverride {
			return nil
		}
		if err != nil {
			return err
		}

		stat.Uid = uint32(data.Uid)
		stat.Gid = uint32(data.Gid)
		return nil

	default:
		return nil
	}
}

// Fstatx returns statx_t for a given file descriptor.
// If a file pointed by fd is a regular file or a directory, it considers xattrs
// to override file metadata. Otherwise it behaves like normal statx(2).
// fd can be a file descriptor opened with O_PATH.
func Fstatx(fd int, mask int, statx *unix.Statx_t) error {
	// Always request the mode field.
	// It is fine for statx(2) to return non-requested fields and thus its
	// mask field differs from the requested mask.
	mask |= unix.STATX_MODE

	// TODO: Pass through AT_STATX_* flags.
	if err := unix.Statx(fd, "", unix.AT_EMPTY_PATH, mask|unix.STATX_MODE, statx); err != nil {
		return err
	}

	switch statx.Mode & unix.S_IFMT {
	case unix.S_IFREG, unix.S_IFDIR:
		f, err := upgradeFd(fd)
		if err != nil {
			return err
		}
		defer f.Close()

		data, err := readOverrideData(f)
		if err == errNoOverride {
			return nil
		}
		if err != nil {
			return err
		}

		if statx.Mask&unix.STATX_UID != 0 {
			statx.Uid = uint32(data.Uid)
		}
		if statx.Mask&unix.STATX_GID != 0 {
			statx.Gid = uint32(data.Gid)
		}
		return nil

	default:
		return nil
	}
}

// Fchown changes ownership of a given file.
// If a file pointed by fd is a regular file or a directory, it sets xattrs
// to override file metadata. Otherwise it fails if ownership is being changed.
// fd can be a file descriptor opened with O_PATH.
func Fchown(fd int, uid int, gid int) error {
	// TODO: Consider locking the file to avoid races.
	// TODO: Avoid upgrading the file descriptor twice.
	var stat unix.Stat_t
	if err := Fstat(fd, &stat); err != nil {
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
		f, err := upgradeFd(fd)
		if err != nil {
			return err
		}
		defer f.Close()

		data := &overrideData{
			Uid: uid,
			Gid: gid,
		}
		if err := writeOverrideData(f, data); err != nil {
			return err
		}

	default:
		if uid != int(stat.Uid) || gid != int(stat.Gid) {
			return errors.New("cannot change ownership of non-regular files")
		}
	}
	return nil
}
