// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// auditfuse is a FUSE file system that audits file accesses on a read-only view
// of another directory.
package main

import (
	"fmt"
	"os"

	"github.com/hanwen/go-fuse/v2/fs"
	"github.com/hanwen/go-fuse/v2/fuse"
	"github.com/urfave/cli/v2"

	"cros.local/bazel/portage/bin/auditfuse/daemonize"
	"cros.local/bazel/portage/bin/auditfuse/fsimpl"
	"cros.local/bazel/portage/bin/auditfuse/reporter"
)

var flagOutput = &cli.StringFlag{
	Name:     "output",
	Aliases:  []string{"o"},
	Usage:    "output audit file path",
	Required: true,
}

var flagForeground = &cli.BoolFlag{
	Name:    "foreground",
	Aliases: []string{"f"},
	Usage:   "run in foreground",
}

var flagVerbose = &cli.BoolFlag{
	Name:    "verbose",
	Aliases: []string{"v"},
	Usage:   "enable verbose logging (needs -f)",
}

var flagDebug = &cli.BoolFlag{
	Name:    "debug",
	Aliases: []string{"d"},
	Usage:   "enable FUSE debug logging (needs -f)",
}

var app = &cli.App{
	Usage:     "FUSE filesystem that audits file access",
	ArgsUsage: "orig-dir mount-dir",
	Flags: []cli.Flag{
		flagOutput,
		flagForeground,
		flagVerbose,
		flagDebug,
	},
	HideHelpCommand: true,
	Action: func(c *cli.Context) error {
		outPath := c.String(flagOutput.Name)
		foreground := c.Bool(flagForeground.Name)
		verbose := c.Bool(flagVerbose.Name)
		debug := c.Bool(flagDebug.Name)
		args := c.Args().Slice()
		if len(args) != 2 {
			cli.ShowAppHelpAndExit(c, 1)
		}
		origDir := args[0]
		mountDir := args[1]

		if !foreground {
			if exit, err := daemonize.Start(); err != nil {
				return err
			} else if exit {
				return nil
			}
		}

		out, err := os.Create(outPath)
		if err != nil {
			return err
		}
		defer out.Close()

		root, err := fsimpl.NewRoot(origDir, reporter.New(out, verbose))
		if err != nil {
			return err
		}

		server, err := fs.Mount(mountDir, root, &fs.Options{
			NullPermissions: true,
			MountOptions: fuse.MountOptions{
				AllowOther:        true,
				Name:              "auditfuse",
				FsName:            origDir,
				DirectMountStrict: true,
				Debug:             debug,
			},
		})
		if err != nil {
			return err
		}

		if !foreground {
			daemonize.Finish()
		}

		server.Wait()
		return nil
	},
}

func main() {
	if err := app.Run(os.Args); err != nil {
		fmt.Fprintf(os.Stderr, "[auditfuse] FATAL: %v\n", err)
		os.Exit(1)
	}
}
