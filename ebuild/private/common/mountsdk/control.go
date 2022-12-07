// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package mountsdk

import (
	"context"
	"errors"
	"log"
	"os"
	"os/signal"

	"golang.org/x/sys/unix"
)

func tcsetpgrp(fd int, pgid int) error {
	return unix.IoctlSetPointerInt(fd, unix.TIOCSPGRP, pgid)
}

func resetControllingTerminal() error {
	pgid := unix.Getpgrp()

	// SIGTTOU will be generated when a background processes tries to write to
	// the terminal. Since we are writing a new tpgid to the terminal, we need to
	// ignore SIGTTOU, otherwise we get suspended.
	signal.Ignore(unix.SIGTTOU)
	defer signal.Reset(unix.SIGTTOU)

	return tcsetpgrp(0, pgid)
}

func handleData(data byte) {
	if data == 't' {
		err := resetControllingTerminal()
		if err != nil {
			log.Println("Failed to update terminal pgid: ", err)
		}
	} else {
		log.Println("Unknown control command:", data)
	}
}

func fifoToChan(ctx context.Context, fifoPath string) (<-chan byte, error) {
	// We open RDWR so that we always keep a write handle to the FIFO. This
	// makes the open call not block waiting for a writer to open the FIFO. It
	// also allows writers to open/close the FIFO without causing the reader (us)
	// to close.
	fifo, err := os.OpenFile(fifoPath, os.O_RDWR, 0)
	if err != nil {
		return nil, err
	}

	bytes := make(chan byte)
	go func() {
		defer close(bytes)

		buf := make([]byte, 1)
		for {
			cnt, err := fifo.Read(buf)
			if errors.Is(err, os.ErrClosed) || cnt == 0 {
				return
			} else if err != nil {
				log.Println("Error reading from control FIFO:", err)
				return
			}

			bytes <- buf[0]
		}
	}()

	go func() {
		<-ctx.Done()
		// This will cause the bytes channel to close
		fifo.Close()
	}()

	return bytes, err
}

func StartControlChannel(fifoPath string) (func(), error) {
	ctx, cancel := context.WithCancel(context.Background())
	stopped := make(chan struct{})

	if err := unix.Mkfifo(fifoPath, 0o666); err != nil {
		return nil, err
	}

	bytes, err := fifoToChan(ctx, fifoPath)
	if err != nil {
		return nil, err
	}

	go func() {
		defer close(stopped)

		for {
			select {
			case b, ok := <-bytes:
				if ok {
					handleData(b)
				} else {
					return
				}
			}
		}
	}()

	return func() {
		cancel()

		<-stopped
	}, nil
}
