// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package fsimpl

import (
	"context"
	"path/filepath"
	"syscall"

	"github.com/hanwen/go-fuse/v2/fs"
	"github.com/hanwen/go-fuse/v2/fuse"

	"cros.local/bazel/portage/bin/auditfuse/reporter"
)

type AuditNode struct {
	fs.LoopbackNode
	r *reporter.Reporter
}

var _ fs.InodeEmbedder = &AuditNode{}

func (n *AuditNode) Lookup(ctx context.Context, name string, out *fuse.EntryOut) (*fs.Inode, syscall.Errno) {
	n.r.Report(reporter.Lookup, filepath.Join("/", n.Path(nil), name))
	return n.LoopbackNode.Lookup(ctx, name, out)
}

func (n *AuditNode) Readdir(ctx context.Context) (fs.DirStream, syscall.Errno) {
	n.r.Report(reporter.Readdir, filepath.Join("/", n.Path(nil)))
	return n.LoopbackNode.Readdir(ctx)
}

func NewRoot(origDir string, r *reporter.Reporter) (*AuditNode, error) {
	// Compute the absolute file path to allow changing the working directory.
	origDir, err := filepath.Abs(origDir)
	if err != nil {
		return nil, err
	}

	var st syscall.Stat_t
	if err := syscall.Stat(origDir, &st); err != nil {
		return nil, err
	}

	root := &fs.LoopbackRoot{
		Path: origDir,
		Dev:  st.Dev,
		NewNode: func(root *fs.LoopbackRoot, parent *fs.Inode, name string, st *syscall.Stat_t) fs.InodeEmbedder {
			parentOps := parent.Operations().(*AuditNode)
			return &AuditNode{
				LoopbackNode: fs.LoopbackNode{
					RootData: root,
				},
				r: parentOps.r,
			}
		},
	}
	return &AuditNode{
		LoopbackNode: fs.LoopbackNode{
			RootData: root,
		},
		r: r,
	}, nil
}
