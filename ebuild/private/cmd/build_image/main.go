// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	_ "embed"
	"fmt"
	"log"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/makechroot"
	"cros.local/bazel/ebuild/private/common/processes"
	"github.com/bazelbuild/rules_go/go/runfiles"
	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/mountsdk"
)

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagOutput = &cli.StringFlag{
	Name:     "output",
	Required: true,
}

const mainScript = "/mnt/host/bazel-build/build_image.sh"

type resourceMount struct {
	resource  string
	mountPath string
}

var app = &cli.App{
	Flags: append(mountsdk.CLIFlags,
		flagBoard,
		mountsdk.FlagInstallTarget,
		flagOutput,
	),
	Action: func(c *cli.Context) error {
		board := c.String(flagBoard.Name)
		finalOutPath := c.String(flagOutput.Name)
		installTargetsUnparsed := c.StringSlice(mountsdk.FlagInstallTarget.Name)

		// We need "supports-graceful-termination" execution requirement in the
		// build action to let Bazel send SIGTERM instead of SIGKILL.
		ctx, cancel := signal.NotifyContext(c.Context, unix.SIGINT, unix.SIGTERM)
		defer cancel()
		c.Context = ctx

		cfg, err := mountsdk.GetMountConfigFromCLI(c)
		if err != nil {
			return err
		}

		resourceMounts := []resourceMount{
			{
				mountPath: filepath.Join("/build", board, "var/cache/edb/chromeos"),
				resource:  "chromiumos/bazel/ebuild/private/cmd/build_image/container_files/edb_chromeos",
			},
			{
				mountPath: filepath.Join("/build", board, "etc/portage/profile/package.provided"),
				resource:  "chromiumos/bazel/ebuild/private/cmd/build_image/container_files/package.provided",
			},
			{
				mountPath: "/mnt/host/bazel-build/install_deps.sh",
				resource:  "chromiumos/bazel/ebuild/private/cmd/install_deps/install_deps.sh",
			},
			{
				mountPath: mainScript,
				resource:  "chromiumos/bazel/ebuild/private/cmd/build_image/container_files/build_image.sh",
			},
		}

		// base_image_util calls install_libc_for_abi, which expects certain
		// cross-compilation tools to be stored at specific locations.
		// TODO: Once we can build with custom use flags, stop hardcoding aarch64.

		// It's possible not all of these packages are needed. I may remove some
		// later if we find out they're never needed throughout the whole
		// build_image process.
		for _, resource := range []string{
			"amd64_host_binutils_2_36_1_r8/file/binutils-2.36.1-r8.tbz2",
			"amd64_host_compiler_rt_15_0_pre458507_r6/file/compiler-rt-15.0_pre458507-r6.tbz2",
			"amd64_host_gcc_10_2_0_r28/file/gcc-10.2.0-r28.tbz2",
			"amd64_host_gdb_9_2_20200923_r9/file/gdb-9.2.20200923-r9.tbz2",
			"amd64_host_glibc_2_33_r17/file/glibc-2.33-r17.tbz2",
			"amd64_host_go_1_18_r2/file/go-1.18-r2.tbz2",
			"amd64_host_libcxx_15_0_pre458507_r6/file/libcxx-15.0_pre458507-r6.tbz2",
			"amd64_host_libxcrypt_4_4_28_r1/file/libxcrypt-4.4.28-r1.tbz2",
			"amd64_host_linux_headers_4_14_r56/file/linux-headers-4.14-r56.tbz2",
			"amd64_host_llvm_libunwind_15_0_pre458507_r4/file/llvm-libunwind-15.0_pre458507-r4.tbz2",
		} {
			// TODO: install_libc hardcodes arm64 to also install the arm32 packages.
			// This is required only if nacl is used.
			// For now, install_libc succeeds if we comment out this hardcoding.
			// Once we can build with custom use flags, we can then support this
			// properly.
			// https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/scripts/build_library/base_image_util.sh;l=272-278;drc=cdaf1eab71d4e607239ccc9db877ff2a22f8568e
			resourceMounts = append(resourceMounts, resourceMount{
				mountPath: filepath.Join("/var/lib/portage/pkgs/cross-aarch64-cros-linux-gnu", filepath.Base(resource)),
				resource:  resource})
		}

		for _, resourceMount := range resourceMounts {
			path, err := runfiles.Rlocation(resourceMount.resource)
			if err != nil {
				return err
			}
			cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
				Source:    path,
				MountPath: resourceMount.mountPath,
			})
		}

		targetPackagesDir := filepath.Join("/build", board, "packages")
		// AddInstallTargetsToConfig returns a set of environment variables for the
		// packages you want to install. We want to drop this to avoid calling
		// emerge on every package which we know is already installed.
		env, err := mountsdk.AddInstallTargetsToConfig(installTargetsUnparsed, targetPackagesDir, cfg)
		if err != nil {
			return err
		}

		if err := mountsdk.RunInSDK(cfg, func(s *mountsdk.MountedSDK) error {
			// setup_board.sh creates emerge-{board} and portageq-{board}, both of
			// which are used by build_image.sh
			boardTemplatePath, err := runfiles.Rlocation("chromiumos/bazel/ebuild/private/cmd/build_image/container_files/board_script.sh")
			if err != nil {
				return err
			}
			boardScriptTemplate, err := os.ReadFile(boardTemplatePath)
			if err != nil {
				return err
			}
			// TODO: stop hardcoding aarch64-cros-linux-gnu.
			boardScript := strings.ReplaceAll(
				strings.ReplaceAll(string(boardScriptTemplate), "${BOARD}", board),
				"${CHOST}", "aarch64-cros-linux-gnu")

			if err := s.WriteFile(fmt.Sprintf("/usr/bin/emerge-%s", board), []byte(strings.ReplaceAll(boardScript, "${COMMAND}", "emerge --root-deps")), 0755); err != nil {
				return err
			}
			if err := s.WriteFile(fmt.Sprintf("/usr/bin/portageq-%s", board), []byte(strings.ReplaceAll(boardScript, "${COMMAND}", "portageq")), 0755); err != nil {
				return err
			}

			args := append([]string{
				mainScript,
				fmt.Sprintf("--board=%s", board),
				// TODO: at some point, we should support a variety of image types
				"base",
			},
				c.Args().Slice()...)

			cmd := s.Command(args[0], args[1:]...)
			cmd.Env = append(cmd.Env, append(env,
				fmt.Sprintf("BOARD=%s", board),
				fmt.Sprintf("HOST_UID=%d", os.Getuid()),
				fmt.Sprintf("HOST_GID=%d", os.Getgid()))...)
			// I have no idea why, but I happened to be trying to run this in a nested
			// namespace initially, and when I tried to remove it, discovered that
			// run_in_container only works inside a mount namespace if you're running
			// as sudo.
			cmd.Args = append([]string{"/usr/bin/sudo", "--preserve-env", "unshare", "--mount", "--"}, cmd.Args...)
			cmd.Path = cmd.Args[0]
			if err := processes.Run(ctx, cmd); err != nil {
				return fmt.Errorf("Failed to run %s: %v", strings.Join(args, " "), err)
			}

			path := filepath.Join("/mnt/host/source/src/build/images/", board, "latest/chromiumos_base_image.bin")
			return fileutil.Copy(filepath.Join(s.DiffDir, path), finalOutPath)
		}); err != nil {
			if err, ok := err.(*exec.ExitError); ok {
				os.Exit(err.ExitCode())
			}
			return err
		}

		return nil
	},
}

func main() {
	bazelutil.FixRunfilesEnv()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
