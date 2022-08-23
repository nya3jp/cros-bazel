package main

import (
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"syscall"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/urfave/cli"
	"golang.org/x/sys/unix"

	"cros.local/ebuild/private/common/runfiles"
)

func runCommand(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

type overlayType int

const (
	overlayDir overlayType = iota
	overlaySquashfs
)

type overlayInfo struct {
	Target string
	Source string
	Type   overlayType
}

func parseOverlaySpecs(specs []string, t overlayType) ([]overlayInfo, error) {
	var mounts []overlayInfo
	for _, spec := range specs {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid overlay spec: %s", spec)
		}
		mounts = append(mounts, overlayInfo{
			Target: "/" + strings.Trim(v[0], "/"),
			Source: v[1],
			Type:   t,
		})
	}
	return mounts, nil
}

var flagDiffDir = &cli.StringFlag{
	Name:     "diff-dir",
	Required: true,
}

var flagWorkDir = &cli.StringFlag{
	Name:     "work-dir",
	Required: true,
}

var flagChdir = &cli.StringFlag{
	Name:  "chdir",
	Value: "/",
}

var flagOverlayDir = &cli.StringSliceFlag{
	Name: "overlay-dir",
}

var flagOverlaySquashfs = &cli.StringSliceFlag{
	Name: "overlay-squashfs",
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
		flagDiffDir,
		flagWorkDir,
		flagChdir,
		flagOverlayDir,
		flagOverlaySquashfs,
		flagKeepHostMount,
		flagInternalContinue,
	},
	Before: func(c *cli.Context) error {
		if len(c.Args()) == 0 {
			return errors.New("positional arguments missing")
		}
		if _, err := parseOverlaySpecs(c.StringSlice(flagOverlaySquashfs.Name), overlayDir); err != nil {
			return err
		}
		if _, err := parseOverlaySpecs(c.StringSlice(flagOverlaySquashfs.Name), overlaySquashfs); err != nil {
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
	dumbInitPath, err := bazel.Runfile("external/dumb_init/file/downloaded")
	if err != nil {
		return err
	}

	args := append([]string{os.Args[0], "--" + flagInternalContinue.Name}, os.Args[1:]...)
	cmd := exec.Command(dumbInitPath, args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.SysProcAttr = &syscall.SysProcAttr{
		Cloneflags: syscall.CLONE_NEWUSER | syscall.CLONE_NEWNS | syscall.CLONE_NEWPID | syscall.CLONE_NEWNET | syscall.CLONE_NEWIPC,
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
			if status.Signaled() {
				os.Exit(int(status.Signal()) + 128)
			}
			os.Exit(status.ExitStatus())
		}
	}
	return fmt.Errorf("fork: %w", err)
}

func continueNamespace(c *cli.Context) error {
	diffDir := c.String(flagDiffDir.Name)
	workDir := c.String(flagWorkDir.Name)
	//squashfusePath := c.String(flagSquashfuse.Name)
	chdir := c.String(flagChdir.Name)
	dirOverlays, err := parseOverlaySpecs(c.StringSlice(flagOverlayDir.Name), overlayDir)
	if err != nil {
		return err
	}
	squashfsOverlays, err := parseOverlaySpecs(c.StringSlice(flagOverlaySquashfs.Name), overlaySquashfs)
	if err != nil {
		return err
	}
	overlays := append(dirOverlays, squashfsOverlays...)
	keepHostMount := c.Bool(flagKeepHostMount.Name)
	args := []string(c.Args())

	squashfusePath, err := bazel.Runfile("prebuilts/squashfuse")
	if err != nil {
		return err
	}

	pivotRootDone := false // whether pivot_root has been called

	// Enable the loopback networking.
	if err := runCommand("/usr/sbin/ifconfig", "lo", "up"); err != nil {
		return err
	}

	stageDir, err := os.MkdirTemp("/tmp", "run_in_container.*")
	if err != nil {
		return err
	}
	defer func() {
		if !pivotRootDone {
			os.Remove(stageDir)
		}
	}()

	// Mount a tmpfs so that staged files are purged automatically on exit.
	if err := unix.Mount("tmpfs", stageDir, "tmpfs", 0, ""); err != nil {
		return err
	}
	defer func() {
		if !pivotRootDone {
			unix.Unmount(stageDir, unix.MNT_DETACH)
		}
	}()

	rootDir := filepath.Join(stageDir, "root")
	baseDir := filepath.Join(stageDir, "base")
	lowersDir := filepath.Join(stageDir, "lowers")

	for _, dir := range []string{rootDir, baseDir, lowersDir} {
		if err := os.Mkdir(dir, 0o755); err != nil {
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
		lowerDir := filepath.Join(lowersDir, strconv.Itoa(i))
		lowerDirsByMountDir[overlay.Target] = append(lowerDirsByMountDir[overlay.Target], lowerDir)
		if err := os.MkdirAll(lowerDir, 0o755); err != nil {
			return err
		}
		switch overlay.Type {
		case overlayDir:
			if err := unix.Mount(overlay.Source, lowerDir, "", unix.MS_BIND|unix.MS_REC, ""); err != nil {
				return fmt.Errorf("failed bind-mounting %s: %w", overlay.Source, err)
			}
		case overlaySquashfs:
			if err := runCommand(squashfusePath, overlay.Source, lowerDir); err != nil {
				return fmt.Errorf("failed mounting %s: %w", overlay.Source, err)
			}
		default:
			return fmt.Errorf("BUG: unknown overlay type %d", overlay.Type)
		}
	}

	// Ensure mountpoints to exist.
	for mountDir, lowerDirs := range lowerDirsByMountDir {
		if err := os.MkdirAll(filepath.Join(baseDir, mountDir), 0o755); err != nil {
			return err
		}
		lowerDirsByMountDir[mountDir] = append([]string{filepath.Join(baseDir, mountDir)}, lowerDirs...)
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
		upperDir := filepath.Join(diffDir, mountDir)
		workDir := filepath.Join(workDir, strconv.Itoa(i))
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

	// Execute pivot_root.
	if err := unix.PivotRoot(rootDir, filepath.Join(rootDir, "host")); err != nil {
		return fmt.Errorf("pivot_root: %w", err)
	}

	// From now on, deferred cleanups will not run.
	pivotRootDone = true
	pivotedTmpDir := filepath.Join("/host", stageDir)
	if err := unix.Unmount(pivotedTmpDir, unix.MNT_DETACH); err != nil {
		fmt.Fprintf(os.Stderr, "WARNING: failed to unmount stage directory: %v\n", err)
	}
	if err := os.Remove(pivotedTmpDir); err != nil {
		fmt.Fprintf(os.Stderr, "WARNING: failed to delete stage directory: %v\n", err)
	}

	// Unmount /host.
	if !keepHostMount {
		if err := unix.Unmount("/host", unix.MNT_DETACH); err != nil {
			return fmt.Errorf("unmounting /host: %w", err)
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
	runfiles.FixEnv()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
