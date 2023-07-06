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

func TestChownRelative(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chown nobody foo
		stat -c %U foo
		`)

	if owner != "nobody" {
		t.Fatalf("Expected owner '%s', got '%s'", "nobody", owner)
	}
}

func TestChownAbsolute(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chown nobody "$(realpath foo)"
		stat -c %U foo
		`)

	if owner != "nobody" {
		t.Fatalf("Expected owner '%s', got '%s'", "nobody", owner)
	}
}

func TestChgrpRelative(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chgrp nobody foo
		stat -c %G foo
		`)

	if owner != "nobody" {
		t.Fatalf("Expected owner '%s', got '%s'", "nobody", owner)
	}
}

func TestChgrpAbsolute(t *testing.T) {
	dir := t.TempDir()

	owner := runBash(t, dir, `
		touch foo
		chgrp nobody "$(realpath foo)"
		stat -c %G foo
		`)

	if owner != "nobody" {
		t.Fatalf("Expected owner '%s', got '%s'", "nobody", owner)
	}
}
