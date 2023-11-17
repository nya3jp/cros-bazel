# ChromeOS Bazelification

This repository provides the implementation to build ChromeOS with Bazel.

## Checking out

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

## Building packages

We support two configurations to build ChromeOS with Bazel:

1. Run Bazel inside the CrOS SDK chroot (**recommended**).
2. Run Bazel outside the CrOS SDK chroot.

### Run Bazel inside the CrOS SDK chroot

*** promo
This is the current recommended way. It is mostly compatible with the current
Portage-based build: Bazel honors Portage's site-specific configurations in
`/etc/portage` in the chroot, including `cros workon` states of packages.
Also, this is the only way you're going to get remote cache hits right now.
On the other hand, you still need to set up a CrOS SDK chroot.
***

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

*** aside
Alternatively, you can run Bazel directly inside the CrOS SDK chroot to build
specific targets, except that you need to use
`/mnt/host/source/chromite/bin/bazel` instead of `/usr/bin/bazel`. Read the
following section to see how to specify build targets.
***

### Run Bazel outside the CrOS SDK chroot

*** promo
This is an experimental way. It works without setting up the CrOS SDK chroot.
`cros workon` states in the CrOS SDK chroot are ignored even if they exist, and
the live (9999) version of packages (if they exist and are not marked as
`CROS_WORKON_MANUAL_UPREV`) will be chosen by default. This means you can edit
your source code and feel confident that the correct packages are getting
rebuilt, though it might cause some build issues when live ebuilds are not
prepared for Bazel builds. Note that you'll get no remote cache hits from CI
builds today.
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

## Advanced information

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

### Testing your change

The `run_tests.sh` script runs currently available tests:

```
$ portage/tools/run_tests.sh
```

Optionally, you can skip running some tests by specifying some of the following
environment variables when running `run_tests.sh`: `SKIP_CARGO_TESTS=1`,
`SKIP_BAZEL_TESTS=1`, `SKIP_PORTAGE_TESTS=1`.

### What do the different stages mean in the target paths?

You might have noticed paths like
`@portage//internal/packages/stage2/host/portage-stable/sys-apps/attr:2.5.1`.

TL;DR, stageN in the package path means the package was built using the stageN
SDK.

The bazel host tools build architecture was inspired by the
[Gentoo Portage bootstrapping processes]. That is, we start with a "bootstrap
tarball/SDK", or what we call the stage1 tarball/SDK. This stage1 SDK is
expected to have all the host tools (i.e., portage, clang, make, etc)
required to build the [virtual/target-sdk-implicit-system] package and its
run-time dependencies. We don't concern ourselves with the specific versions
and build-time configuration of the packages contained in the stage1 SDK. We
only need the tools to be recent enough to perform a "cross-root" compile of the
[virtual/target-sdk-implicit-system] package and its dependencies.

*** note
A cross-root compilation is defined as `CBUILD=CHOST && BROOT=/ &&
ROOT=/build/host` (e.g., `CHOST=x86_64-pc-linux-gnu`). In other words, we use
the native compiler (instead of a cross-compiler) to build a brand new sysroot
in a different directory.
***

A cross-root build allows us to bootstrap the system from scratch. That means we
don’t build or link against any of the headers and libraries installed in the
`BROOT`. By starting from scratch, we can choose which packages to build,
their versions, and their build-time configuration. We call these packages built
using the stage1 SDK the "stage1 packages".

*** note
Since the stage1 SDK is the root node of all packages, we want to avoid updating
it unnecessarily to avoid cache busting all the packages.
***

Once all the "stage1 packages" (or "implicit system packages") have been built,
we take that newly created sysroot (i.e., `/build/host`) and generate the
"stage2 tarball/SDK". This bootstrap flow has two big advantages:

1.  By cross-root compiling instead of trying to update the stage1 SDK in place,
    we avoid performing any analysis on the packages it contains, and computing
    the install, uninstall, and rebuild actions required to "upgrade" the stage1
    SDK in place. This reduces a great deal of complexity.
2.  Changes to any of the implicit system packages are immediately taken into
    account. There is no separate out-of-band processes required. This means we
    can catch build breakages before a CL lands. For example, if we wanted to
    upgrade `bash` or `portage`, we could just rev-bump the ebuild, and
    everything would get rebuilt automatically using the new version.

*** note
This flow essentially replaces the `update_chroot` step used by the portage
flow.
***

Now that we have a newly minted stage2 SDK, we can start building the "stage2
host packages". We no longer need to cross-root build these host tools, but can
instead perform a regular native-root build (i.e., `ROOT=/`) since we now know
the versions of the headers and libraries that are installed. When performing a
native-root build, there is basically no difference between a `DEPEND` and
`BDEPEND` dependency.

The next step is building the cross compilers for the target board. Under
portage, this is normally done using the [crossdev] tool. With bazel we just
build the cross compilers like any other package. The ebuilds have been modified
to properly declare their dependencies, so everything just works.

Once the cross compilers are built, we can start building the target board
packages. We first start with the [primordial packages] (i.e., `glibc`,
`libcxx`, etc). You can think of these as implicit system dependencies for the
target board. From there we can continue building the specified target package.
Since we are cross-compiling the target board's packages, we depend on the
packages to declare proper `BDEPEND`s so we can inject the proper host tools.

*** note
If you encounter an ebuild with `EAPI < 7` (which doesn't support
`BDEPEND`), please upgrade it and declare the proper `BDEPEND`s. For these older
ebuilds, we need to treat the `DEPEND` line as a `BDEPEND` line. This results in
building extra host tools that might not be necessary. To limit the impact of
these extra dependencies, we maintain a [list] of `DEPEND`s that we consider
valid `BDEPEND`s.
***

Conceptually you can think of every package getting a custom built SDK that
contains only the things it specifies as dependencies. We create these
per-package ephemeral SDKs as an overlayfs filesystem for efficiency, layering
all declared `BDEPEND`s, `DEPEND`s, and `RDEPEND`s atop an implicit system
layer, and executing the package's ebuild actions within that ephemeral SDK.

In summary, this is what the structure looks like:

*   //bazel/portage/sdk:stage1 <-- The downloaded bootstrap/stage1 SDK.
*   @portage//internal/ <- Alchemist implementation details.
    *   sdk/ <-- Directory containing all the SDK targets.
        *   stage1/target/host <-- `stage1` SDK with the `/build/host` sysroot
            containing the [primordial packages].
        *   stage2 <-- The stage2 SDK/tarball containing the freshly built and
            up-to-date `stage1/target/host` [virtual/target-sdk-implicit-system]
            packages.
            *   target/board <-- `stage2` SDK with the `/build/board` sysroot
                containing the [primordial packages].
        *   stage3:bootstrap <-- Will be added in the future. It will contain
            the `stage2/host` [virtual/target-sdk-implicit-system] packages, and
            all their transitive `BDEPEND`s. This tarball can then be used as a
            stage1 tarball whenever we need a new one.
    *   packages/ -- Directory containing all the package targets.
        *   stage1/target/host/`${OVERLAY}`/`${CATEGORY}`/`${PACKAGE}` <-- The
            cross-root compiled host packages built using the
            `stage1/target/host` SDK. These are what go into making the `stage2`
            SDK.
        *   stage2/ <-- Directory containing all the stage2 packages.
            *   host/`${OVERLAY}`/`${CATEGORY}`/`${PACKAGE}` <-- The native-root
                compiled host packages built using the `stage2` SDK. These will
                also be used to generate the `stage3:bootstrap` SDK in the
                future.
            *   target/board/`${OVERLAY}`/`${CATEGORY}`/`${PACKAGE}` <-- The
                cross-compiled target board packages built using the
                `stage2/target/board` SDK.
        *   stage3/target/host/`${OVERLAY}`/`${CATEGORY}`/`${PACKAGE}` <-- The
            cross-root compiled host packages built using the `stage3:bootstrap`
            SDK. This will be used to confirm that the `stage3:bootstrap` SDK
            can be successfully used as a "stage1 SDK/tarball".

As you can see, it's turtles (SDKs?) all the way down.

[Gentoo Portage bootstrapping processes]: https://web.archive.org/web/20171119211822/https://wiki.gentoo.org/wiki/Sakaki%27s_EFI_Install_Guide/Building_the_Gentoo_Base_System_Minus_Kernel#Bootstrapping_the_Base_System_.28Optional_but_Recommended.29
[virtual/target-sdk-implicit-system]: https://chromium.googlesource.com/chromiumos/overlays/chromiumos-overlay/+/refs/heads/main/virtual/target-sdk-implicit-system/target-sdk-implicit-system-9999.ebuild
[crossdev]: https://wiki.gentoo.org/wiki/Cross_build_environment
[primordial packages]: https://chromium.googlesource.com/chromiumos/bazel/+/refs/heads/main/portage/bin/alchemist/src/bin/alchemist/generate_repo/common.rs#21
[list]: https://chromium.googlesource.com/chromiumos/bazel/+/refs/heads/main/portage/bin/alchemist/src/analyze/dependency.rs#550

### Troubleshooting

#### Bad cache results when non-hermetic inputs change
Bazel is able to correctly reuse content from the cache when all inputs are
identified to it so it can detect when they change. Since our toolchain and our
host tools (e.g. gsutil) are not yet fully hermetic, it's possible that you'll
run into problems when tools not yet tracked by Bazel are updated. In these
situations we've found it useful to run `bazel clean --expunge` to clear cached
artifacts that seem not to be cleared without the `--expunge` flag.

If you find you need the `--expunge` flag, please file a bug to let the
Bazelification team know about the non-hermeticity so we can fix the problem.

### Directory structure

* `portage/` ... for building Portage packages (aka Alchemy)
    * `bin/` ... executables
    * `common/` ... common Rust/Go libraries
    * `build_defs/` ... build rule definitions in Starlark
    * `repo_defs/` ... additional repository definitions
        * `prebuilts/` ... defines prebuilt binaries
    * `sdk/` ... defines the base SDK
    * `tools/` ... misc small tools for development
* `workspace_root/` ... contains various files to be symlinked to the workspace root, including `WORKSPACE.bazel` and `BUILD.bazel`

### Misc Memo

#### Bazel remote caching with RBE

You can speed up the build by enabling remote Bazel caching with RBE.
To do this, follow [this instruction](https://chromium.googlesource.com/chromiumos/docs/+/HEAD/developer_guide.md#authenticate-for-remote-bazel-caching-with-rbe_if-applicable)
to authenticate.

After authentication, make sure that you restart the Bazel instance by running
`bazel shutdown`.

#### Debugging a failing package

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

#### Using Goma to build Chrome

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

#### Injecting prebuilt binary packages

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

#### Extracting binary packages

In case you need to extract the contents of a binary package so you can easily
inspect it, you can use the `xpak split` CLI.

```sh
bazel run //bazel/portage/bin/xpak:xpak -- split --extract libffi-3.1-r8.tbz2 libusb-0-r2.tbz2
```

#### Running tests on every local commit

If you'd like to run the tests every time you commit, add the following. You can
skip it with `git commit --no-verify`.

```sh
cd ~/chromiumos/src/bazel
ln -s ../../../../../src/bazel/portage/tools/run_tests.sh .git/hooks/pre-commit
```

#### Bazel Build Event Services

Bazel supports uploading and persisting build/test events and top level outputs
(e.g. what was built, invocation command, hostname, performance metrics) to a
backend. These build events can then be visualized and accessed over a shareable
URL. These standardized backends are known as Build Event Services (BES), and the
events are defined in
[build_event_stream.proto](https://github.com/bazelbuild/bazel/blob/master/src/main/java/com/google/devtools/build/lib/buildeventstream/proto/build_event_stream.proto).

Currently, BES uploads are enabled by default for all CI and local builds (for
Googlers). The URL is printed at the start and end of every invocation. For
example:

``` sh
$ BOARD=amd64-generic bazel test //bazel/rust/examples/...
(09:18:58) INFO: Invocation ID: 2dbec8dc-8dfe-4263-b0db-399a029b7dc7
(09:18:58) INFO: Streaming build results to: http://sponge2/2dbec8dc-8dfe-4263-b0db-399a029b7dc7
...
(09:19:13) INFO: Elapsed time: 16.542s, Critical Path: 2.96s
(09:19:13) INFO: 6 processes: 3 remote cache hit, 7 linux-sandbox.
Executed 5 out of 5 tests: 5 tests pass.
(09:19:13) INFO: Streaming build results to: http://sponge2/2dbec8dc-8dfe-4263-b0db-399a029b7dc7
```

The flags related to BES uploads are grouped behind the `--config=bes` flag,
defined in the common bazelrc file.
