// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// A fuse file system that mounts an empty directory and waits for some process
// to call setattr on the directory. When it happens, it prints info of the
// process and exits. It exits automatically after 10 seconds if setattr is not
// called.

package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"syscall"
	"time"

	"github.com/hanwen/go-fuse/v2/fs"
	"github.com/hanwen/go-fuse/v2/fuse"
)

type trapRoot struct {
	fs.Inode
	exitCh chan<- struct{}
}

func newTrapRoot() (*trapRoot, <-chan struct{}) {
	exitCh := make(chan struct{}, 1)
	return &trapRoot{
		exitCh: exitCh,
	}, exitCh
}

func (r *trapRoot) Getattr(ctx context.Context, fh fs.FileHandle, out *fuse.AttrOut) syscall.Errno {
	out.Mode = 0o755
	return 0
}

func (r *trapRoot) Setattr(ctx context.Context, f fs.FileHandle, in *fuse.SetAttrIn, out *fuse.AttrOut) syscall.Errno {
	pid := in.Caller.Pid
	fmt.Printf("[trapfs] PID %d called setattr on us\n", pid)
	cmd := exec.Command("/bin/ps", fmt.Sprintf("%d", pid))
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	_ = cmd.Run()
	r.exitCh <- struct{}{}
	return syscall.EPROTO // random error code
}

var _ = (fs.NodeGetattrer)(&trapRoot{})
var _ = (fs.NodeSetattrer)(&trapRoot{})

func main() {
	if err := func() error {
		if len(os.Args) != 2 {
			return fmt.Errorf("need exactly one argument")
		}

		mountPoint := os.Args[1]
		root, exitCh := newTrapRoot()
		options := &fs.Options{
			MountOptions: fuse.MountOptions{
				AllowOther: true,
				FsName:     "trapfs",
			},
		}
		server, err := fs.Mount(mountPoint, root, options)
		if err != nil {
			return err
		}

		fmt.Println("[trapfs] started")

		// Unmount when setattr is called or 10 seconds pass.
		go func() {
			select {
			case <-exitCh:
			case <-time.After(10 * time.Second):
				fmt.Println("[trapfs] timeout was reached")
			}
			server.Unmount()
		}()

		server.Wait()

		fmt.Println("[trapfs] finished")
		return nil
	}(); err != nil {
		fmt.Fprintf(os.Stderr, "[trapfs] FATAL: %v\n", err)
		os.Exit(1)
	}
}
