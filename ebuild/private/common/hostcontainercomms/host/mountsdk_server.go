// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package host

import (
	"context"
	_ "embed"
	"fmt"
	"os"
	"path/filepath"
	"strconv"

	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/mountsdk"
	"github.com/bazelbuild/rules_go/go/tools/bazel"
)

const (
	PrepClientFile = "/helpers/prep_client"
	RunAsRoot      = "/helpers/run_as_root"
)

func RunInSDKWithServer(ctx context.Context, cfg *mountsdk.Config, action mountsdk.Action) error {
	for _, file := range []struct {
		sdkPath, resource string
	}{
		{PrepClientFile, "bazel/ebuild/private/common/hostcontainercomms/container/prep_client.sh"},
		{RunAsRoot, "bazel/ebuild/private/common/hostcontainercomms/container/run_as_root/run_as_root_/run_as_root"},
	} {
		hostPath, err := bazel.Runfile(file.resource)
		if err != nil {
			return err
		}

		cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
			Source:    hostPath,
			MountPath: file.sdkPath,
		})
	}
	temp, err := os.MkdirTemp("", "build_image_pid")
	if err != nil {
		return err
	}
	defer os.RemoveAll(temp)
	pidFile := filepath.Join(temp, "dumb_init_pid")
	cfg.RunInContainerExtraArgs = append(cfg.RunInContainerExtraArgs, "--same-network", fmt.Sprintf("--pid-file=%s", pidFile))

	server, err := StartServer(ctx, pidFile)
	if err != nil {
		return err
	}
	defer server.Close()

	return mountsdk.RunInSDK(cfg, func(s *mountsdk.MountedSDK) error {
		cmd := s.Command(ctx, PrepClientFile, strconv.Itoa(os.Getuid()), strconv.Itoa(os.Getgid()), fmt.Sprintf("localhost:%d", server.Port()))
		if err := cmd.Run(); err != nil {
			return err
		}
		return action(s)
	})
}
