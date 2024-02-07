# Advanced Topics

[TOC]

## What do the different stages mean in the target paths?

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
        *   stage3:bootstrap <-- It is built using the `stage2/host`
            [virtual/target-sdk-implicit-system] packages, and all their
            transitive `BDEPEND`s. This tarball can then be used as a
            stage1 tarball whenever we need a new one.
        *   stage4 <-- This is only used to verify that the `stage3:bootstrap`
            SDK can build the implicit system. It is built using the packages
            from `stage3/target/host`.
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

## Bazel Build Event Services

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

## Injecting prebuilt binary packages

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

[generate_chrome_prebuilt_config.py]: ../portage/tools/generate_chrome_prebuilt_config.py
[generate-stage2-prebuilts]: ../portage/tools/generate-stage2-prebuilts

## Extracting binary packages

In case you need to extract the contents of a binary package so you can easily
inspect it, you can use the `xpak split` CLI.

```sh
bazel run //bazel/portage/bin/xpak:xpak -- split --extract libffi-3.1-r8.tbz2 libusb-0-r2.tbz2
```
