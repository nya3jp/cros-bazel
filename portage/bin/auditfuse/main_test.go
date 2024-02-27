// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main_test

import (
	"fmt"
	"log"
	"math/rand"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"sync"
	"syscall"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
)

func auditfuseBin(t *testing.T) string {
	bin, ok := bazel.FindBinary("bazel/portage/bin/auditfuse", "auditfuse")
	if !ok {
		t.Fatal("auditfuse not found")
	}
	return bin
}

func mount(t *testing.T, origDir string) (mountDir string, outputPath string) {
	tempDir := t.TempDir()
	mountDir = filepath.Join(tempDir, "mount")
	outputPath = filepath.Join(tempDir, "output")

	if err := os.Mkdir(mountDir, 0o700); err != nil {
		t.Fatal(err)
	}

	cmd := exec.Command(auditfuseBin(t), "--output", outputPath, origDir, mountDir)
	if err := cmd.Run(); err != nil {
		t.Fatalf("Failed to mount auditfuse: %v", err)
	}

	return mountDir, outputPath
}

func unmount(t *testing.T, mountDir string) {
	if err := syscall.Unmount(mountDir, syscall.MNT_DETACH); err != nil {
		t.Fatal(err)
	}
}

func TestLookup(t *testing.T) {
	origDir := t.TempDir()
	if err := os.Mkdir(filepath.Join(origDir, "foo"), 0o700); err != nil {
		t.Fatal(err)
	}
	mountDir, outputPath := mount(t, origDir)
	defer unmount(t, mountDir)

	os.Stat(filepath.Join(mountDir, "foo/bar/baz"))

	out, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatal(err)
	}

	got := string(out)
	const want = "LOOKUP\t/foo\x00LOOKUP\t/foo/bar\x00"
	if got != want {
		t.Errorf("Wrong output: got %#v, want %#v", got, want)
	}
}

func TestReaddir(t *testing.T) {
	origDir := t.TempDir()
	if err := os.Mkdir(filepath.Join(origDir, "foo"), 0o700); err != nil {
		t.Fatal(err)
	}
	mountDir, outputPath := mount(t, origDir)
	defer unmount(t, mountDir)

	os.ReadDir(filepath.Join(mountDir, "foo"))

	out, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatal(err)
	}

	got := string(out)
	const want = "LOOKUP\t/foo\x00READDIR\t/foo\x00"
	if got != want {
		t.Errorf("Wrong output: got %#v, want %#v", got, want)
	}
}

func TestConcurrency(t *testing.T) {
	const workers = 10
	const entries = 10000

	var names []string
	for i := 0; i < entries; i++ {
		names = append(names, fmt.Sprintf("%04d", i))
	}
	rand.Shuffle(len(names), func(i, j int) {
		names[i], names[j] = names[j], names[i]
	})

	origDir := t.TempDir()
	for _, name := range names {
		if err := os.Mkdir(filepath.Join(origDir, name), 0o700); err != nil {
			t.Fatal(err)
		}
	}

	mountDir, outputPath := mount(t, origDir)
	defer unmount(t, mountDir)

	queue := make(chan string, len(names))
	for _, name := range names {
		queue <- name
	}
	close(queue)

	var wg sync.WaitGroup
	for i := 0; i < workers; i++ {
		go func() {
			defer wg.Done()
			for name := range queue {
				os.Stat(filepath.Join(mountDir, name))
			}
		}()
		wg.Add(1)
	}
	wg.Wait()

	out, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatal(err)
	}

	// Verify that the output is not corrupted.
	lines := strings.Split(string(out), "\x00")
	if lines[len(lines)-1] != "" {
		t.Errorf("Output must end with a null byte")
	}
	lines = lines[:len(lines)-1]

	if len(lines) != entries {
		t.Errorf("Wrong number of reports: got %d, want %d", len(lines), entries)
	}

	pattern := regexp.MustCompile(`^LOOKUP\t/\d{4}$`)
	for _, line := range lines {
		if !pattern.MatchString(line) {
			t.Fatalf("Invalid line: %#v", line)
		}
	}
}

func TestMain(m *testing.M) {
	const rerunEnvName = "AUDITFUSE_TEST_RERUN"

	if os.Getenv(rerunEnvName) != "" {
		os.Exit(m.Run())
	}

	// Re-run the self executable with namespaces.
	exe, err := os.Executable()
	if err != nil {
		log.Fatal(err)
	}

	cmd := exec.Command(exe)
	cmd.Args = os.Args
	cmd.Env = append(os.Environ(), fmt.Sprintf("%s=1", rerunEnvName))
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.SysProcAttr = &syscall.SysProcAttr{
		Cloneflags: syscall.CLONE_NEWUSER | syscall.CLONE_NEWNS,
		UidMappings: []syscall.SysProcIDMap{
			{
				HostID:      os.Getuid(),
				ContainerID: 0,
				Size:        1,
			},
		},
		GidMappings: []syscall.SysProcIDMap{
			{
				HostID:      os.Getgid(),
				ContainerID: 0,
				Size:        1,
			},
		},
		GidMappingsEnableSetgroups: false,
	}
	switch err := cmd.Run().(type) {
	case nil:
		os.Exit(0)
	case *exec.ExitError:
		os.Exit(err.ExitCode())
	default:
		log.Fatal(err)
	}
}
