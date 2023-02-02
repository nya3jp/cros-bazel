# ChromeOS Bazel Design / Architecture

This document will explain the overall design of the ChromeOS bazel migration and the rational
for why certain things are architected the way that they are.

## Introduction

TODO

## Configuration

ChromeOS has three main sources of configuration data:
1) [ebuild repositories](https://chromium.googlesource.com/chromiumos/docs/+/HEAD/portage/overlay_faq.md) (overlays) - This was the initial configuration format used by ChromeOS. Overlays are organized into various categories that logically group specific traits. These overlays can define parent overlays which forms a graph of configuration nodes where child nodes can override configuration data of parent nodes.
    i.e.,
    > eclass-overlay -> portage-stable -> chromiumos-overlay -> chipset-XXX -> baseboard-YYY -> board-ZZZ.

    As of Jan 2023 we have almost 500 public and private overlays. The primary mechanism of controlling feature enablement is via [USE flags](https://wiki.gentoo.org/wiki/USE_flag).

    [ebuilds](https://devmanual.gentoo.org/ebuild-writing/index.html) are also a wealth of [configuration](https://devmanual.gentoo.org/ebuild-writing/variables/index.html) data. They define all the `USE` flags supported by the package, the valid combinations of those `USE` flags, the different types of dependencies, the constraints that need to be imposed on its dependencies, variables to control how `eclasses` get configured (i.e., [cros-workon](https://source.chromium.org/chromium/chromiumos/overlays/chromiumos-overlay/+/main:eclass/cros-workon.eclass;l=65;drc=bb94461990266fcd3368bbbe9f907497339902f3), [cros-rust](https://source.chromium.org/chromium/chromiumos/overlays/chromiumos-overlay/+/main:eclass/cros-rust.eclass;l=27;drc=bb94461990266fcd3368bbbe9f907497339902f3), etc).

1) [chromeos-config](https://chromium.googlesource.com/chromiumos/platform2/+/HEAD/chromeos-config/README.md) - This configuration system was introduced to help ChromeOS scale. Previously every OEM device that derived from a reference design required its own `board-` overlay. This meant that each model would have its own OS build which is very expensive to maintain.  With the introduction of `chromeos-config` (i.e., `unibuild`) it was now possible to have a single `board-` overlay that supported multiple models. `chromeos-config` does runtime probing of the device and exposes a directory at `/run/chromeos-config` that contains all the various runtime configuration for the device. There are some ebuilds that need to generate per-model artifacts. These ebuilds consume the `chromeos config` at build time and iterate over all the defined models. Since a single build is used for all models, the `USE` flags set by the `board-` overlay must be compatible with all the models.

1) [boxster](go/cros-boxster-site) - Boxster was a reenvisioning of `chromeos-config`. It uses `proto` and `starlark` as the configuration language and provides a lot more structure to how configuration is defined. The boxster configuration gets transformed into the same output format as `chromeos-config`. This insulates the ebuilds and devices from having to learn about a new configuration system.

With the migration to Bazel, we have an option for another configuration model (insert `xkcd` link here), [bazel platforms](https://bazel.build/extending/platforms). Bazel platforms can be used to tell bazel how to the target should be compiled, which features to enable, what libraries to link in, etc.

### USE Flags

Due to the massive amounts of portage configuration used by ChromeOS and the constant churn of that configuration, it doesn't make sense to take on a massive configuration conversion effort as part of the bazel migration. Instead we will embrace `USE` flags and the portage configuration model. Due to the [expressiveness](https://dev.gentoo.org/~ulm/pms/head/pms.html#section-8.2) of the `USE` flag expressions, and the complexity of how `USE` flags get computed it's practically impossible to convert the `USE` flag model into `bazel` [constraint_settings](https://bazel.build/reference/be/platform#constraint_setting) and [select](https://bazel.build/reference/be/functions#select) clauses. Instead we will preprocesses all of the portage configuration settings and bake their final values into the generated `BUILD.bazel` files.

```
ebuild(
    name = "8.1_p1-r1",
    ebuild = "readline-8.1_p1-r1.ebuild",
    distfiles = {
        "@portage-dist_readline-8.1.tar.gz//file": "readline-8.1.tar.gz",
        "@portage-dist_readline81-001//file": "readline81-001",
    },
    build_deps = [
        "//internal/overlays/third_party/portage-stable/sys-libs/ncurses:5.9-r99",
    ],
    runtime_deps = [
        "//internal/overlays/third_party/portage-stable/sys-libs/ncurses:5.9-r99",
    ],
    files = glob(["files/**", "*.bashrc"]),
    use = ["-cros_host", "-static-libs", "unicode", "utils"],
    eclasses = [
        "//internal/overlays/third_party/portage-stable/eclass:flag-o-matic",
        "//internal/overlays/third_party/chromiumos-overlay/eclass:toolchain-funcs",
        "//internal/overlays/third_party/portage-stable/eclass:usr-ldscript",
    ],
    sdk = "//internal/sdk",
    visibility = ["//visibility:public"],
)
```

This removes bazel from having to understand anything about `USE` flags, portages dependency resolution logic, and keeps the generated `BUILD` files readable.
