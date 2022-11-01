// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package syscallabi

// StatArgs contains arguments to stat(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/stat.c;l=290
type StatArgs struct {
	Filename uintptr
	Statbuf  uintptr
}

// LstatArgs contains arguments to lstat(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/stat.c;l=303
type LstatArgs struct {
	Filename uintptr
	Statbuf  uintptr
}

// FstatArgs contains arguments to fstat(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/stat.c;l=316
type FstatArgs struct {
	Fd      int
	Statbuf uintptr
}

// NewfstatatArgs contains arguments to newfstatat(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/stat.c;l=702
type NewfstatatArgs struct {
	Dfd      int
	Filename uintptr
	Statbuf  uintptr
	Flag     int
}

// StatxArgs contains arguments to statx(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/stat.c;l=633
type StatxArgs struct {
	Dfd      int
	Filename uintptr
	Flags    int
	Mask     int
	Buffer   uintptr
}

// ChownArgs contains arguments to chown(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/open.c;l=732
type ChownArgs struct {
	Filename uintptr
	Owner    int
	Group    int
}

// LchownArgs contains arguments to lchown(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/open.c;l=737
type LchownArgs struct {
	Filename uintptr
	Owner    int
	Group    int
}

// FchownArgs contains arguments to fchown(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/open.c;l=768
type FchownArgs struct {
	Fd    int
	Owner int
	Group int
}

// FchownatArgs contains arguments to fchownat(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/open.c;l=726
type FchownatArgs struct {
	Dfd      int
	Filename uintptr
	User     int
	Group    int
	Flag     int
}

// ListxattrArgs contains arguments to listxattr(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/xattr.c;l=817
type ListxattrArgs struct {
	Pathname uintptr
	List     uintptr
	Size     int
}

// LlistxattrArgs contains arguments to llistxattr(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/xattr.c;l=823
type LlistxattrArgs struct {
	Pathname uintptr
	List     uintptr
	Size     int
}

// FlistxattrArgs contains arguments to flistxattr(2).
// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/third_party/kernel/v5.15/fs/xattr.c;l=29
type FlistxattrArgs struct {
	Fd   int
	List uintptr
	Size int
}
