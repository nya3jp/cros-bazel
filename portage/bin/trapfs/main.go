// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// A fuse file system that mounts an empty directory and waits for some process
// to call setattr on the directory. When it happens, it prints info of the
// process. It exits automatically after 10 seconds.

package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"strconv"
	"syscall"
	"time"

	"github.com/hanwen/go-fuse/v2/fs"
	"github.com/hanwen/go-fuse/v2/fuse"
)

func lookupPIDFromTID(tid uint32) (uint32, error) {
	path := fmt.Sprintf("/proc/%d/status", tid)
	text, err := os.ReadFile(path)
	if err != nil {
		return 0, fmt.Errorf("failed to read %s: %w", path, err)
	}
	match := regexp.MustCompile("\nTgid:\\s*(\\d+)").FindStringSubmatch(string(text))
	if match == nil {
		return 0, fmt.Errorf("failed to find PID from %s", path)
	}
	pid64, err := strconv.ParseUint(match[1], 10, 32)
	if err != nil {
		return 0, fmt.Errorf("failed to parse %s: %w", path, err)
	}
	return uint32(pid64), nil
}

type trapRoot struct {
	fs.Inode
}

func newTrapRoot() *trapRoot {
	return &trapRoot{}
}

func (r *trapRoot) Getattr(ctx context.Context, fh fs.FileHandle, out *fuse.AttrOut) syscall.Errno {
	out.Mode = 0o755
	return 0
}

func (r *trapRoot) Setattr(ctx context.Context, f fs.FileHandle, in *fuse.SetAttrIn, out *fuse.AttrOut) syscall.Errno {
	tid := in.Caller.Pid
	pid, err := lookupPIDFromTID(tid)
	if err != nil {
		fmt.Printf("[trapfs] %v\n", err)
		return 0
	}

	fmt.Printf("[trapfs] PID %d TID %d called setattr on us\n", pid, tid)

	cmd := exec.Command("ps", fmt.Sprintf("%d", pid))
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	_ = cmd.Run()

	cmd = exec.Command("jstack", "-l", "-e", fmt.Sprintf("%d", pid))
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	_ = cmd.Run()

	return 0
}

var _ = (fs.NodeGetattrer)(&trapRoot{})
var _ = (fs.NodeSetattrer)(&trapRoot{})

func main() {
	// Set minimal PATH.
	os.Setenv("PATH", "/usr/local/bin:/usr/bin:/bin")

	if err := func() error {
		if len(os.Args) != 2 {
			return fmt.Errorf("need exactly one argument")
		}

		mountPoint := os.Args[1]
		root := newTrapRoot()
		options := &fs.Options{
			MountOptions: fuse.MountOptions{
				AllowOther: true,
				FsName:     "trapfs",
				Options:    []string{"nonempty"},
			},
		}
		server, err := fs.Mount(mountPoint, root, options)
		if err != nil {
			return err
		}

		fmt.Println("[trapfs] started")

		// Unmount after 10 seconds.
		time.Sleep(10 * time.Second)
		fmt.Println("[trapfs] timeout was reached")

		server.Unmount()
		// For some reasons, server.Wait causes deadlock. I'm suspecting that this
		// is because the FUSE mountpoint is still alive in other mount namespaces,
		// but I'm not 100% sure. In any case, after 10-second timeout, we don't
		// have reasons to serve the file system, so let us forcibly exit the FUSE
		// process. This may put left mountpoints inaccessible, but we don't care
		// about them as the build is doomed to failure.
		fmt.Println("[trapfs] finished")
		return nil
	}(); err != nil {
		fmt.Fprintf(os.Stderr, "[trapfs] FATAL: %v\n", err)
		os.Exit(1)
	}
}
