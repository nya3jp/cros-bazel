package main

import (
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"

	"github.com/urfave/cli"
	"golang.org/x/sys/unix"
)

func runCommand(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

type bindInfo struct {
	Src string
	Dst string
}

func parseBindSpecs(specs []string) ([]bindInfo, error) {
	var binds []bindInfo
	for _, spec := range specs {
		v := strings.Split(spec, ":")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid bind mount spec: %s", spec)
		}
		binds = append(binds, bindInfo{Src: v[0], Dst: v[1]})
	}
	return binds, nil
}

var flagStore = &cli.StringFlag{
	Name: "store",
	Required: true,
}

var flagInit = &cli.StringFlag{
	Name: "init",
	Required: true,
}

var flagChdir = &cli.StringFlag{
	Name: "chdir",
	Value: "/",
}

var flagOverlaySquashfs = &cli.StringSliceFlag{
	Name: "overlay-squashfs",
}

var flagBindMount = &cli.StringSliceFlag{
	Name: "bind-mount",
}

var flagKeepHostMount = &cli.BoolFlag{
	Name: "keep-host-mount",
}

var flagInternalContinue = &cli.BoolFlag{
	Name: "internal-continue",
	Hidden: true,
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagStore,
		flagInit,
		flagChdir,
		flagOverlaySquashfs,
		flagBindMount,
		flagKeepHostMount,
		flagInternalContinue,
	},
	Before: func (c *cli.Context) error {
		if len(c.Args()) == 0 {
			return errors.New("positional arguments missing")
		}
		if _, err := parseBindSpecs(c.StringSlice(flagBindMount.Name)); err != nil {
			return err
		}
		return nil
	},
	Action: func (c *cli.Context) error {
		if !c.Bool(flagInternalContinue.Name) {
			return enterNamespace(c)
		}
		return continueNamespace(c)
	},
}

func enterNamespace(c *cli.Context) error {
	init := c.String(flagInit.Name)

	args := append([]string{os.Args[0], "--" + flagInternalContinue.Name}, os.Args[1:]...)
	cmd := exec.Command(init, args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.SysProcAttr = &syscall.SysProcAttr{
		Cloneflags: syscall.CLONE_NEWUSER | syscall.CLONE_NEWNS | syscall.CLONE_NEWPID,
		UidMappings: []syscall.SysProcIDMap{{
			ContainerID: 0,
			HostID: os.Getuid(),
			Size: 1,
		}},
		GidMappings: []syscall.SysProcIDMap{{
			ContainerID: 0,
			HostID: os.Getgid(),
			Size: 1,
		}},
	}
	err := cmd.Run()
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
	storeDir := c.String(flagStore.Name)
	chdir := c.String(flagChdir.Name)
	images := c.StringSlice(flagOverlaySquashfs.Name)
	bindSpecs := c.StringSlice(flagBindMount.Name)
	keepHostMount := c.Bool(flagKeepHostMount.Name)
	args := []string(c.Args())

	pivotRootDone := false // whether pivot_root has been called

	binds, err := parseBindSpecs(bindSpecs)
	if err != nil {
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
	storeMountDir := filepath.Join(stageDir, "store")

	for _, dir := range []string{rootDir, baseDir, lowersDir, storeMountDir} {
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

	// Create mountpoints for the user-requested bind mounts.
	for _, bind := range binds {
		if err := os.MkdirAll(filepath.Join(baseDir, bind.Dst), 0o755); err != nil {
			return err
		}
	}

	// Set up lower directories.
	lowerDirs := []string{baseDir}
	for i, image := range images {
		lowerDir := filepath.Join(lowersDir, strconv.Itoa(i))
		lowerDirs = append(lowerDirs, lowerDir)
		if err := os.Mkdir(lowerDir, 0o755); err != nil {
			return err
		}
		if err := runCommand("squashfuse", image, lowerDir); err != nil {
			return fmt.Errorf("failed mounting %s: %w", image, err)
		}
	}

	// Bind-mount the store directory to avoid special characters in overlayfs options.
	if err := unix.Mount(storeDir, storeMountDir, "", unix.MS_BIND, ""); err != nil {
		return fmt.Errorf("bind-mounting store dir: %w", err)
	}

	// Set up the store directory.
	upperDir := filepath.Join(storeMountDir, "upper")
	workDir := filepath.Join(storeMountDir, "work")
	for _, dir := range []string{upperDir, workDir} {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return err
		}
	}

	// Mount overlayfs.
	overlayOptions := fmt.Sprintf("upperdir=%s,workdir=%s,lowerdir=%s", upperDir, workDir, strings.Join(lowerDirs, ":"))
	if err := unix.Mount("none", rootDir, "overlay", 0, overlayOptions); err != nil {
		return fmt.Errorf("mounting overlayfs: %w", err)
	}

	// Mount essential filesystems.
	if err := unix.Mount("/dev", filepath.Join(rootDir, "dev"), "", unix.MS_BIND | unix.MS_REC, ""); err != nil {
		return fmt.Errorf("bind-mounting /dev: %w", err)
	}
	if err := unix.Mount("proc", filepath.Join(rootDir, "proc"), "proc", 0, ""); err != nil {
		return fmt.Errorf("mounting /proc: %w", err)
	}
	if err := unix.Mount("/sys", filepath.Join(rootDir, "sys"), "", unix.MS_BIND | unix.MS_REC, ""); err != nil {
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

	// Execute user-requested bind mounts.
	for _, bind := range binds {
		if err := unix.Mount(bind.Src, bind.Dst, "", unix.MS_BIND | unix.MS_REC, ""); err != nil {
			return fmt.Errorf("bind-mounting %s to %s: %w", bind.Src, bind.Dst, err)
		}
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
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
