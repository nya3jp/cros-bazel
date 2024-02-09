// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"os"
	"os/exec"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
)

func fakeFsBin(t *testing.T) string {
	bin, ok := bazel.FindBinary("bazel/portage/bin/fakefs", "fakefs")
	if !ok {
		t.Fatal("fakefs not found")
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

// Runs the command under fakefs and returns stdout
func runCmd(t *testing.T, cwd string, cmd []string) string {
	args := []string{"--verbose", "--"}
	args = append(args, cmd...)

	c := exec.Command(fakeFsBin(t), args...)
	c.Dir = cwd
	c.Stdin = nil
	// cmd.Stdout = os.Stdout
	c.Stderr = os.Stderr

	output, err := c.Output()
	if err != nil {
		t.Fatalf("Executing %s failed: %v", c.String(), err)
	}

	return strings.TrimSpace(string(output))
}

func runBash(t *testing.T, cwd string, cmd string) string {
	return runCmd(t, cwd, []string{"bash", "-xe", "-c", cmd})
}

func runTestHelper(t *testing.T, cwd string, cmd string, args ...string) string {
	bin := testHelperBin(t)
	return runCmd(t, cwd, append([]string{bin, cmd}, args...))
}

func TestChownRelative(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chown 123 foo
		stat -c %u foo
		`)

	if owner != "123" {
		t.Fatalf("Expected owner %s, got %s", "123", owner)
	}
}

func TestChownAbsolute(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chown 123 "$(realpath foo)"
		stat -c %u foo
		`)

	if owner != "123" {
		t.Fatalf("Expected owner %s, got %s", "123", owner)
	}
}

func TestChgrpRelative(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chgrp 234 foo
		stat -c %g foo
		`)

	if owner != "234" {
		t.Fatalf("Expected owner %s, got %s", "234", owner)
	}
}

func TestChgrpAbsolute(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chgrp 234 "$(realpath foo)"
		stat -c %g foo
		`)

	if owner != "234" {
		t.Fatalf("Expected owner %s, got %s", "234", owner)
	}
}

func TestFstatatEmptyPath(t *testing.T) {
	dir := t.TempDir()

	runBash(t, dir, "touch foo; chown 123:234 foo")
	got := runTestHelper(t, dir, "fstatat-empty-path", "foo")

	const want = "123:234"
	if got != want {
		t.Fatalf("Unexpected ownership: got %s, want %s", got, want)
	}
}

func TestProcSelf(t *testing.T) {
	dir := t.TempDir()

	runBash(t, dir, "touch foo; chown 123:234 foo")
	got := runTestHelper(t, dir, "stat-proc-self-fd", "foo")

	const want = "123:234"
	if got != want {
		t.Fatalf("Unexpected ownership: got %s, want %s", got, want)
	}
}
