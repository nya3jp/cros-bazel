// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Package fakechroot implements a utility to enter a fake CrOS chroot.
package fakechroot

import (
	"context"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"

	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
)

const rootDirEnvName = "FAKECHROOT_ROOT_DIR"

func enterNamespace() error {
	// Catch signals.
	ctx, cancel := signal.NotifyContext(context.Background(), unix.SIGINT, unix.SIGTERM)
	defer cancel()

	exe, err := os.Executable()
	if err != nil {
		return err
	}

	newRootDir, err := os.MkdirTemp("/tmp", "fakechroot.")
	if err != nil {
		return err
	}
	defer os.RemoveAll(newRootDir)

	cmd := exec.CommandContext(ctx, exe, os.Args[1:]...)
	cmd.Args[0] = os.Args[0]
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = append(os.Environ(), fmt.Sprintf("%s=%s", rootDirEnvName, newRootDir))
	cmd.SysProcAttr = &syscall.SysProcAttr{
		Cloneflags: syscall.CLONE_NEWUSER | syscall.CLONE_NEWNS,
		UidMappings: []syscall.SysProcIDMap{{
			ContainerID: 0,
			HostID:      os.Getuid(),
			Size:        1,
		}},
		GidMappings: []syscall.SysProcIDMap{{
			ContainerID: 0,
			HostID:      os.Getgid(),
			Size:        1,
		}},
	}

	err = cmd.Run()

	if cmd.ProcessState != nil {
		if status, ok := cmd.ProcessState.Sys().(syscall.WaitStatus); ok {
			// We're going to exit, deferred calls will not run.
			os.RemoveAll(newRootDir)
			if status.Signaled() {
				os.Exit(int(status.Signal()) + 128)
			}
			os.Exit(status.ExitStatus())
		}
	}
	return fmt.Errorf("fork: %w", err)
}

func continueNamespace(newRootDir string) error {
	const oldRootDir = "/.old-root"

	// Make the directory a mount point for pivot_root(2).
	if err := unix.Mount(newRootDir, newRootDir, "", unix.MS_BIND, ""); err != nil {
		return fmt.Errorf("bind-mounting %s: %w", newRootDir, err)
	}

	sourceDir := filepath.Dir(bazelutil.WorkspaceDir())
	chrootDir := filepath.Join(sourceDir, "chroot")

	// Create chroot symlinks.
	for _, o := range []struct {
		src, dst string
	}{
		{"/build", filepath.Join(chrootDir, "build")},
		{"/etc/make.conf", filepath.Join(chrootDir, "etc/make.conf")},
		{"/etc/make.conf.board_setup", filepath.Join(chrootDir, "etc/make.conf.board_setup")},
		{"/etc/make.conf.host_setup", filepath.Join(chrootDir, "etc/make.conf.host_setup")},
		{"/etc/make.conf.user", filepath.Join(chrootDir, "etc/make.conf.user")},
		{"/etc/portage", filepath.Join(chrootDir, "etc/portage")},
		{"/mnt/host/source", sourceDir},
	} {
		if err := os.MkdirAll(filepath.Dir(filepath.Join(newRootDir, o.src)), 0o700); err != nil {
			return fmt.Errorf("creating chroot symlinks: %w", err)
		}
		if err := os.Symlink(o.dst, filepath.Join(newRootDir, o.src)); err != nil {
			return fmt.Errorf("creating chroot symlinks: %w", err)
		}
	}

	// Create system symlinks.
	if err := filepath.WalkDir(newRootDir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !d.IsDir() {
			return nil
		}

		relPath := strings.TrimPrefix(path, newRootDir)
		newDir := filepath.Join(newRootDir, relPath)
		oldDir := filepath.Join(oldRootDir, relPath)
		fis, err := os.ReadDir(filepath.Join("/", relPath))
		if errors.Is(err, os.ErrNotExist) {
			return nil
		}
		if err != nil {
			return err
		}
		for _, fi := range fis {
			if err := os.Symlink(filepath.Join(oldDir, fi.Name()), filepath.Join(newDir, fi.Name())); err != nil && !errors.Is(err, os.ErrExist) {
				return err
			}
		}
		return nil
	}); err != nil {
		return fmt.Errorf("creating system symlinks: %w", err)
	}

	// Create a directory to mount the old root.
	if err := os.MkdirAll(filepath.Join(newRootDir, oldRootDir), 0o700); err != nil {
		return err
	}

	// Call pivot_root(2).
	cwd, err := os.Getwd()
	if err != nil {
		return err
	}
	if err := unix.PivotRoot(newRootDir, filepath.Join(newRootDir, oldRootDir)); err != nil {
		return fmt.Errorf("pivot_root: %w", err)
	}
	if err := unix.Chdir(cwd); err != nil {
		return fmt.Errorf("chdir: %w", err)
	}
	return nil
}

// Enter lets the current process enter a fake CrOS chroot.
//
// A fake CrOS chroot is not a real CrOS chroot, but it's more like a unified
// view of a part of the CrOS chroot and the original system environment.
// Specifically, a fake CrOS chroot provides /mnt/host/source, /build,
// /etc/portage and several other files in the CrOS chroot needed to evaluate
// Portage profiles and ebuilds. However, the process can still access other
// file paths on the system, e.g. Bazel runfiles.
//
// This function internally re-executes the current executable as a subprocess
// to enter new namespaces. It calls os.Exit when a subprocess exits, which
// means that the process exits without running deferred functions calls.
// To avoid leaking deferred function calls, call this function very early in
// your program before needing to clean something up.
func Enter() error {
	if _, err := os.Stat("/etc/cros_chroot_version"); err == nil {
		return errors.New("this program must run outside CrOS chroot")
	}

	newRootDir := os.Getenv(rootDirEnvName)
	if newRootDir == "" {
		if err := enterNamespace(); err != nil {
			return fmt.Errorf("entering fake chroot: entering namespace: %w", err)
		}
		return nil
	}
	if err := continueNamespace(newRootDir); err != nil {
		return fmt.Errorf("entering fake chroot: setting up namespace: %w", err)
	}
	return nil
}
