# ChromeOS Bazelification

This is an experiment to build ChromeOS with Bazel.

## Checking out

For the prototyping phase, we're working on building a snapshot of ChromiumOS.
Use `repo` to check out a snapshotted ChromiumOS tree + Bazel files.

```sh
$ mkdir cros-bazel
$ cd cros-bazel
$ repo init -u sso://team/cros-build-tiger/cros-bazel-manifest -b main
$ repo sync -c -j 4
$ cd src
```

## Installing host dependencies

```sh
GOBIN=$HOME/go/bin go install github.com/bazelbuild/bazelisk@latest
```

You'll also need to get Bazelisk onto your PATH, to be executed before any Bazel
that's already on your PATH, and we'd like to invoke Bazelisk whenever we run
`bazel`. Create a symlink to bazelisk in a directory that'll be on your PATH
before any other bazels, and name the link `bazel`. Example:

```sh
ln -s ~/go/bin/bazelisk ~/bin/bazel
```

## Enabling commit hooks (optional)
If you'd like to run the tests every time you commit, add the following (you can skip it with `git commit --no-verify`).
```sh
cd cros-bazel/src/bazel
ln -s tools/run_tests.sh .git/hooks/pre-commit
```

## Building packages

To build sys-apps/attr:

```sh
$ BOARD=arm64-generic bazel build @portage//sys-apps/attr
```

To build all target packages:

```
$ BOARD=arm64-generic bazel build --keep_going //:all_target_packages
```

This is a short-cut to build `@portage//virtual/target-os:package_set`.

## Building images

We have the following targets to build images:

- `//:chromiumos_minimal_image`: Minimal image that contains
  `sys-apps/baselayout` and `sys-kernel/chromeos-kernel` only.
- `//:chromiumos_base_image`: Base image.
- `//:chromiumos_dev_image`: Dev image.
- `//:chromiumos_test_image`: Test image.

*** note
For historical reasons, the output file name of the dev image is chromiumos_image.bin, not chromiumos_dev_image.bin.
***

As of 2023-03-09, we support building images for amd64-generic only. We have
known build issues in some packages:
- `chromeos-base/chromeos-chrome`: Takes too long time (multiple hours) to
  build. Also randomly fails to build ([b/273830995](http://b/273830995)).
- `chromeos-base/chromeos-fonts`: Requires root to install binfmt_misc handlers
  ([b/262458823](http://b/262458823)).

You can inject prebuilt binary packages to bypass building those packages to
build a base image. You can pass `--config=prebuilts/amd64-generic` to do this
easily.

```
$ BOARD=amd64-generic bazel build --config=prebuilts/amd64-generic //:chromiumos_base_image
```

See [Injecting prebuilt binary packages](#injecting-prebuilt-binary-packages)
for more details.

After building an image, you can use `cros_vm` command available in CrOS SDK
to run a VM locally. Make sure to copy an image out from `bazel-bin` as it's not
writable by default.

```
~/cros-bazel$ cp src/bazel-bin/chromiumos_base_image.bin /tmp/
~/cros-bazel$ chmod +w /tmp/chromiumos_base_image.bin
~/cros-bazel$ chromite/bin/cros_vm --start --board=amd64-generic --image-path /tmp/chromiumos_base_image.bin
```

You can use VNC viewer to view the VM.
```
~/$ vncviewer localhost:5900
```

You can also use `cros_vm` command to stop the VM.
```
~/cros-bazel$ chromite/bin/cros_vm --stop
```

## Directory structure

See [manifest/_bazel.xml] for details on how this repository is organized.

[manifest/_bazel.xml]: https://team.git.corp.google.com/cros-build-tiger/cros-bazel-manifest/+/refs/heads/main/_bazel.xml

* `src/`
    * `WORKSPACE.bazel` ... Bazel workspace definitions; symlink to `bazel/workspace_root/WORKSPACE.bazel`
    * `BUILD.bazel` ... Bazel top-level target definitions; symlink to `bazel/workspace_root/BUILD.bazel`
    * `bazel/` ... contains Bazel-related files
        * `ebuild/`
            * `defs.bzl` ... provides Bazel rules
            * `cmd` ... commands for development
                * `extract_deps/` ... **DEPRECATED** extracts dependency graph from ebuilds
                * `generate_stats/` ... **DEPRECATED** generates package coverage stats
                * `update_build/` ... **DEPRECATED** generates BUILD files for ebuilds
            * `private/` ... contains programs used by Bazel rules
                * `alchemist` ... generates a Bazel repository on `bazel build`
                * `cmd/` commands internally used in the build
                    * `run_in_container/` ... runs a program within an unprivileged Linux container; used by other programs such as `build_sdk` and `build_package`
                    * `build_sdk/` ... builds SDK squashfs; used by `sdk` rule
                    * `build_package/` ... builds a Portage binary package; used by `ebuild` rule
                * `common/` ... common Rust/Go libraries
        * `config/` ... contains build configs like which overlays to use
        * `prebuilts/` ... defines prebuilt binaries
        * `sdk/` ... **DEPRECATED** defines SDK to use
        * `third_party/` ... contains build rules for third-party softwares needed
        * `workspace_root/` ... contains various files to be symlinked to the workspace root, including `WORKSPACE.bazel` and `BUILD.bazel`
* `manifest/` ... copy of cros-bazel-manifest repository

## Misc Memo

### Debugging a failing package

*** note
**TODO:** Fix the ability to get the build working directory. The method
described here is no longer working.
***

If a package is failing to build, it's sometimes useful to view the package's
work directory. To do this run:

```
bazel build --sandbox_debug //your/ebuild
```

In the build output you will see a `cd` into the `execroot`:

```
cd /home/rrangel/.cache/bazel/_bazel_rrangel/ca19c0757f7accdebe9bbcbd2cb0838e/sandbox/linux-sandbox/842/execroot/__main__
```

This directory will contain a directory called `build_package.*`. It contains
all the artifacts that were generated while building the package.

Build logs can be found in:

    scratch/diff/build/arm64-generic/tmp/portage/logs/

The package work dir can be found in:

    scratch/diff/build/<board>/tmp/portage/<category>/<package>-<version>

### Debugging an ephemeral CrOS chroot

Sometimes you want to enter an ephemeral CrOS chroot where a package build is
failing to inspect the environment interactively.

To enter an ephemeral CrOS chroot, run the following command:

```
$ BOARD=arm64-generic bazel run @portage//sys-apps/attr:debug -- --login=after
```

This command will give you an interactive shell after building a package.
You can also specify other values to `--login` to choose the timing to enter
an interactive console:

- `--login=before`: before building the package
- `--login=after`: after building the package
- `--login=after-fail`: after failing to build the package

### Injecting prebuilt binary packages

In the case your work is blocked by some package build failures, you can
workaround them by injecting prebuilt binary packages via command line flags.

For every `ebuild` target under `@portage//internal/packages/...`, an associated
string flag target is defined. You can set a `gs://` URL of a prebuilt binary
package to inject it.

For example, to inject a prebuilt binary packages for `chromeos-chrome`, you can
set this option:

```
--@portage//internal/packages/third_party/chromiumos-overlay/chromeos-base/chromeos-chrome:107.0.5257.0_rc-r1_prebuilt=gs://chromeos-prebuilt/board/amd64-generic/postsubmit-R107-15066.0.0-38990-8804973494937369745/packages/chromeos-base/chromeos-chrome-107.0.5257.0_rc-r1.tbz2
```

We have several named config groupings in [prebuilts.bazelrc] that define
typical options to inject prebuilts. You can specify `--config` to use them.

- `--config=prebuilts/amd64-generic`: Injects prebuilt binary packages needed to
  build amd64-generic images.

[prebuilts.bazelrc]: ./bazelrcs/prebuilts.bazelrc

### Extracting binary packages

In case you need to extract the contents of a binary package so you can easily
inspect it, you can use the `xpak split` CLI.

```sh
bazel run //bazel/ebuild/cmd/xpak:xpak -- split --extract libffi-3.1-r8.tbz2 libusb-0-r2.tbz2
```


## Building ToT

The cros-bazel-manifest was created from a snapshot from Aug 2022. We also
pinned the SDK at that time as well. If you are working on ToT (as of Mar 2023)
you can use the newer SDK by passing in the `--//bazel/sdk:new-sdk` flag.
