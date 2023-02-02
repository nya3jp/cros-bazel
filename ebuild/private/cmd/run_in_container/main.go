// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"syscall"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/cliutil"
	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/processes"
	"cros.local/bazel/ebuild/private/common/tar"
)

func runCommand(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

type overlayInfo struct {
	Target string
	Source string
	Type   makechroot.OverlayType
}

func parseOverlaySpecs(specs []string) ([]overlayInfo, error) {
	overlays, err := makechroot.ParseOverlaySpecs(specs)
	if err != nil {
		return nil, err
	}

	var mounts []overlayInfo
	for _, overlay := range overlays {
		// If the mount path is relative, use /mnt/host/source as the base.
		mountDir := overlay.MountDir
		if !filepath.IsAbs(mountDir) {
			mountDir = filepath.Join("/mnt/host/source", mountDir)
		}

		overlayType, err := makechroot.DetectOverlayType(overlay.ImagePath)
		if err != nil {
			return nil, err
		}

		mounts = append(mounts, overlayInfo{
			Target: mountDir,
			Source: overlay.ImagePath,
			Type:   overlayType,
		})
	}
	return mounts, nil
}

// resolveOverlaySourcePath translates an overlay source path by possibly
// following symlinks.
//
// The --overlay inputs can be three types:
//  1. A path to a real file/directory.
//  2. A symlink to a file/directory.
//  3. A directory tree with symlinks pointing to real files.
//
// This function undoes the case 3. Bazel should be giving us a symlink to
// the directory, instead of creating a symlink tree. We don't want to use the
// symlink tree because that would require bind mounting the whole execroot
// inside the container. Otherwise we couldn't resolve the symlinks.
//
// This method will find the first symlink in the symlink forest which will be
// pointing to the real execroot. It then calculates the folder that should have
// been passed in by bazel.
func resolveOverlaySourcePath(inputPath string) (string, error) {
	info, err := os.Lstat(inputPath)
	if err != nil {
		return "", err
	}

	if info.Mode()&fs.ModeSymlink != 0 {
		// Resolve the symlink so we always return an absolute path.
		pointsTo, err := filepath.EvalSymlinks(inputPath)
		if err != nil {
			return "", err
		}
		return pointsTo, nil
	}

	if !info.IsDir() {
		return inputPath, nil
	}

	done := errors.New("done")

	// In the case that the directory is empty, we still want the returned path to
	// be valid.
	resolvedInputPath := inputPath
	err = filepath.WalkDir(inputPath, func(path string, info fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		if info.IsDir() {
			return nil
		}

		fileInfo, err := info.Info()
		if err != nil {
			return err
		}

		target := path
		if fileInfo.Mode().Type()&fs.ModeSymlink != 0 {
			target, err = os.Readlink(path)
			if err != nil {
				return err
			}
		}

		insidePath := strings.TrimPrefix(path, inputPath)

		resolvedInputPath = strings.TrimSuffix(target, insidePath)

		return done
	})

	if err != nil && err != done {
		return "", err
	}

	return resolvedInputPath, nil
}

var flagStagingDir = &cli.StringFlag{
	Name:     "scratch-dir",
	Required: true,
	Usage: "Directory that will be used by the overlayfs mount. " +
		"Output artifacts can be found in the 'diff' directory.",
}

var flagChdir = &cli.StringFlag{
	Name:  "chdir",
	Value: "/",
}

var flagOverlay = &cli.StringSliceFlag{
	Name: "overlay",
	Usage: "<inside>=<outside>. " +
		"Mounts a file system layer found at an outside path to a directory " +
		"inside the container at the specified inside path. " +
		"This option can be specified multiple times. The earlier " +
		"overlays are mounted as the higher layer, and the later " +
		"overlays are mounted as the lower layer.",
	Required: true,
}

var flagBindMount = &cli.StringSliceFlag{
	Name: "bind-mount",
	Usage: "<mountpoint>=<source>. " +
		"Mounts a file contained at source to the mountpoint specified",
}

var flagKeepHostMount = &cli.BoolFlag{
	Name: "keep-host-mount",
}

var flagInternalContinue = &cli.BoolFlag{
	Name:   "internal-continue",
	Hidden: true,
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagStagingDir,
		flagChdir,
		flagOverlay,
		flagBindMount,
		flagKeepHostMount,
		flagInternalContinue,
	},
	Before: func(c *cli.Context) error {
		if c.Args().Len() == 0 {
			return errors.New("positional arguments missing")
		}
		if _, err := parseOverlaySpecs(c.StringSlice(flagOverlay.Name)); err != nil {
			return err
		}
		if _, err := makechroot.ParseBindMountSpec(c.StringSlice(flagBindMount.Name)); err != nil {
			return err
		}
		return nil
	},
	Action: func(c *cli.Context) error {
		if !c.Bool(flagInternalContinue.Name) {
			return enterNamespace(c)
		}
		return continueNamespace(c)
	},
}

func enterNamespace(c *cli.Context) error {
	// should be dumb_init/file/downloaded after switching to rlocation.
	dumbInitPath, err := bazel.Runfile("file/downloaded")
	if err != nil {
		return err
	}

	// --single-child tells dumb-init to not create a new SID. A new SID doesn't
	// have a controlling terminal, so running `bash` won't work correctly.
	// By omitting the new SID creation, the init processes will inherit the
	// current (outside) SID and PGID. This is desirable because then the parent
	// shell can correctly perform job control (Ctrl+Z) on all the processes.
	// It also tells dumb-init to only forward signals to the child, instead of
	// the child's PGID, this is undesirable, but not really a problem in
	// practice. The other processes we run are `squashfsfuse`, and these create
	// their own SID's, so we were never forwarding the signals to those processes
	// in the first place. Honestly, I'm not sure if we really even want signal
	// forwarding. Instead our `init` processes should only handle
	// `SIGINT`/`SIGTERM`, perform a `kill -TERM -1` to notify all the processes
	// in the PID namespace to shut down cleanly, then wait for all processes
	// to exit.
	args := append([]string{"--single-child", os.Args[0], "--" + flagInternalContinue.Name}, os.Args[1:]...)
	cmd := exec.Command(dumbInitPath, args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	var cloneFlags uintptr = syscall.CLONE_NEWNS | syscall.CLONE_NEWPID | syscall.CLONE_NEWNET | syscall.CLONE_NEWIPC
	// If this script is running as root, it's assumed that the user wants to do
	// things requiring root privileges inside the container. However, mapping 0
	// to 0 is not the same as no user namespace. If you map 0 to 0, you will no
	// longer be able to execute commands requiring root, and won't be able to
	// access files owned by other users.
	if os.Getuid() != 0 {
		cloneFlags |= syscall.CLONE_NEWUSER
		cmd.SysProcAttr = &syscall.SysProcAttr{
			Cloneflags: cloneFlags,
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
	}
	err = processes.Run(c.Context, cmd)
	if cmd.ProcessState != nil {
		if status, ok := cmd.ProcessState.Sys().(syscall.WaitStatus); ok {
			if status.Signaled() {
				return cliutil.ExitCode(int(status.Signal()) + 128)
			}
			return cliutil.ExitCode(status.ExitStatus())
		}
	}
	return fmt.Errorf("fork: %w", err)
}

func continueNamespace(c *cli.Context) error {
	stageDir, err := filepath.Abs(c.String(flagStagingDir.Name))
	if err != nil {
		return err
	}

	chdir := c.String(flagChdir.Name)
	overlays, err := parseOverlaySpecs(c.StringSlice(flagOverlay.Name))
	if err != nil {
		return err
	}
	bindMounts, err := makechroot.ParseBindMountSpec(c.StringSlice(flagBindMount.Name))
	if err != nil {
		return err
	}
	keepHostMount := c.Bool(flagKeepHostMount.Name)
	args := c.Args().Slice()

	squashfusePath, err := bazel.Runfile("rules_cros/third_party/squashfuse/squashfuse")
	if err != nil {
		return err
	}

	// Enable the loopback networking.
	if err := runCommand("/usr/sbin/ifconfig", "lo", "up"); err != nil {
		return err
	}

	// We keep all the directories in the stage dir to keep relative file
	// paths short.
	rootDir := filepath.Join(stageDir, "root") // Merged directory
	baseDir := filepath.Join(stageDir, "base") // Directory containing mount targets
	lowersDir := filepath.Join(stageDir, "lowers")
	diffDir := filepath.Join(stageDir, "diff")
	workDir := filepath.Join(stageDir, "work")
	tarDir := filepath.Join(stageDir, "tar")

	for _, dir := range []string{rootDir, baseDir, lowersDir, diffDir, workDir, tarDir} {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return err
		}
	}

	for _, dir := range []string{rootDir, baseDir, lowersDir} {
		// Mount a tmpfs so that files are purged automatically on exit.
		if err := unix.Mount("tmpfs", dir, "tmpfs", 0, ""); err != nil {
			return err
		}
	}

	// Set up the base directory.
	for _, name := range []string{"dev", "proc", "sys", "tmp", "host"} {
		if err := os.Mkdir(filepath.Join(baseDir, name), 0o755); err != nil {
			return err
		}
	}

	// Set up lower directories.
	lowerDirsByMountDir := make(map[string][]string)
	for i, overlay := range overlays {
		sourcePath, err := resolveOverlaySourcePath(overlay.Source)
		if err != nil {
			return err
		}

		lowerDir := filepath.Join(lowersDir, strconv.Itoa(i))
		if err := os.MkdirAll(lowerDir, 0o755); err != nil {
			return err
		}

		switch overlay.Type {
		case makechroot.OverlayDir:
			if err := unix.Mount(sourcePath, lowerDir, "", unix.MS_BIND|unix.MS_REC, ""); err != nil {
				return fmt.Errorf("failed bind-mounting %s: %w", sourcePath, err)
			}
		case makechroot.OverlaySquashfs:
			if err := runCommand(
				squashfusePath,
				sourcePath,
				lowerDir); err != nil {
				return fmt.Errorf("failed mounting %s: %w", sourcePath, err)
			}
		case makechroot.OverlayTar:
			// We use a dedicated directory for the extracted artifacts instead of
			// putting them in the lower directory because the lower directory is a
			// tmpfs mount and we don't want to use up all the RAM.
			lowerDir = filepath.Join(tarDir, strconv.Itoa(i))
			if err := os.MkdirAll(lowerDir, 0o755); err != nil {
				return err
			}
			if err := tar.Extract(overlay.Source, lowerDir); err != nil {
				return fmt.Errorf("failed to extract %s: %w", overlay.Source, err)
			}
		default:
			return fmt.Errorf("BUG: unknown overlay type %d", overlay.Type)
		}

		lowerDirsByMountDir[overlay.Target] = append(lowerDirsByMountDir[overlay.Target], lowerDir)
	}

	// Ensure mountpoints to exist in base.
	for mountDir, lowerDirs := range lowerDirsByMountDir {
		if err := os.MkdirAll(filepath.Join(baseDir, mountDir), 0o755); err != nil {
			return err
		}
		lowerDirsByMountDir[mountDir] = append(lowerDirs, filepath.Join(baseDir, mountDir))
	}

	// Change the current directory to minimize the option string passed to
	// mount(2) as its length is constrained.
	if err := os.Chdir(lowersDir); err != nil {
		return err
	}

	// Overlay multiple directories.
	// This is done by mounting overlayfs per mount offset.
	mountDirs := make([]string, 0, len(lowerDirsByMountDir))
	for mountDir := range lowerDirsByMountDir {
		mountDirs = append(mountDirs, mountDir)
	}
	sort.Strings(mountDirs) // must be topologically sorted

	for i, mountDir := range mountDirs {
		// Set up the store directory.
		// TODO: Avoid overlapped upper directories. They cause overlayfs to emit
		// warnings in kernel logs.
		upperDir, err := filepath.Rel(lowersDir, filepath.Join(diffDir, mountDir))
		if err != nil {
			return err
		}

		workDir, err := filepath.Rel(lowersDir, filepath.Join(workDir, strconv.Itoa(i)))
		if err != nil {
			return err
		}

		for _, dir := range []string{upperDir, workDir} {
			if err := os.MkdirAll(dir, 0o755); err != nil {
				return err
			}
		}

		// Compute shorter representations of lower directories.
		lowerDirs := lowerDirsByMountDir[mountDir]
		shortLowerDirs := make([]string, 0, len(lowerDirs))
		for _, lowerDir := range lowerDirs {
			relLowerDir, err := filepath.Rel(lowersDir, lowerDir)
			if err != nil {
				return err
			}
			shortLowerDir := relLowerDir
			if len(relLowerDir) > len(lowerDir) {
				shortLowerDir = lowerDir
			}
			shortLowerDirs = append(shortLowerDirs, shortLowerDir)
		}

		// Mount overlayfs.
		overlayOptions := fmt.Sprintf("upperdir=%s,workdir=%s,lowerdir=%s", upperDir, workDir, strings.Join(shortLowerDirs, ":"))
		if err := unix.Mount("none", filepath.Join(rootDir, mountDir), "overlay", 0, overlayOptions); err != nil {
			return fmt.Errorf("mounting overlayfs: %w", err)
		}
	}

	// Mount essential filesystems.
	if err := unix.Mount("/dev", filepath.Join(rootDir, "dev"), "", unix.MS_BIND|unix.MS_REC, ""); err != nil {
		return fmt.Errorf("bind-mounting /dev: %w", err)
	}
	if err := unix.Mount("proc", filepath.Join(rootDir, "proc"), "proc", 0, ""); err != nil {
		return fmt.Errorf("mounting /proc: %w", err)
	}
	if err := unix.Mount("/sys", filepath.Join(rootDir, "sys"), "", unix.MS_BIND|unix.MS_REC, ""); err != nil {
		return fmt.Errorf("bind-mounting /sys: %w", err)
	}

	for _, mount := range bindMounts {
		target := filepath.Join(rootDir, mount.MountPath)
		if err := os.MkdirAll(filepath.Dir(target), 0755); err != nil {
			return err
		}

		// When bind-mounting, the destination must exist.
		_, err := os.Stat(target)
		if errors.Is(err, os.ErrNotExist) {
			srcStat, err := os.Stat(mount.Source)
			if err != nil {
				return fmt.Errorf("stat %s failed: %w", mount.Source, err)
			}
			if srcStat.IsDir() {
				if err := os.Mkdir(target, 0755); err != nil {
					return fmt.Errorf("mkdir %s failed: %w", target, err)
				}
			} else {
				if err := os.WriteFile(target, []byte{}, 0755); err != nil {
					return fmt.Errorf("touch %s failed: %w", target, err)
				}
			}
		} else if err != nil {
			return fmt.Errorf("stat %s failed: %w", target, err)
		}

		// Unfortunately, the unix.MS_RDONLY flag is ignored for bind-mounts.
		// Thus, we mount a bind-mount, then remount it as readonly.
		if err := unix.Mount(mount.Source, target, "", unix.MS_BIND, ""); err != nil {
			return fmt.Errorf("failed bind-mounting file %s: %w", mount.Source, err)
		}
		if err := unix.Mount("", target, "", unix.MS_REMOUNT|unix.MS_BIND|unix.MS_RDONLY, ""); err != nil {
			return fmt.Errorf("failed remounting file %s as read-only: %w", mount.Source, err)
		}
	}

	// Execute pivot_root.
	if err := unix.PivotRoot(rootDir, filepath.Join(rootDir, "host")); err != nil {
		return fmt.Errorf("pivot_root: %w", err)
	}

	// Unmount /host.
	if !keepHostMount {
		if err := unix.Unmount("/host", unix.MNT_DETACH); err != nil {
			return fmt.Errorf("unmounting /host: %w", err)
		}
	}

	// These are absolute paths that are no longer valid after we pivot.
	for _, envVarName := range []string{"RUNFILES_DIR", "RUNFILES_MANIFEST_FILE"} {
		if err := os.Unsetenv(envVarName); err != nil {
			return fmt.Errorf("Failed to unset %s: %w", envVarName, err)
		}
	}

	// Proceed to run the user command.
	if err := os.Chdir(chdir); err != nil {
		return fmt.Errorf("chdir: %w", err)
	}

	exe, err := exec.LookPath(args[0])
	if err != nil {
		return err
	}

	return unix.Exec(exe, args, os.Environ())
}

func main() {
	bazelutil.FixRunfilesEnv()
	cliutil.Exit(app.Run(os.Args))
}
