// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package equery

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"

	"github.com/alessio/shellescape"

	"cros.local/bazel/ebuild/private/common/standard/version"
)

type Package struct {
	Name    string
	Version *version.Version
	Uses    map[string]bool
}

type Client struct {
	workspaceDir string
	board        string
}

func NewClient(workspaceDir string, board string) *Client {
	return &Client{
		workspaceDir: workspaceDir,
		board:        board,
	}
}

func (c *Client) run(args ...string) (string, error) {
	name := fmt.Sprintf("equery-%s", c.board)
	cmd := exec.Command(name, args...)
	cmd.Stderr = os.Stderr
	cmd.Dir = c.workspaceDir

	log.Printf("Running: %s", shellescape.QuoteCommand(cmd.Args))
	b, err := cmd.Output()
	return string(b), err
}

func (c *Client) ListInstalledPackages() ([]*Package, error) {
	out, err := c.run("list", "*")
	if err != nil {
		return nil, err
	}

	var pkgs []*Package
	for _, line := range strings.Split(strings.TrimRight(out, "\n"), "\n") {
		name, ver, err := version.ExtractSuffix(line)
		if err != nil {
			return nil, err
		}

		out, err := c.run("uses", fmt.Sprintf("=%s-%s", name, ver.String()))
		if err != nil {
			return nil, err
		}

		uses := make(map[string]bool)
		for _, use := range strings.Split(strings.TrimRight(out, "\n"), "\n") {
			value := strings.HasPrefix(use, "+")
			use = strings.TrimLeft(use, "+-")
			uses[use] = value
		}

		pkgs = append(pkgs, &Package{
			Name:    name,
			Version: ver,
			Uses:    uses,
		})
	}

	return pkgs, nil
}
