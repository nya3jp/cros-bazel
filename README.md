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

## Building

To build sys-apps/attr:

```sh
$ BOARD=arm64-generic bazel build @portage//sys-apps/attr
```

To build all target packages:

```
$ BOARD=arm64-generic bazel build --keep_going //:all_target_packages
```

This is a short-cut to build `@portage//virtual/target-os:package_set`.

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
                    * `ver_test/` ... **DEPRECATED** implements `ver_test` function in ebuilds
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

### Extracting binary packages

In case you need to extract the contents of a binary package so you can easily
inspect it, you can use the `xpak split` CLI.

```sh
bazel run //bazel/ebuild/cmd/xpak:xpak -- split --extract libffi-3.1-r8.tbz2 libusb-0-r2.tbz2
```
