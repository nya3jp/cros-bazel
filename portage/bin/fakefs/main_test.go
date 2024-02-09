// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"syscall"
	"testing"

	"github.com/bazelbuild/rules_go/go/runfiles"
	"github.com/bazelbuild/rules_go/go/tools/bazel"
)

func fakeFsBin(t *testing.T) string {
	bin, ok := bazel.FindBinary("bazel/portage/bin/fakefs", "fakefs")
	if !ok {
		t.Fatal("fakefs not found")
	}
	return bin
}

func fakeFsPreloadBin(t *testing.T) string {
	bin, err := runfiles.Rlocation("cros/bazel/portage/bin/fakefs/preload/libfakefs_preload.so")
	if err != nil {
		t.Fatalf("libfakefs_preload.so not found: %v", err)
	}
	return bin
}

func testHelperBin(t *testing.T) string {
	bin, err := runfiles.Rlocation("cros/bazel/portage/bin/fakefs/testhelper/testhelper")
	if err != nil {
		t.Fatalf("testhelper not found: %v", err)
	}
	return bin
}

// Specifies the run mode of fakefs.
type runMode int

const (
	// Runs fakefs normally with ptrace and preload.
	runNormal runMode = iota
	// Runs fakefs with ptrace only, essentially simulating the case with
	// statically-linked binaries.
	runNoPreload
	// Runs fakefs with ptrace and preload, and lets the process abort when the
	// preload library had to fall back to ptrace. This mode is useful to exercise
	// the preload library's ability to process certain libc function calls fast.
	runAbortOnSlow
)

// A list of runMode that simulate production behavior.
var productionModes = []runMode{runNormal, runNoPreload}

func (m runMode) String() string {
	switch m {
	case runNormal:
		return "normal"
	case runNoPreload:
		return "no-preload"
	case runAbortOnSlow:
		return "abort-on-slow"
	default:
		panic(fmt.Sprintf("unknown run mode: %d", int(m)))
	}
}

// Runs the command under fakefs and returns stdout.
func runCmd(t *testing.T, mode runMode, cwd string, cmd []string) string {
	args := []string{"--verbose"}
	if mode != runNoPreload {
		args = append(args, fmt.Sprintf("--preload=%s", fakeFsPreloadBin(t)))
	}
	args = append(args, "--")
	args = append(args, cmd...)

	c := exec.Command(fakeFsBin(t), args...)
	c.Dir = cwd
	c.Stdin = nil
	c.Stderr = os.Stderr
	c.Env = os.Environ()

	if mode == runAbortOnSlow {
		c.Env = append(c.Env, "FAKEFS_ABORT_ON_SLOW=1")
	}

	output, err := c.Output()
	if err != nil {
		t.Fatalf("Executing %s failed: %v", c.String(), err)
	}

	return strings.TrimSpace(string(output))
}

func runBash(t *testing.T, mode runMode, cwd string, cmd string) string {
	return runCmd(t, mode, cwd, []string{"bash", "-xe", "-c", cmd})
}

func runTestHelper(t *testing.T, mode runMode, cwd string, cmd string, args ...string) string {
	bin := testHelperBin(t)
	return runCmd(t, mode, cwd, append([]string{bin, cmd}, args...))
}

func TestChownRelative(t *testing.T) {
	for _, mode := range productionModes {
		t.Run(mode.String(), func(t *testing.T) {
			dir := t.TempDir()

			owner := runBash(t, mode, dir, `
				touch foo
				chown 123 foo
				stat -c %u foo
				`)

			if owner != "123" {
				t.Fatalf("Expected owner %s, got %s", "123", owner)
			}
		})
	}
}

func TestChownAbsolute(t *testing.T) {
	for _, mode := range productionModes {
		t.Run(mode.String(), func(t *testing.T) {
			dir := t.TempDir()

			owner := runBash(t, mode, dir, `
				touch foo
				chown 123 "$(realpath foo)"
				stat -c %u foo
				`)

			if owner != "123" {
				t.Fatalf("Expected owner %s, got %s", "123", owner)
			}
		})
	}
}

func TestChgrpRelative(t *testing.T) {
	for _, mode := range productionModes {
		t.Run(mode.String(), func(t *testing.T) {
			dir := t.TempDir()

			owner := runBash(t, mode, dir, `
				touch foo
				chgrp 234 foo
				stat -c %g foo
				`)

			if owner != "234" {
				t.Fatalf("Expected owner %s, got %s", "234", owner)
			}
		})
	}
}

func TestChgrpAbsolute(t *testing.T) {
	for _, mode := range productionModes {
		t.Run(mode.String(), func(t *testing.T) {
			dir := t.TempDir()

			owner := runBash(t, mode, dir, `
				touch foo
				chgrp 234 "$(realpath foo)"
				stat -c %g foo
				`)

			if owner != "234" {
				t.Fatalf("Expected owner %s, got %s", "234", owner)
			}
		})
	}
}

func TestFstatatEmptyPath(t *testing.T) {
	for _, mode := range productionModes {
		t.Run(mode.String(), func(t *testing.T) {
			dir := t.TempDir()

			runBash(t, mode, dir, "touch foo; chown 123:234 foo")
			got := runTestHelper(t, mode, dir, "fstatat-empty-path", "foo")

			const want = "123:234"
			if got != want {
				t.Fatalf("Unexpected ownership: got %s, want %s", got, want)
			}
		})
	}
}

func TestProcSelf(t *testing.T) {
	for _, mode := range productionModes {
		t.Run(mode.String(), func(t *testing.T) {
			dir := t.TempDir()

			runBash(t, mode, dir, "touch foo; chown 123:234 foo")
			got := runTestHelper(t, mode, dir, "stat-proc-self-fd", "foo")

			const want = "123:234"
			if got != want {
				t.Fatalf("Unexpected ownership: got %s, want %s", got, want)
			}
		})
	}
}

func TestFstatOPathFD(t *testing.T) {
	dir := t.TempDir()

	runBash(t, runNormal, dir, "touch foo")
	runTestHelper(t, runAbortOnSlow, dir, "fstatat-empty-path", "foo")
}

func TestFchownOPathFD(t *testing.T) {
	dir := t.TempDir()

	runBash(t, runNormal, dir, "touch foo; chown 123:234 foo")

	// Record the initial ctime.
	stat1, err := os.Stat(filepath.Join(dir, "foo"))
	if err != nil {
		t.Fatalf("Failed to stat foo: %v", err)
	}

	// Call fchown. This should be processed by libfakefs_preload.so.
	runTestHelper(t, runAbortOnSlow, dir, "fchown-self", "foo")

	// Make sure ctime has been updated.
	stat2, err := os.Stat(filepath.Join(dir, "foo"))
	if err != nil {
		t.Fatalf("Failed to stat foo: %v", err)
	}
	if stat2.Sys().(*syscall.Stat_t).Ctim == stat1.Sys().(*syscall.Stat_t).Ctim {
		t.Fatal("fchown did not change ctime")
	}

	// Call fchown again. This should be processed by libfakefs_preload.so too.
	runTestHelper(t, runAbortOnSlow, dir, "fchown-self", "foo")

	// Make sure ctime has been updated.
	stat3, err := os.Stat(filepath.Join(dir, "foo"))
	if err != nil {
		t.Fatalf("Failed to stat foo: %v", err)
	}
	if stat3.Sys().(*syscall.Stat_t).Ctim == stat2.Sys().(*syscall.Stat_t).Ctim {
		t.Fatal("fchown did not change ctime")
	}
}
