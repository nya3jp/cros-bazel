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
