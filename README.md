# ChromeOS Bazelification

This is an experiment to build ChromeOS with Bazel.

## Checking out

For the prototyping phase, we're working on building a snapshot of ChromiumOS.
Use `repo` to check out a snapshotted ChromiumOS tree + Bazel files.

```
mkdir cros-bazel
cd cros-bazel
repo init -u sso://team/cros-build-tiger/cros-bazel-manifest
repo sync -c
```

## Building

For example, to build sys-apps/ethtool:

```
bazel build //third_party/portage-stable/sys-apps/ethtool
tar tvf bazel-bin/third_party/portage-stable/sys-apps/ethtool/ethtool-4.13.tbz2
```

## Directory structure

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
