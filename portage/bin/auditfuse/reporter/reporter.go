// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package reporter

import (
	"fmt"
	"io"
	"os"
	"sync"
)

type AccessType string

const (
	Lookup  AccessType = "LOOKUP"
	Readdir AccessType = "READDIR"
)

type entry struct {
	Type AccessType `json:"type"`
	Path string     `json:"path"`
}

type Reporter struct {
	out     io.Writer
	verbose bool

	mu   sync.RWMutex
	seen map[entry]struct{} // protected by mu
}

func (r *Reporter) Report(t AccessType, path string) error {
	if r.verbose {
		fmt.Fprintf(os.Stderr, "[auditfuse] %s: %s\n", t, path)
	}

	e := entry{
		Type: t,
		Path: path,
	}

	r.mu.RLock()
	_, ok := r.seen[e]
	r.mu.RUnlock()
	if ok {
		return nil
	}

	r.mu.Lock()
	_, err := fmt.Fprintf(r.out, "%s\t%s\x00", t, path)
	r.seen[e] = struct{}{}
	r.mu.Unlock()
	return err
}

func New(out io.Writer, verbose bool) *Reporter {
	return &Reporter{
		out:     out,
		verbose: verbose,
		seen:    make(map[entry]struct{}),
	}
}
