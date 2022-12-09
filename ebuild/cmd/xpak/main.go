// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"os"

	"cros.local/bazel/ebuild/private/common/cliutil"
	"github.com/urfave/cli/v2"
)

var app = &cli.App{
	Commands: []*cli.Command{
		{
			Name:    "split",
			Aliases: []string{"s"},
			Usage:   "Splits a portage package (tbz2) into a .xpak and .tar.X archive",
			Flags: []cli.Flag{
				&cli.BoolFlag{
					Name:    "extract",
					Aliases: []string{"e"},
					Usage:   "Extracts the file contents contained in the tbz2",
				},
				&cli.PathFlag{
					Name:    "dest",
					Aliases: []string{"d"},
					Usage: "Destination directory to extract contents. By default the contents will be" +
						" extracted to the same directory as the input file.",
				},
			},
			Action: func(cCtx *cli.Context) error {
				return splitCmd(cCtx.Context,
					cCtx.Bool("extract"),
					cCtx.String("dest"),
					cCtx.Args().Slice())
			},
		},
	},
}

func main() {
	// Bazel will change the CWD to the runfile directory. We don't want this
	// behavior for a CLI app.
	if cwd, ok := os.LookupEnv("BUILD_WORKING_DIRECTORY"); ok {
		if err := os.Chdir(cwd); err != nil {
			cliutil.Exit(err)
		}
	}
	cliutil.Exit(app.Run(os.Args))
}
