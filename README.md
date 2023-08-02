# ChromeOS Bazelification

This repository provides the implementation to build ChromeOS with Bazel.

## Checking out

Building ChromeOS with Bazel is currently possible only on a special branch for
Bazel development. Use the following `repo` command to check out the branch with
a few additional repositories.

```sh
$ mkdir ~/chromiumos
$ cd ~/chromiumos
$ repo init -u https://chrome-internal.googlesource.com/chromeos/manifest-internal -b stabilize-15429.B -g default,bazel
$ repo sync -c -j 4
$ cd src
```

Unless otherwise specified, examples in this doc assume that your current
directory is `~/chromiumos/src`.

## Building packages

Now you're ready to start building. To build a single Portage package, e.g.
sys-apps/attr:

```sh
$ BOARD=amd64-generic bazel build @portage//sys-apps/attr
```

To build all packages included in the ChromeOS base image:

```sh
$ BOARD=amd64-generic bazel build @portage//virtual/target-os:package_set
```

*** note
Please make sure that `which bazel` prints a path under your [depot_tools] checkout. The wrapper script provided by depot_tools performs additional tasks besides running the real `bazel` executable.
***

[depot_tools]: https://commondatastorage.googleapis.com/chrome-infra-docs/flat/depot_tools/docs/html/depot_tools_tutorial.html#_setting_up

### Inside CrOS SDK chroot

Inside CrOS SDK chroot (i.e. the build environment you enter with `cros_sdk` command), you should be able to run the same `bazel build` command.

You can also run `build_packages --bazel --board=$BOARD` to run `build_packages` with Bazel.

## Building images

We have the following targets to build images:

- `//bazel/images:chromiumos_minimal_image`: Minimal image that contains
  `sys-apps/baselayout` and `sys-kernel/chromeos-kernel` only.
- `//bazel/images:chromiumos_base_image`: Base image.
- `//bazel/images:chromiumos_dev_image`: Dev image.
- `//bazel/images:chromiumos_test_image`: Test image.

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
for more details.

After building an image, you can use `cros_vm` command available in CrOS SDK
to run a VM locally. Make sure to copy an image out from `bazel-bin` as it's not
writable by default.

```sh
$ cp bazel-bin/bazel/images/chromiumos_base_image.bin /tmp/
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

## Directory structure

* `portage/` ... for building Portage packages (aka Alchemy)
    * `bin/` ... executables
    * `common/` ... common Rust/Go libraries
    * `build_defs/` ... build rule definitions in Starlark
    * `repo_defs/` ... additional repository definitions
        * `prebuilts/` ... defines prebuilt binaries
    * `sdk/` ... defines the base SDK
    * `tools/` ... misc small tools for development
* `images/` ... defines ChromeOS image targets
* `workspace_root/` ... contains various files to be symlinked to the workspace root, including `WORKSPACE.bazel` and `BUILD.bazel`

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
--@portage//internal/packages/stage1/target/board/chromiumos/chromeos-base/chromeos-chrome:114.0.5715.0_rc-r2_prebuilt=gs://chromeos-prebuilt/board/amd64-generic/postsubmit-R114-15427.0.0-49533-8783437624917045025/packages/chromeos-base/chromeos-chrome-114.0.5715.0_rc-r2.tbz2
```

*** note
You can run [generate_chrome_prebuilt_config.py] to generate the prebuilt config
for the current version of chromeos-chrome.

```sh
% BOARD=amd64-generic portage/tools/generate_chrome_prebuilt_config.py
```

***

[generate_chrome_prebuilt_config.py]: ./portage/tools/generate_chrome_prebuilt_config.py

We have several named config groupings in [prebuilts.bazelrc] that define
typical options to inject prebuilts. You can specify `--config` to use them.

- `--config=prebuilts/arm64-generic`: Injects prebuilt binary packages needed to
  build arm64-generic images.

[prebuilts.bazelrc]: ./bazelrcs/prebuilts.bazelrc

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
ln -s tools/run_tests.sh .git/hooks/pre-commit
```
