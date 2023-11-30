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
(cr) $ /mnt/host/source/src/scripts/create_sdk_board_root --board amd64-host --profile sdk/bootstrap
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

## Building ChromeOS packages

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
CrOS SDK chroot.

## Tips

### Bazel remote caching with RBE

You can speed up the build by enabling remote Bazel caching with RBE.
To do this, follow [this instruction](https://chromium.googlesource.com/chromiumos/docs/+/HEAD/developer_guide.md#authenticate-for-remote-bazel-caching-with-rbe_if-applicable)
to authenticate.

After authentication, make sure that you restart the Bazel instance by running
`bazel shutdown`.

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
