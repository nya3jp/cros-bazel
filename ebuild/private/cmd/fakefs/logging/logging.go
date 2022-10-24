// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package logging

import (
	"fmt"
	"os"

	"cros.local/bazel/ebuild/private/cmd/fakefs/syscallabi"
)

type Thread struct {
	Tid int
	Pid int
}

type GlobalLogger struct {
	verbose       bool
	currentThread *Thread
}

func New(verbose bool) *GlobalLogger {
	return &GlobalLogger{
		verbose: verbose,
	}
}

func (l *GlobalLogger) Flush() {
	if l.currentThread == nil {
		return
	}
	if l.verbose {
		fmt.Fprintf(os.Stderr, "<unfinished...>\n")
	}
	l.currentThread = nil
}

func (l *GlobalLogger) Printf(format string, args ...interface{}) {
	l.Flush()
	if l.verbose {
		fmt.Fprintf(os.Stderr, format+"\n", args...)
	}
}

func (l *GlobalLogger) StartSyscall(t *Thread, nr int) {
	l.Flush()
	if l.verbose {
		fmt.Fprintf(os.Stderr, "%s %s(...) = ", header(t), syscallabi.Name(nr))
	}
	l.currentThread = t
}

func (l *GlobalLogger) FinishSyscall(t *Thread, nr int, ret int64) {
	if l.currentThread != nil && l.currentThread.Tid == t.Tid {
		if l.verbose {
			fmt.Fprintf(os.Stderr, "%d\n", ret)
		}
		l.currentThread = nil
		return
	}

	l.Flush()
	if l.verbose {
		fmt.Fprintf(os.Stderr, "%s <...resumed> %s(...) = %d\n", header(t), syscallabi.Name(nr), ret)
	}
}

func (l *GlobalLogger) ForThread(t *Thread) *ThreadLogger {
	return &ThreadLogger{
		logger:         l,
		thread:         t,
		currentSyscall: -1,
	}
}

type ThreadLogger struct {
	logger         *GlobalLogger
	thread         *Thread
	currentSyscall int
}

func (tl *ThreadLogger) Printf(format string, args ...interface{}) {
	format = "%s " + format
	args = append([]interface{}{header(tl.thread)}, args...)
	tl.logger.Printf(format, args...)
}

func (tl *ThreadLogger) StartSyscall(nr int) {
	tl.logger.StartSyscall(tl.thread, nr)
	tl.currentSyscall = nr
}

func (tl *ThreadLogger) FinishSyscall(ret int64) {
	tl.logger.FinishSyscall(tl.thread, tl.currentSyscall, ret)
	tl.currentSyscall = -1
}

func header(t *Thread) string {
	return fmt.Sprintf("[thread %d; process %d]", t.Tid, t.Pid)
}
