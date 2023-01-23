// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package mountsdk_test

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/mountsdk"
	"cros.local/bazel/ebuild/private/common/processes"
	"github.com/bazelbuild/rules_go/go/tools/bazel"
)

func TestRunInSdk(t *testing.T) {
	ctx := context.Background()

	getRunfile := func(runfile string) string {
		path, err := bazel.Runfile(runfile)
		if err != nil {
			t.Fatal(err)
		}
		return path
	}

	helloFile := filepath.Join(t.TempDir(), "hello")
	if err := os.WriteFile(helloFile, []byte("hello"), 0755); err != nil {
		t.Fatal(err)
	}

	// These values were obtained by looking at an invocation of build_package.
	portageStable := filepath.Join(mountsdk.SourceDir, "src/third_party/portage-stable")
	ebuildFile := filepath.Join(portageStable, "mypkg/mypkg.ebuild")
	cfg := mountsdk.Config{
		Overlays: []makechroot.OverlayInfo{
			{
				ImagePath: getRunfile("bazel/sdk/arm64-generic"),
				MountDir:  "/",
			},
			{
				ImagePath: getRunfile("bazel/sdk/arm64-generic.symindex"),
				MountDir:  "/",
			},
			{
				ImagePath: getRunfile("bazel/sdk/base_sdk"),
				MountDir:  "/",
			},
			{
				ImagePath: getRunfile("bazel/sdk/base_sdk.symindex"),
				MountDir:  "/",
			},
			{
				ImagePath: getRunfile("overlays/overlay-arm64-generic/overlay-arm64-generic.squashfs"),
				MountDir:  filepath.Join(mountsdk.SourceDir, "src/overlays/overlay-arm64-generic"),
			},
			{
				ImagePath: getRunfile("third_party/eclass-overlay/eclass-overlay.squashfs"),
				MountDir:  filepath.Join(mountsdk.SourceDir, "src/third_party/eclass-overlay"),
			},
			{
				ImagePath: getRunfile("third_party/chromiumos-overlay/chromiumos-overlay.squashfs"),
				MountDir:  filepath.Join(mountsdk.SourceDir, "src/third_party/chromiumos-overlay"),
			},
			{
				ImagePath: getRunfile("third_party/portage-stable/portage-stable.squashfs"),
				MountDir:  portageStable,
			},
		},
		BindMounts: []makechroot.BindMount{
			{
				Source:    getRunfile("bazel/ebuild/private/common/mountsdk/testdata/mypkg.ebuild"),
				MountPath: ebuildFile,
			},
			{
				Source:    helloFile,
				MountPath: "/hello",
			},
		},
		Remounts: []string{filepath.Join(portageStable, "mypkg")},
	}

	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		if err := processes.Run(ctx, s.Command("true")); err != nil {
			t.Errorf("Failed to run true: %v", err)
		}
		return nil
	}); err != nil {
		t.Error(err)
	}

	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		if err := processes.Run(ctx, s.Command("false")); err == nil {
			t.Error("The command 'false' unexpectedly succeeded")
		}
		return nil
	}); err != nil {
		t.Error(err)
	}

	// Should be a read-only mount.
	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		if err := processes.Run(ctx, s.Command("/bin/bash", "-c", "echo world > /hello")); err == nil {
			t.Error("Writing to the mount '/hello' succeeded, but it should be read-only")
		}
		return nil
	}); err != nil {
		t.Error(err)
	}
	contents, err := os.ReadFile(helloFile)
	if err != nil {
		t.Error(err)
	}
	if string(contents) != "hello" {
		t.Error("Chroot unexpectedly wrote to a mount that should be read-only")
	}

	// Check we're in the SDK by using a binary unlikely to be on the host machine.
	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		if err := processes.Run(ctx, s.Command("test", "-f", "/usr/bin/ebuild")); err != nil {
			t.Errorf("Failed to find /usr/bin/ebuild: %v", err)
		}
		return nil
	}); err != nil {
		t.Error(err)
	}

	// Confirm that overlays were loaded in to the SDK.
	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		if err := processes.Run(ctx, s.Command("test", "-d", filepath.Join(portageStable, "eclass"))); err != nil {
			t.Errorf("Failed to find the eclass directory: %v", err)
		}
		return nil
	}); err != nil {
		t.Error(err)
	}

	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		if err := processes.Run(ctx, s.Command("grep", "EBUILD_CONTENTS", ebuildFile)); err != nil {
			t.Errorf("Failed to grep the ebuild file: %v", err)
		}
		return nil
	}); err != nil {
		t.Error(err)
	}

	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		outPkg := s.RootDir.Add("build/arm64-generic/packages/mypkg")
		if err := os.MkdirAll(outPkg.Outside(), 0755); err != nil {
			t.Error(err)
		}
		outFile := outPkg.Add("mpkg.tbz2")

		if err := processes.Run(ctx, s.Command("touch", outFile.Inside())); err != nil {
			t.Errorf("Failed to touch %s: %v", outFile.Inside(), err)
		}
		hostOutFile := filepath.Join(s.DiffDir, outFile.Inside())
		if _, err := os.Stat(hostOutFile); err != nil {
			t.Errorf("Expected %s to exist: %v", hostOutFile, err)
		}
		return nil
	}); err != nil {
		t.Error(err)
	}
}
