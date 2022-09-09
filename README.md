# ChromeOS Bazelification

This is an experiment to build ChromeOS with Bazel.

## Checking out

For the prototyping phase, we're working on building a snapshot of ChromiumOS.
Use `repo` to check out a snapshotted ChromiumOS tree + Bazel files.

```sh
$ mkdir cros-bazel
$ cd cros-bazel
$ repo init -u sso://team/cros-build-tiger/cros-bazel-manifest -b main -g minilayout,bazel
$ repo sync -c -j 4
$ cd src
```

## Building

To build a single package (sys-apps/ethtool for example):

```sh
$ bazel build //third_party/portage-stable/sys-apps/ethtool
```

To build all target packages:

```
$ bazel build --keep_going //:all_target_packages
```

## Directory structure

See [manifest/_bazel.xml] for details on how this repository is organized.

[manifest/_bazel.xml]: https://team.git.corp.google.com/cros-build-tiger/cros-bazel-manifest/+/refs/heads/main/_bazel.xml

* `src/`
    * `bazel/` ... contains Bazel-related files
        * `ebuild/`
            * `defs.bzl` ... provides Bazel rules
            * `private/` ... contains programs used by Bazel rules
                * `cmd/`
                    * `create_squashfs/` ... creates squashfs from a file set; used by `overlay` rule
                    * `run_in_container/` ... runs a program within an unprivileged Linux container; used by other programs such as `build_sdk` and `build_package`
                    * `build_sdk/` ... builds SDK squashfs; used by `sdk` rule
                    * `build_package/` ... builds a Portage binary package; used by `ebuild` rule
                    * `update_build/` ... generates BUILD files for ebuilds
        * `config/` ... contains build configs like which overlays to use
        * `sdk/` ... defines SDK to use
    * `overlays/`
        * `overlay-arm64-generic/` ... a fork of overlay
    * `third_party/`
        * `portage-stable/` ... a fork of overlay
        * `eclass-overlay/` ... a fork of overlay
        * `chromiumos-overlay/` ... a fork of overlay
* `manifest/` ... copy of cros-bazel-manifest repository

## Misc Memo

### Generating BUILD files in overlays

Firstly, run `extract_deps` **in CrOS chroot** to extract package dependency
info from ebuilds.

```sh
$ cros_sdk bazel-5 run //bazel/ebuild/private/cmd/extract_deps -- --board=arm64-generic --start=virtual/target-os > bazel/data/deps.json
```

Then you can run `generate_build` to update `BUILD` files.

```sh
$ bazel run //bazel/ebuild/private/cmd/update_build -- --package-info-file $PWD/bazel/data/deps.json
```

### Visualizing the dependency graph

Firstly, build all packages to generate .tbz2 files.

```sh
$ bazel build --keep_going //:all_target_packages
```

Then run `bazel/tools/generate_depgraph.py` to generate a DOT file. It inspects
`bazel-bin` directory to see if a package was successfully built or not.

```sh
$ bazel/tools/generate_depgraph.py bazel/data/deps.json > bazel/data/deps.dot
```
