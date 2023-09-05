// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/portage/bin/fakefs/exit"
)

var cmdFstatEmptyPath = &cli.Command{
	Name:  "fstatat-empty-path",
	Usage: "Calls fstatat with AT_EMPTY_PATH",
	Action: func(c *cli.Context) error {
		args := c.Args().Slice()
		if len(args) != 1 {
			return errors.New("needs exactly 1 path")
		}
		path := args[0]

		dirfd, err := unix.Open(path, unix.O_RDONLY|unix.O_PATH, 0)
		if err != nil {
			return err
		}
		defer unix.Close(dirfd)

		var stat unix.Stat_t
		if err := unix.Fstatat(dirfd, "", &stat, unix.AT_EMPTY_PATH); err != nil {
			return err
		}
		fmt.Printf("%d:%d", stat.Uid, stat.Gid)
		return nil
	},
}

var cmdStatProcSelfFD = &cli.Command{
	Name:  "stat-proc-self-fd",
	Usage: "Calls stat via /proc/self/fd",
	Action: func(c *cli.Context) error {
		args := c.Args().Slice()
		if len(args) != 1 {
			return errors.New("needs exactly 1 path")
		}
		path := args[0]

		f, err := os.Open(path)
		if err != nil {
			return err
		}
		defer f.Close()

		var stat unix.Stat_t
		if err := unix.Stat(fmt.Sprintf("/proc/self/fd/%d", f.Fd()), &stat); err != nil {
			return err
		}

		fmt.Printf("%d:%d", stat.Uid, stat.Gid)
		return nil
	},
}

var app = &cli.App{
	Usage: "A helper binary used in fakefs tests",
	Commands: []*cli.Command{
		cmdFstatEmptyPath,
		cmdStatProcSelfFD,
	},
}

func main() {
	exit.Exit(app.Run(os.Args))
}
