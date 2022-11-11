package mountsdk_test

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"cros.local/bazel/ebuild/private/common/mountsdk"
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

	// These values were obtained by looking at an invocation of build_package.
	portageStable := filepath.Join(mountsdk.SourceDir, "src/third_party/portage-stable")
	ebuildFile := filepath.Join(portageStable, "mypkg/mypkg.ebuild")
	cfg := mountsdk.Config{
		Overlays: []mountsdk.MappedDualPath{
			{
				HostPath: getRunfile("bazel/sdk/sdk"),
				SDKPath:  "/",
			},
			{
				HostPath: getRunfile("bazel/sdk/sdk.symindex"),
				SDKPath:  "/",
			},
			{
				HostPath: getRunfile("bazel/sdk/base_sdk"),
				SDKPath:  "/",
			},
			{
				HostPath: getRunfile("bazel/sdk/base_sdk.symindex"),
				SDKPath:  "/",
			},
			{
				HostPath: getRunfile("overlays/overlay-arm64-generic/overlay-arm64-generic.squashfs"),
				SDKPath:  filepath.Join(mountsdk.SourceDir, "src/overlays/overlay-arm64-generic"),
			},
			{
				HostPath: getRunfile("third_party/eclass-overlay/eclass-overlay.squashfs"),
				SDKPath:  filepath.Join(mountsdk.SourceDir, "src/third_party/eclass-overlay"),
			},
			{
				HostPath: getRunfile("third_party/chromiumos-overlay/chromiumos-overlay.squashfs"),
				SDKPath:  filepath.Join(mountsdk.SourceDir, "src/third_party/chromiumos-overlay"),
			},
			{
				HostPath: getRunfile("third_party/portage-stable/portage-stable.squashfs"),
				SDKPath:  portageStable,
			},
		},
		Remounts: []string{filepath.Join(portageStable, "mypkg")},
		CopyToSDK: []mountsdk.MappedDualPath{
			{
				HostPath: getRunfile("bazel/ebuild/private/common/mountsdk/testdata/mypkg.ebuild"),
				SDKPath:  ebuildFile,
			},
		},
	}

	if err := mountsdk.RunInSDK(&cfg, func(s *mountsdk.MountedSDK) error {
		if err := s.Command(ctx, "false").Run(); err == nil {
			t.Error("The command 'false' unexpectedly succeeded")
		}

		outPkg := s.RootDir.Add("build/arm64-generic/packages/mypkg")
		if err := os.MkdirAll(outPkg.Outside(), 0755); err != nil {
			t.Error(err)
		}
		outFile := outPkg.Add("mpkg.tbz2")

		for _, cmd := range []*exec.Cmd{
			s.Command(ctx, "true"),
			// Check we're in the SDK by using a binary unlikely
			// to be on the host machine.
			s.Command(ctx, "test", "-f", "/usr/bin/ebuild"),
			// Confirm that overlays were loaded in to the SDK.
			s.Command(ctx, "test", "-d", filepath.Join(portageStable, "eclass")),
			s.Command(ctx, "test", "-d", outPkg.Inside()),
			s.Command(ctx, "test", "-f", ebuildFile),
			s.Command(ctx, "grep", "EBUILD_CONTENTS", ebuildFile),
			s.Command(ctx, "touch", outFile.Inside()),
		} {
			if err := cmd.Run(); err != nil {
				t.Errorf("Failed to run %s: %v", strings.Join(cmd.Args, " "), err)
			}
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
