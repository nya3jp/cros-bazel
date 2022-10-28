// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package tracer

import (
	"sort"

	"golang.org/x/sys/unix"
)

type threadState struct {
	Tid             int
	Pid             int
	SyscallExitHook func(regs *unix.PtraceRegsAmd64)
}

type threadStateIndex struct {
	threadByTid map[int]*threadState
	threadByPid map[int]map[int]*threadState
}

func newThreadStateIndex() *threadStateIndex {
	return &threadStateIndex{
		threadByTid: make(map[int]*threadState),
		threadByPid: make(map[int]map[int]*threadState),
	}
}

func (ti *threadStateIndex) GetByTid(tid int) *threadState {
	return ti.threadByTid[tid]
}

func (ti *threadStateIndex) GetByPid(pid int) []*threadState {
	var threads []*threadState
	for _, t := range ti.threadByPid[pid] {
		threads = append(threads, t)
	}
	sort.Slice(threads, func(i, j int) bool {
		return threads[i].Tid < threads[j].Tid
	})
	return threads
}

func (ti *threadStateIndex) Put(t *threadState) {
	ti.threadByTid[t.Tid] = t
	if ti.threadByPid[t.Pid] == nil {
		ti.threadByPid[t.Pid] = make(map[int]*threadState)
	}
	ti.threadByPid[t.Pid][t.Tid] = t
}

func (ti *threadStateIndex) Remove(t *threadState) {
	delete(ti.threadByTid, t.Tid)
	delete(ti.threadByPid[t.Pid], t.Tid)
	if len(ti.threadByPid[t.Pid]) == 0 {
		delete(ti.threadByPid, t.Pid)
	}
}
