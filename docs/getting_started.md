# Getting Started

[TOC]

## Checking out the repository

For building ChromeOS with Bazel, use the following `repo` command to check out
with a few additional repositories.

```sh
$ mkdir ~/chromiumos
$ cd ~/chromiumos
# If you have access to the internal manifests:
$ repo init -u https://chrome-internal.googlesource.com/chromeos/manifest-internal -g default,bazel -b snapshot
# Otherwise:
$ repo init -u https://chromium.googlesource.com/chromiumos/manifest -g default,bazel -b snapshot
$ repo sync -c -j 16
$ cd src
```

- `-g default,bazel` is important to check out some additional repositories
  needed to build with Bazel.
- We use the `snapshot` branch rather than `main` because Bazel's caching logic
  requires all inputs to match exactly, so you're better off working from the
  `snapshot` branch that was already built by the Snapshot/CQ builders rather
  than working from `main` and picking up whatever commit happened to be at ToT
  at the time you ran `repo sync`. You'll be at most 40 minutes behind ToT, and
  you'll have the best chance of getting cache hits to speed your builds.
- It's safe to run the `repo init` command atop an existing checkout.

Unless otherwise specified, examples in this doc assume that your current
directory is `~/chromiumos/src`.

## One-time setup

First, enter the CrOS SDK chroot with `cros_sdk` command, which will create a
new one if you haven't already. Then run the following command to create the
`amd64-host` sysroot.

```sh
(cr) $ /mnt/host/source/chromite/shell/create_sdk_board_root --board amd64-host --profile sdk/bootstrap
```

This will create `/build/amd64-host`. This sysroot contains the Portage
configuration that is used when building host tool packages. i.e., [CBUILD].

You can then proceed to create the target board's sysroot:

```sh
(cr) $ setup_board --board amd64-generic --public
```

If you're attempting to build a public image while using an internal manifest,
the `--public` flag allows you to do that, which is useful when attempting to
recreate a CQ/Snapshot failure for build targets that use public manifests on
the CI bots, such as amd64-generic. Omit the flag if you do want to build
internal packages.

[CBUILD]: https://wiki.gentoo.org/wiki/Embedded_Handbook/General/Introduction#Toolchain_tuples

## Building ChromeOS packages with `cros build-packages`

Now that you have configured your chroot, you can invoke a build with the
standard `cros build-packages` command, except that you need to pass the extra
option `--bazel` to build with Bazel.

To build a single Portage package, e.g. `sys-apps/attr`:

```sh
$ cros build-packages --board=amd64-generic --bazel sys-apps/attr
```

To build all packages included in the ChromeOS test image:

```sh
$ cros build-packages --board=amd64-generic --bazel
```

Upon successful completion, packages are installed to the sysroot inside the
CrOS SDK chroot, so you can use other commands expecting built packages to be in
the sysroot, e.g. `cros build-image` and `cros deploy`.

## Building ChromeOS packages directly with Bazel

You can alternatively run Bazel directly to build certain targets.

*** promo
Currently you need to use `/mnt/host/source/chromite/bin/bazel` instead of
`/usr/bin/bazel` that is found first on the standard `$PATH`.
***

The syntax for specifying a Portage package is:

```
@portage//<host|target>/<category>/<package>`.
```

`host` means the build host ([CBUILD]), and `target` means the cross-compiled
target ([CHOST]) specified by the `BOARD` environment variable.

Now you're ready to start building. To build a single Portage package, e.g.
`sys-apps/attr`:

```sh
$ BOARD=amd64-generic /mnt/host/source/chromite/bin/bazel build @portage//target/sys-apps/attr
```

To build all packages included in the ChromeOS base image:

```sh
$ BOARD=amd64-generic /mnt/host/source/chromite/bin/bazel build @portage//target/virtual/target-os:package_set
```

A `package_set` is a special target that also includes the target's [PDEPEND]s.

To build a package for the host, use the `host` prefix:

```sh
$ BOARD=amd64-generic /mnt/host/source/chromite/bin/bazel build @portage//host/app-shells/bash
```

To build all packages included in the ChromeOS test image:

```sh
$ BOARD=amd64-generic /mnt/host/source/chromite/bin/bazel build @portage//target/virtual/target-os:package_set @portage//target/virtual/target-os-dev:package_set @portage//target/virtual/target-os-test:package_set
```

*** note
When you build packages directly with `bazel build`, packages are not installed
to the sysroot, which means that you can't use built packages with existing
tools like `cros deploy`. Use `cros build-packages --bazel` instead if you want
to install packages to the sysroot after they're built by Bazel.
***

[CBUILD]: https://wiki.gentoo.org/wiki/Embedded_Handbook/General/Introduction#Toolchain_tuples
[CHOST]: https://wiki.gentoo.org/wiki/Embedded_Handbook/General/Introduction#Toolchain_tuples
[PDEPEND]: https://devmanual.gentoo.org/general-concepts/dependencies/#post-dependencies

## Building ChromeOS images

You can simply run `cros build-image` to build ChromeOS images if you use
`cros build-packages --bazel` to build ChromeOS packages included in ChromeOS
images.

## Tips

### Bazel remote caching with RBE

You can speed up the build by enabling remote Bazel caching with RBE.
To do this, follow [this instruction](https://chromium.googlesource.com/chromiumos/docs/+/HEAD/developer_guide.md#authenticate-for-remote-bazel-caching-with-rbe_if-applicable)
to authenticate.

After authentication, make sure that you restart the Bazel instance by running
`bazel shutdown`.

### Using reclient for faster build

Building `chromeos-base/chromeos-chrome` takes 2-3 hours, but you can use reclient
to make it as fast as less than 1 hour.

To use reclient, please run `gcloud auth application-default login` for
authentication. If you're going to run `bazel build` inside the chroot, please
make sure that you run this outside the chroot after exiting the chroot in all
existing windows.

After authentication, you can just run `bazel build` with `USE_REMOTEEXEC=true`
to enable reclient.

```
$ USE_REMOTEEXEC=true BOARD=amd64-generic bazel build @portage//chromeos-base/chromeos-chrome
```

You can also run `build_packages` with `--run_remoteexec` to run it with reclient.

```
$ build_packages --bazel --board=amd64-generic --run_remoteexec
```

### Enabling @portage tab completion

By default you can't tab complete the `@portage//` repository. This is because
bazel doesn't provide support for tab completing external repositories. By
setting `export ENABLE_PORTAGE_TAB_COMPLETION=1` in your `.bashrc`/`.profile`,
`bazel` will create a `@portage` symlink in the workspace root (i.e.,
`~/chromiumos/src`). This allows the bazel tab completion to work, but comes
with one caveat. You can no longer run `bazel build //...` because it will
generate analysis errors. This is why this flag is not enabled by default.

The `@portage` symlink has another added benefit, you can easily browse the
generated `BUILD.bazel` files.
