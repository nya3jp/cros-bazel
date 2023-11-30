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

The syntax for specifying a Portage package is:

```
@portage//<host|target>/<category>/<package>`.
```

`host` means the build host ([CBUILD]), and `target` means the cross-compiled
target ([CHOST]) specified by the `BOARD` environment variable.

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

To build a package for the host, use the `host` prefix:

```sh
$ BOARD=amd64-generic bazel build @portage//host/app-shells/bash
```

To build all packages included in the ChromeOS test image:

```sh
$ BOARD=amd64-generic bazel build @portage//target/virtual/target-os:package_set @portage//target/virtual/target-os-dev:package_set @portage//target/virtual/target-os-test:package_set
```

[depot_tools]: https://commondatastorage.googleapis.com/chrome-infra-docs/flat/depot_tools/docs/html/depot_tools_tutorial.html#_setting_up
[CBUILD]: https://wiki.gentoo.org/wiki/Embedded_Handbook/General/Introduction#Toolchain_tuples
[CHOST]: https://wiki.gentoo.org/wiki/Embedded_Handbook/General/Introduction#Toolchain_tuples
[PDEPEND]: https://devmanual.gentoo.org/general-concepts/dependencies/#post-dependencies

## Building images

When you run Bazel inside the CrOS SDK chroot, you can simply use the standard
`cros build-image` command to build ChromeOS images.

The rest of the section describes the **very experimental** way to build
ChromeOS images under Bazel outside the CrOS SDK chroot.

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
