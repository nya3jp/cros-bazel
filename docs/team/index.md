# Bazelification Team Resources

[TOC]

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

## Testing your changes

The `run_tests.sh` script runs currently available tests:

```
$ portage/tools/run_tests.sh
```

Optionally, you can skip running some tests by specifying some of the following
environment variables when running `run_tests.sh`: `SKIP_CARGO_TESTS=1`,
`SKIP_BAZEL_TESTS=1`, `SKIP_PORTAGE_TESTS=1`.

If you'd like to run the tests every time you commit, add the following. You can
skip it with `git commit --no-verify`.

```sh
cd ~/chromiumos/src/bazel
ln -s ../../../../../src/bazel/portage/tools/run_tests.sh .git/hooks/pre-commit
```

## Running Bazel outside CrOS chroot

*** promo
**Running Bazel outside CrOS chroot is experimental.**
It works without setting up the CrOS SDK chroot.
`cros workon` states in the CrOS SDK chroot are ignored even if they exist, and
the live (9999) version of packages (if they exist and are not marked as
`CROS_WORKON_MANUAL_UPREV`) will be chosen by default. This means you can edit
your source code and feel confident that the correct packages are getting
rebuilt, though it might cause some build issues when live ebuilds are not
prepared for Bazel builds. Note that you'll get no remote cache hits from CI
builds today.

See [Getting Started](/docs/getting_started.md) for the currently recommended
way to run Bazel inside CrOS chroot. It is mostly compatible with the current
Portage-based build: Bazel honors Portage's site-specific configurations in
`/etc/portage` in the chroot, including `cros workon` states of packages.
Also, this is the only way you're going to get remote cache hits right now.
On the other hand, you still need to set up a CrOS SDK chroot.
***

Before you start building a package you need to ensure that `which bazel` prints
a path under your [depot_tools] checkout. The wrapper script provided by
`depot_tools` performs additional tasks besides running the real `bazel`
executable.

By default, the preflight check fails if you attempt to run Bazel outside CrOS
chroot. Set the environment variable `ALCHEMY_EXPERIMENTAL_OUTSIDE_CHROOT=1` to
bypass the check.

Then just follow [Getting Started] to run Bazel directly, except that you should
use Bazel from depot_tools and you can run it outside CrOS chroot.

[depot_tools]: https://commondatastorage.googleapis.com/chrome-infra-docs/flat/depot_tools/docs/html/depot_tools_tutorial.html#_setting_up
[Getting Started]: /docs/getting_started.md#building-chromeos-packages-directly-with-bazel

## Building images under Bazel

*** note
As of Oct 2023, we don't actively test building ChromeOS images under Bazel
due to priority reasons. You can try the following instruction, but it may fail.
***

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
