// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package logging

import (
	"fmt"
	"os"

	"github.com/alessio/shellescape"
)

type Logger struct {
	verbose    bool
	args       []string
	intercepts uint64
}

func NewLogger(verbose bool, args []string) *Logger {
	return &Logger{
		verbose:    verbose,
		args:       args,
		intercepts: 0,
	}
}

func (l *Logger) printf(tid int, format string, args ...interface{}) {
	header := fmt.Sprintf("[fakefs %d] ", tid)
	fmt.Fprintf(os.Stderr, header+format+"\n", args...)
}

func (l *Logger) Infof(tid int, format string, args ...interface{}) {
	if !l.verbose {
		return
	}
	l.printf(tid, format, args...)
}

func (l *Logger) Errorf(tid int, format string, args ...interface{}) {
	l.printf(tid, format, args...)
}

func (l *Logger) RecordIntercept() {
	l.intercepts++
}

func (l *Logger) PrintStats() {
	fmt.Fprintf(os.Stderr, "[fakefs] intercepted %d syscalls: %s\n", l.intercepts, shellescape.QuoteCommand(l.args))
}
