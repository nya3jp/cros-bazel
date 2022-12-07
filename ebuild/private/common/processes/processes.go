// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package processes

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/exec"
	"os/signal"

	"golang.org/x/sys/unix"
)

func sendSignal(cmd *exec.Cmd, s os.Signal) {
	if err := cmd.Process.Signal(s); err != nil {
		// This might happen if the processes has already terminated.
		log.Printf("Failed to send %s to PID %d: %v\n", s, cmd.Process.Pid, err)
	}
}
func handleSignal(cmd *exec.Cmd, s os.Signal) error {
	switch s {
	case unix.SIGTERM:
		sendSignal(cmd, s)

		return nil
	default:
		return fmt.Errorf("Unexpected signal received: %s", s)
	}
}

// `cmd` must not have been created with `CommandContext` since it will `kill`
// the processes instead of gracefully terminating it.
//   - Forwards SIGTERM to the child processes
//   - Ignores SIGINT while the processes is running. SIGINT is normally generated
//     by the terminal when Ctrl+C is pressed. The signal is sent to all processes
//     in the foreground processes group. This means that the child processes
//     should receive the signal by default so we don't need to forward it. One
//     exception is if the child puts itself into a different processes group, but
//     we want to avoid that.
func Run(ctx context.Context, cmd *exec.Cmd) error {
	signal.Ignore(unix.SIGINT)
	defer signal.Reset(unix.SIGINT)

	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, unix.SIGTERM)
	defer signal.Stop(sigs)

	if err := cmd.Start(); err != nil {
		return err
	}

	errc := make(chan error)
	go func() {
		errc <- cmd.Wait()
	}()

	for {
		// TODO: Should we add a timer that sends a SIGKILL if the
		// processes doesn't respond to SIGTERM? That will mean we will
		// leak any child processes though... It also requires choosing
		// a reasonable timeout value.
		select {
		case s := <-sigs:
			if err := handleSignal(cmd, s); err != nil {
				// We don't exit since we need to clean up
				log.Println(err)
			}
		case <-ctx.Done():
			sendSignal(cmd, unix.SIGTERM)
			// Wait for the processes to terminate
			err := <-errc
			return err
		case err := <-errc:
			return err
		}
	}
}
