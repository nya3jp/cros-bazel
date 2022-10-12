// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package binarypackage

import (
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"syscall"
)

// See https://www.mankier.com/5/xpak for the format specification.
type File struct {
	xpakStart int64
	size      int64
	f         *os.File
}

type XPAK map[string][]byte

func BinaryPackage(path string) (bp *File, err error) {
	bp = &File{}
	bp.f, err = os.Open(path)
	if err != nil {
		return nil, err
	}

	success := false
	defer func() {
		if !success {
			bp.Close()
		}
	}()

	fi, err := bp.f.Stat()
	if err != nil {
		return nil, err
	}
	bp.size = fi.Size()

	if bp.size < 24 {
		return nil, errors.New("corrupted .tbz2 file: size is too small")
	}
	if err := bp.expectMagic(bp.size-4, "STOP"); err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}
	xpakOffset, err := bp.readUint32(bp.size - 8)
	if err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}
	bp.xpakStart = bp.size - 8 - int64(xpakOffset)
	if bp.xpakStart < 0 {
		return nil, errors.New("corrupted .tbz2 file: invalid xpak_offset")
	}
	if err := bp.expectMagic(bp.size-16, "XPAKSTOP"); err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}
	if err := bp.expectMagic(bp.xpakStart, "XPAKPACK"); err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}

	success = true
	return bp, nil
}

func (bp *File) Close() error {
	return bp.f.Close()
}

func (bp *File) TarballReader() (io.ReadCloser, error) {
	newFd, err := syscall.Dup(int(bp.f.Fd()))
	if err != nil {
		return nil, err
	}
	f := os.NewFile(uintptr(newFd), bp.f.Name())
	if _, err := f.Seek(0, io.SeekStart); err != nil {
		return nil, err
	}
	return readCloser{
		Reader: io.LimitReader(f, bp.xpakStart),
		Closer: f,
	}, nil
}

func (bp *File) Merge(path string) error {
	tarball, err := bp.TarballReader()
	if err != nil {
		return err
	}
	defer tarball.Close()

	// Note that at the moment, ownership is not retained.
	// --same-owner should fix that, but:
	// 1) it can only be run with sudo.
	// 2) If we generate bazel output files with strange ownership, bazel won't
	// have permissions to clean it up.
	// When we write to an image, we'll need to do some work to preserve metadata.
	cmd := exec.Command("tar", "--zstd", "--keep-old-files", "--same-permissions", "-xf", "-")
	cmd.Dir = path
	cmd.Stdin = tarball
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("failed to extract from %s - maybe multiple packages attempt to define the same file: %w", bp.f.Name(), err)
	}
	return nil
}

func (bp *File) Xpak() (XPAK, error) {
	indexLen, err := bp.readUint32(bp.xpakStart + 8)
	if err != nil {
		return nil, err
	}
	dataLen, err := bp.readUint32(bp.xpakStart + 12)
	if err != nil {
		return nil, err
	}
	indexStart := bp.xpakStart + 16
	dataStart := indexStart + int64(indexLen)
	if dataStart+int64(dataLen) != bp.size-16 {
		return nil, fmt.Errorf("corrupted .tbz2 file: data length inconsistency")
	}

	xpak := make(map[string][]byte)
	for indexPos := indexStart; indexPos < dataStart; {
		nameLen, err := bp.readUint32(indexPos)
		if err != nil {
			return nil, err
		}
		indexPos += 4
		nameBuf := make([]byte, int(nameLen))
		if _, err := io.ReadFull(bp.f, nameBuf); err != nil {
			return nil, err
		}
		indexPos += int64(nameLen)
		name := string(nameBuf)
		dataOffset, err := bp.readUint32(indexPos)
		if err != nil {
			return nil, err
		}
		indexPos += 4
		dataLen, err := bp.readUint32(indexPos)
		if err != nil {
			return nil, err
		}
		indexPos += 4

		if _, err := bp.f.Seek(dataStart+int64(dataOffset), io.SeekStart); err != nil {
			return nil, err
		}
		data := make([]byte, int(dataLen))
		if _, err := io.ReadFull(bp.f, data); err != nil {
			return nil, err
		}

		xpak[name] = data
	}

	return xpak, nil
}

func (bp *File) readUint32(offset int64) (uint32, error) {
	if _, err := bp.f.Seek(offset, io.SeekStart); err != nil {
		return 0, err
	}
	buf := make([]byte, 4)
	if _, err := io.ReadFull(bp.f, buf); err != nil {
		return 0, err
	}
	return binary.BigEndian.Uint32(buf), nil
}

func (bp *File) expectMagic(offset int64, want string) error {
	if _, err := bp.f.Seek(offset, io.SeekStart); err != nil {
		return err
	}
	buf := make([]byte, len(want))
	if _, err := io.ReadFull(bp.f, buf); err != nil {
		return err
	}
	if got := string(buf); got != want {
		return fmt.Errorf("bad magic: got %q, want %q", got, want)
	}
	return nil
}

func ReadXpak(path string) (XPAK, error) {
	bp, err := BinaryPackage(path)
	if err != nil {
		return nil, err
	}
	defer bp.Close()
	return bp.Xpak()
}

type readCloser struct {
	io.Reader
	io.Closer
}
