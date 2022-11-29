package host_test

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"testing"

	"cros.local/bazel/ebuild/private/common/hostcontainercomms/host"
	"cros.local/bazel/ebuild/private/common/makechroot"
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

	cfg := mountsdk.Config{
		Overlays: []makechroot.OverlayInfo{
			{
				ImagePath: getRunfile("bazel/sdk/sdk"),
				MountDir:  "/",
			},
			{
				ImagePath: getRunfile("bazel/sdk/sdk.symindex"),
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
		},
	}

	if err := host.RunInSDKWithServer(ctx, &cfg, func(s *mountsdk.MountedSDK) error {
		for _, tc := range []struct {
			cmd *exec.Cmd
			out string
		}{
			{cmd: s.Command(ctx, "test", "-f", host.PrepClientFile)},
			// If this doesn't exist, prep client wasn't executed properly.
			{cmd: s.Command(ctx, "test", "-f", "/helpers/host_uid")},

			// Both root, but one refers to the real root while the other refers to
			// the container root.
			{
				cmd: s.Command(ctx, "whoami"),
				out: "root\n",
			},
			{
				cmd: s.Command(ctx, host.RunAsRoot, "whoami"),
				out: "root\n",
			},

			{
				cmd: s.Command(ctx, "stat", "-c", "%U", "/"),
				out: "root\n",
			},
			// Inside the mount namespace we should be able to correctly see the real
			// UID, but be unable to map it to a real user.
			{
				cmd: s.Command(ctx, host.RunAsRoot, "stat", "-c", "%u", "/"),
				out: fmt.Sprintf("%d\n", os.Getuid()),
			},

			// /dev should be owned by real root.
			{
				cmd: s.Command(ctx, "stat", "-c", "%U", "/dev"),
				out: "nobody\n",
			},
			{
				cmd: s.Command(ctx, host.RunAsRoot, "stat", "-c", "%U", "/dev"),
				out: "root\n",
			},
		} {
			var stderrBuffer bytes.Buffer
			tc.cmd.Stderr = io.MultiWriter(tc.cmd.Stderr, &stderrBuffer)

			args := strings.Join(tc.cmd.Args, " ")
			if err := tc.cmd.Run(); err != nil {
				t.Errorf("Failed to run %s: %v", args, err)
			}

			// mountsdk/setup.sh is run with -x, and merges both stdout and stderr to
			// stderr.
			// We need to filter out lines created by -x, which is lines starting with
			// '+ '.
			var filteredLines []string
			for _, line := range strings.Split(string(stderrBuffer.Bytes()), "\n") {
				if !strings.HasPrefix(line, "+ ") {
					filteredLines = append(filteredLines, line)
				}
			}
			gotStdout := strings.Join(filteredLines, "\n")

			if gotStdout != tc.out {
				t.Errorf("Incorrect output for %s: got %q, want %q", args, gotStdout, tc.out)
			}
		}
		return nil
	}); err != nil {
		t.Error(err)
	}
}
