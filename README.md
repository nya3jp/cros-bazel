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
sudo apt install squashfs-tools

go install github.com/bazelbuild/bazelisk@latest
```

You'll also need to get Bazelisk onto your PATH, to be executed before any Bazel
that's already on your PATH, and we'd like to invoke Bazelisk whenever we run
`bazel`. Create a symlink to bazelisk in a directory that'll be on your PATH
before any other bazels, and name the link `bazel`. Example:
```sh
ln -s ${GOPATH:-$HOME/go}/bin/bazelisk ~/bin/bazel
```

## Building

First you need to generate `BUILD.bazel` files for Portage packages.
Package data needed to generate them are managed in `bazel/data/deps.json`
and it can be converted to `BUILD.bazel` with the following command:

```sh
$ bazel run //bazel/ebuild/cmd/update_build
```

Then you can start building packages. To build sys-apps/attr for example:

```sh
$ bazel build //third_party/portage-stable/sys-apps/attr:0
```

Note that the label "0" is a SLOT identifier. It is typically "0", but it can
have different values for packages where multiple versions can be installed
at the same time.

To build all target packages:

```
$ bazel build --keep_going //:all_target_packages
```

This is basically a short-cut to build
`//third_party/chromiumos-overlay/virtual/target-os:0`.

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
                * `extract_deps/` ... extracts dependency graph from ebuilds
                * `generate_stats/` ... generates package coverage stats
                * `update_build/` ... generates BUILD files for ebuilds
            * `private/` ... contains programs used by Bazel rules
                * `cmd/`
                    * `create_squashfs/` ... creates squashfs from a file set; used by `overlay` rule
                    * `run_in_container/` ... runs a program within an unprivileged Linux container; used by other programs such as `build_sdk` and `build_package`
                    * `build_sdk/` ... builds SDK squashfs; used by `sdk` rule
                    * `build_package/` ... builds a Portage binary package; used by `ebuild` rule
                    * `ver_test/` ... implements `ver_test` function in ebuilds
        * `config/` ... contains build configs like which overlays to use
        * `prebuilts/` ... defines prebuilt binaries
        * `sdk/` ... defines SDK to use
        * `third_party/` ... contains build rules for third-party softwares needed
        * `workspace_root/` ... contains `WORKSPACE.bazel` and `BUILD.bazel` to be symlinked to the workspace root
    * `overlays/`
        * `overlay-arm64-generic/` ... a fork of overlay
    * `third_party/`
        * `portage-stable/` ... a fork of overlay
        * `eclass-overlay/` ... a fork of overlay
        * `chromiumos-overlay/` ... a fork of overlay
* `manifest/` ... copy of cros-bazel-manifest repository

## Misc Memo

### Regenerating dependency graph info

When you make changes to ebuilds or to our ebuild analyzer, you need to
regenerate the dependency graph info stored at `bazel/data/deps.json` so that
`update_build` generates `BUILD.bazel` files with up-to-date dependency info.

Run `extract_deps` to regenerate the dependency graph info.

```sh
$ bazel run //bazel/ebuild/cmd/extract_deps
```

Then you can run `generate_build` as usual to update `BUILD.bazel` files.

```sh
$ bazel run //bazel/ebuild/cmd/update_build
```

### Generating package coverage stats

Firstly, build all packages to generate .tbz2 files.

```sh
$ bazel build --keep_going //:all_target_packages
```

Then run `generate_stats` tool to generate a CSV file describing the package
coverage. It inspects `bazel-bin` directory to see if a package was successfully
built or not.

```sh
$ bazel run //bazel/ebuild/cmd/generate_stats > bazel/data/deps.csv
```

### Debugging a failing package

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

To debug an ephemeral CrOS chroot, build a target package with
`--spawn_strategy=standalone` option. On the very first line of its stderr
output, a command line to enter the chroot is printed, just like this:

```
HINT: To debug this build environment, run the Bazel build with --spawn_strategy=standalone, and run the command printed below:
( cd path/to/somewhere && path/to/command --some-options --login )
```

The message is shown without `--spawn_strategy=standalone`, but the printed
command does not work because Bazel uses a temporary execroot.

Related code is located [here](https://team.git.corp.google.com/cros-build-tiger/cros-bazel/+/refs/heads/main/ebuild/cmd/build_package/main.go#190).

### Comparing binary packages

Use `cros_sdk` to enter the standard CrOS chroot, and run `setup_board` and
`build_packages` to build packages for `arm64-generic`. Binary packages will be
saved to `/build/arm64-generic/packages`.

Also follow [the building section](#building) to build packages with Bazel.

Then run the following command to compare binary packages from the two
implementations.

```sh
$ bazel run //bazel/ebuild/cmd/compare_packages
```

### Handling packages that contain BUILD or BUILD.bazel files

Some ebuild packages use `bazel` as their build system, so they will contain
`BUILD` or `BUILD.bazel` files in their src tree. This poses a problem because
by default we `glob(['*'])` the src tree and provide it as an input to the
ebuild. `glob` doesn't traverse into sub-packages (i.e., folders with `BUILD`
files).

To work around this you can:
1) Manually specify a `filegroup` in the sub-package that contains all the src
   for that sub-package and include it as part of the package's root `:src`
   target.
2) Tell `bazel` to ignore the `BUILD` files so that glob works correctly. To do
   this you can use the `./bazel/tools/regenerate_ignore_list` script. Modify
   the script to add your project and then execute the script from the workspace
   root. This will update the `bazel/.bazelrc-BUILD-ignore`.
