# ChromeOS Bazelification

This repository provides the implementation to build ChromeOS with Bazel.

## Checking out

For building ChromeOS with Bazel, use the following `repo` command to check out
with a few additional repositories.

```sh
$ mkdir ~/chromiumos
$ cd ~/chromiumos
$ repo init -u https://chrome-internal.googlesource.com/chromeos/manifest-internal -g default,bazel -b snapshot
$ repo sync -c -j 16
$ cd src
```

We use the `snapshot` branch rather than `main` because Bazel's caching logic
requires all inputs to match exactly, so you're better off working from the
`snapshot` branch that was already built by the Snapshot/CQ builders rather
than working from `main` and picking up whatever commit happened to be at ToT
at the time you ran `repo sync`. You'll be at most 40 minutes behind ToT, and
you'll have the best chance of getting cache hits to speed your builds. It's
safe to run the `repo init` command atop an existing checkout, which will
switch you to the `snapshot` branch.

Unless otherwise specified, examples in this doc assume that your current
directory is `~/chromiumos/src`.

## Building packages

Before you start building a package you need to ensure that `which bazel` prints a path under
your [depot_tools] checkout. The wrapper script provided by `depot_tools` performs additional
tasks besides running the real `bazel` executable.

The syntax for specifying a `portage` package is:

```
@portage//<host|target>/<category>/<package>`.
```

`host` means the build host ([CBUILD]), and `target` means the cross-compiled target ([CHOST])
specified by the `BOARD` environment variable.

Now you're ready to start building. To build a single Portage package, e.g.
`sys-apps/attr`:

```sh
$ BOARD=amd64-generic bazel build @portage//target/sys-apps/attr
```

To build all packages included in the ChromeOS base image:

```sh
$ BOARD=amd64-generic bazel build @portage//target/virtual/target-os:package_set
```

A `package_set` is a special target that also includes the target's [PDEPEND]s.

To build a package for the `host` , use the `host` prefix:

```sh
$ BOARD=amd64-generic bazel build @portage//host/app-shells/bash
```

To build all packages included in the ChromeOS test image:

```sh
$ BOARD=amd64-generic bazel build @portage//target/virtual/target-os:package_set @portage//target/virtual/target-os-dev:package_set @portage//target/virtual/target-os-test:package_set
```

When building packages outside the chroot, the `9999` version of packages (if they exist and are
not marked as `CROS_WORKON_MANUAL_UPREV`) will be chosen by default. This means you can edit
your source code and feel confident that the correct packages are getting rebuilt.

[depot_tools]: https://commondatastorage.googleapis.com/chrome-infra-docs/flat/depot_tools/docs/html/depot_tools_tutorial.html#_setting_up
[PDEPEND]: https://devmanual.gentoo.org/general-concepts/dependencies/#post-dependencies
[CBUILD]: https://wiki.gentoo.org/wiki/Embedded_Handbook/General/Introduction#Toolchain_tuples
[CHOST]: https://wiki.gentoo.org/wiki/Embedded_Handbook/General/Introduction#Toolchain_tuples

### Inside CrOS SDK chroot

Inside CrOS SDK chroot (i.e. the build environment you enter with `cros_sdk` command), you should be able to run the same command except that you need to use `/mnt/host/source/chromite/bin/bazel` instead of `bazel`.

Before you do anything, ensure you have created the `amd64-host` `sysroot`.

```sh
(cr) $ /mnt/host/source/src/scripts/create_sdk_board_root --board amd64-host --profile sdk/bootstrap
```

This will create `/build/amd64-host`. This `sysroot` contains the portage configuration that is
used when building `host` tool packages. i.e., [CBUILD].

You can then proceed to create the board's `sysroot`:

```sh
(cr) $ setup_board --board amd64-generic --skip-chroot-upgrade --skip-toolchain-update
```

Now that you have configured your chroot, you can invoke a build:

```sh
(cr) $ BOARD=amd64-generic /mnt/host/source/chromite/bin/bazel build @portage//target/sys-apps/attr
```

You can also run `build_packages --bazel --board=$BOARD` to run `build_packages` with Bazel.

`cros-workon-$BOARD start <pkg>` is required to work on a `9999` package when working inside the
chroot.

## Building images

We have the following targets to build images:

- `@portage//images:chromiumos_minimal_image`: Minimal image that contains
  `sys-apps/baselayout` and `sys-kernel/chromeos-kernel` only.
- `@portage//images:chromiumos_base_image`: Base image.
- `@portage//images:chromiumos_dev_image`: Dev image.
- `@portage//images:chromiumos_test_image`: Test image.

*** note
For historical reasons, the output file name of the dev image is
chromiumos_image.bin, not chromiumos_dev_image.bin.
***

As of June 2023, we primarily test our builds for amd64-generic and
arm64-generic. Please file bugs if images don't build for these two boards.
Other boards may or may not work (yet).

Building a ChromeOS image takes several hours. Most packages build in a few
minutes, but there are several known heavy packages, such as
`chromeos-base/chromeos-chrome` that takes 2-3 hours. You can inject prebuilt
binary packages to bypass building those packages.
See [Injecting prebuilt binary packages](#injecting-prebuilt-binary-packages)
for more details. To make `chromeos-base/chromeos-chrome` build faster, you can
also use [Goma](#using-goma-to-build-chrome).

After building an image, you can use `cros_vm` command available in CrOS SDK
to run a VM locally. Make sure to copy an image out from `bazel-bin` as it's not
writable by default.

```sh
$ cp bazel-bin/external/_main~portage~portage/images/chromiumos_base_image.bin /tmp/
$ chmod +w /tmp/chromiumos_base_image.bin
$ chromite/bin/cros_vm --start --board=amd64-generic --image-path /tmp/chromiumos_base_image.bin
```

You can use VNC viewer to view the VM.
```sh
$ vncviewer localhost:5900
```

You can also use `cros_vm` command to stop the VM.
```sh
$ chromite/bin/cros_vm --stop
```

## Enabling @portage tab completion

By default you can't tab complete the `@portage//` repository. This is because
bazel doesn't provide support for tab completing external repositories. By
setting `export ENABLE_PORTAGE_TAB_COMPLETION=1` in your `.bashrc`/`.profile`,
`bazel` will create a `@portage` symlink in the workspace root (i.e.,
`~/chromiumos/src`). This allows the bazel tab completion to work, but comes
with one caveat. You can no longer run `bazel build //...` because it will
generate analysis errors. This is why this flag is not enabled by default.

The `@portage` symlink has another added benefit, you can easily browse the
generated `BUILD.bazel` files.

## Testing your change

The `run_tests.sh` script runs currently available tests:

```
$ portage/tools/run_tests.sh
```

Optionally, you can skip running some tests by specifying some of the following
environment variables when running `run_tests.sh`: `SKIP_CARGO_TESTS=1`,
`SKIP_BAZEL_TESTS=1`, `SKIP_PORTAGE_TESTS=1`.

## Troubleshooting

### Bad cache results when non-hermetic inputs change
Bazel is able to correctly reuse content from the cache when all inputs are
identified to it so it can detect when they change. Since our toolchain and our
host tools (e.g. gsutil) are not yet fully hermetic, it's possible that you'll
run into problems when tools not yet tracked by Bazel are updated. In these
situations we've found it useful to run `bazel clean --expunge` to clear cached
artifacts that seem not to be cleared without the `--expunge` flag.

If you find you need the `--expunge` flag, please file a bug to let the
Bazelification team know about the non-hermeticity so we can fix the problem.

## Directory structure

* `portage/` ... for building Portage packages (aka Alchemy)
    * `bin/` ... executables
    * `common/` ... common Rust/Go libraries
    * `build_defs/` ... build rule definitions in Starlark
    * `repo_defs/` ... additional repository definitions
        * `prebuilts/` ... defines prebuilt binaries
    * `sdk/` ... defines the base SDK
    * `tools/` ... misc small tools for development
* `workspace_root/` ... contains various files to be symlinked to the workspace root, including `WORKSPACE.bazel` and `BUILD.bazel`

## Misc Memo

### Bazel remote caching with RBE

You can speed up the build by enabling remote Bazel caching with RBE.
To do this, follow [this instruction](https://chromium.googlesource.com/chromiumos/docs/+/HEAD/developer_guide.md#authenticate-for-remote-bazel-caching-with-rbe_if-applicable)
to authenticate.

After authentication, make sure that you restart the Bazel instance by running
`bazel shutdown`.

### Debugging a failing package

Sometimes you want to enter an ephemeral CrOS chroot where a package build is
failing to inspect the environment interactively.

To enter an ephemeral CrOS chroot, run the following command:

```
$ BOARD=arm64-generic bazel run @portage//target/sys-apps/attr:debug -- --login=after
```

This command will give you an interactive shell after building a package.
You can also specify other values to `--login` to choose the timing to enter
an interactive console:

- `--login=before`: before building the package
- `--login=after`: after building the package (default)
- `--login=after-fail`: after failing to build the package

### Using Goma to build Chrome

Building `chromeos-base/chromeos-chrome` takes 2-3 hours, but you can use Goma
to make it as fast as less than 1 hour.

To use Goma, please follow [Goma for Chromium contributors] (or
[go/chrome-linux-build](http://go/chrome-linux-build) if you're a Googler) and
run `goma_auth login` for authentication. Please make sure that you perform
authentication inside the chroot if you're going to run `bazel build` inside
the chroot, and do that outside the chroot if you're going to run it outside the
chroot.

[Goma for Chromium contributors]: https://chromium.googlesource.com/infra/goma/client/+/HEAD/doc/early-access-guide.md

After authentication, you can just run `bazel build` with `USE_GOMA=true` to
enable Goma.

```
$ USE_GOMA=true BOARD=amd64-generic bazel build @portage//chromeos-base/chromeos-chrome
```

You can also run `build_packages` with `--run-goma` to run it with Goma.

```
$ build_packages --bazel --board=amd64-generic --run-goma
```

### Injecting prebuilt binary packages

In the case your work is blocked by some package build failures, you can
workaround them by injecting prebuilt binary packages via command line flags.

For every `ebuild` target under `@portage//internal/packages/...`, an associated
string flag target is defined. You can set a `gs://` URL of a prebuilt binary
package to inject it.

For example, to inject a prebuilt binary packages for `chromeos-chrome`, you can
set this option:

```
--@portage//internal/packages/stage1/target/board/chromiumos/chromeos-base/chromeos-chrome:114.0.5715.0_rc-r2_prebuilt=gs://chromeos-prebuilt/board/amd64-generic/postsubmit-R114-15427.0.0-49533-8783437624917045025/packages/chromeos-base/chromeos-chrome-114.0.5715.0_rc-r2.tbz2
```

*** note
You can run [generate_chrome_prebuilt_config.py] to generate the prebuilt config
for the current version of chromeos-chrome.

```sh
% BOARD=amd64-generic portage/tools/generate_chrome_prebuilt_config.py
```
***

When performing changes to `eclasses`, `build_packages`, `chromite` or other
things that cache bust large parts of the graph, it might be beneficial to pin
the binary packages for already built packages so you don't need to rebuild
them when iterating on your changes. You can use the [generate-stage2-prebuilts]
script to do this:

```sh
$ BOARD=amd64-generic ./bazel/portage/tools/generate-stage2-prebuilts
```

This will scan your `bazel-bin` directory for any existing binpkgs and copy them
to `~/.cache/binpkgs`. It will then generate a `prebuilts.bazelrc` that contains
various `--config` options. The `prebuilts.bazelrc` is invalid after you
`repo sync` since it contains package version numbers. Just re-run the script
after a `repo sync` to regenerate the `prebuilts.bazelrc` and it will pin the
packages with versions that still exist in your `bazel-bin`.

Running a build with pinned packages:

```sh
$ BOARD=amd64-generic bazel build --config=prebuilts/stage2-board-sdk @portage//target/sys-apps/attr
```

[generate_chrome_prebuilt_config.py]: ./portage/tools/generate_chrome_prebuilt_config.py
[generate-stage2-prebuilts]: ./portage/tools/generate-stage2-prebuilts

### Extracting binary packages

In case you need to extract the contents of a binary package so you can easily
inspect it, you can use the `xpak split` CLI.

```sh
bazel run //bazel/portage/bin/xpak:xpak -- split --extract libffi-3.1-r8.tbz2 libusb-0-r2.tbz2
```

### Running tests on every local commit

If you'd like to run the tests every time you commit, add the following. You can
skip it with `git commit --no-verify`.

```sh
cd ~/chromiumos/src/bazel
ln -s ../../../../../src/bazel/portage/tools/run_tests.sh .git/hooks/pre-commit
```
