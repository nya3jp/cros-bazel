// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"archive/tar"
	"bytes"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"strings"

	"github.com/urfave/cli"
)

var flagOutput = &cli.StringFlag{
	Name:     "output",
	Required: true,
}

var flagSpecsFrom = &cli.StringFlag{
	Name: "specs-from",
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagOutput,
		flagSpecsFrom,
	},
	Action: func(c *cli.Context) error {
		outputPath := c.String(flagOutput.Name)
		specsFile := c.String(flagSpecsFrom.Name)
		specs := []string(c.Args())

		if specsFile != "" {
			b, err := os.ReadFile(specsFile)
			if err != nil {
				return err
			}
			specs = append(specs, strings.Split(strings.TrimRight(string(b), "\n"), "\n")...)
		}

		cmd := exec.Command("/usr/bin/mksquashfs", "-", outputPath, "-tar", "-noappend", "-all-time", "0", "-comp", "lz4")
		stdin, err := cmd.StdinPipe()
		if err != nil {
			return err
		}
		out := &bytes.Buffer{}
		cmd.Stdout = out
		cmd.Stderr = out
		if err := cmd.Start(); err != nil {
			return err
		}
		defer func() {
			cmd.Process.Kill()
			cmd.Wait()
		}()

		w := tar.NewWriter(stdin)
		for _, spec := range specs {
			if err := processFile(w, spec); err != nil {
				return err
			}
		}
		if err := w.Close(); err != nil {
			return err
		}

		stdin.Close()
		if err := cmd.Wait(); err != nil {
			io.Copy(os.Stderr, out)
			return err
		}
		return nil
	},
}

func writeZeros(w io.Writer, n int64) error {
	buf := make([]byte, 4096)
	for n > 0 {
		size := n
		if size > int64(len(buf)) {
			size = int64(len(buf))
		}
		written, err := w.Write(buf[:size])
		if err != nil {
			return err
		}
		n -= int64(written)
	}
	return nil
}

func processFile(w *tar.Writer, spec string) error {
	v := strings.Split(spec, ":")
	if len(v) != 2 {
		return fmt.Errorf("invalid file spec: %s; maybe file names containing colons?", spec)
	}
	dst := v[0]
	src := v[1]

	// Always follow symlinks.
	stat, err := os.Stat(src)
	if err != nil {
		return err
	}

	switch stat.Mode().Type() {
	case 0:
		w.WriteHeader(&tar.Header{
			Name:     dst,
			Typeflag: tar.TypeReg,
			Size:     stat.Size(),
			Mode:     int64(stat.Mode().Perm()),
		})
		f, err := os.Open(src)
		if err != nil {
			return err
		}
		defer f.Close()

		written, err := io.CopyN(w, f, stat.Size())
		if err != nil {
			return err
		}
		if written < stat.Size() {
			log.Printf("WARNING: short read (got %d, want %d)\n", written, stat.Size())
			if err := writeZeros(w, stat.Size()-written); err != nil {
				return err
			}
		}
		return nil

	case os.ModeDir:
		w.WriteHeader(&tar.Header{
			Name:     dst,
			Typeflag: tar.TypeDir,
			Mode:     int64(stat.Mode().Perm()),
		})
		return nil

	default:
		return fmt.Errorf("unsupported file type %v", stat.Mode().Type())
	}
}

func main() {
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
