// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package reporter_test

import (
	"bytes"
	"fmt"
	"math/rand"
	"regexp"
	"strings"
	"sync"
	"testing"

	"cros.local/bazel/portage/bin/auditfuse/reporter"
)

func TestReporter(t *testing.T) {
	var buf bytes.Buffer
	r := reporter.New(&buf, false)
	r.Report(reporter.Lookup, "/aaa")
	r.Report(reporter.Readdir, "/aaa")
	r.Report(reporter.Lookup, "/bbb")
	r.Report(reporter.Readdir, "/ccc")
	r.Report(reporter.Lookup, "/aaa")
	r.Report(reporter.Readdir, "/ccc")

	const want = "LOOKUP\t/aaa\x00READDIR\t/aaa\x00LOOKUP\t/bbb\x00READDIR\t/ccc\x00"
	got := buf.String()
	if got != want {
		t.Fatalf("Report result mismatch: got %v, want %v", got, want)
	}
}

func TestReporter_Concurrency(t *testing.T) {
	const workers = 10
	const reportPerWorker = 100000

	var buf bytes.Buffer
	r := reporter.New(&buf, false)

	// Run N goroutines making reports concurrently.
	var wg sync.WaitGroup
	for i := 0; i < workers; i++ {
		go func() {
			defer wg.Done()
			for j := 0; j < reportPerWorker; j++ {
				path := fmt.Sprintf("/%09d", rand.Intn(1000000000))
				r.Report(reporter.Lookup, path)
			}
		}()
		wg.Add(1)
	}
	wg.Wait()

	// Verify that the output is not corrupted.
	lines := strings.Split(buf.String(), "\x00")
	if lines[len(lines)-1] != "" {
		t.Errorf("Output must end with a null byte")
	}

	pattern := regexp.MustCompile(`^LOOKUP\t/\d{9}$`)
	for _, line := range lines[:len(lines)-1] {
		if !pattern.MatchString(line) {
			t.Fatalf("Invalid line: %#v", line)
		}
	}
}
