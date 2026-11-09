# Working with rust

Our rust toolchain is very different from cargo, and fits into bazel's norms.

## Creating a new 1st-party crate
For now, writing crates in pure bazel doesn't play nice with cargo's IDE support, so we make the crate support both rust and cargo.
To do so, we:
1) Create a new directory `bazel/path/to/my_crate/src/` containing either main.rs (for a binary crate) or lib.rs (for a library crate)
   Similar to any other language, create a `.rs` file, and then create a `rust_library/binary` target which lists that file as it's `srcs`.
2) Add the following to `bazel/path/to/my_crate/BUILD.bazel`:
```py
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("//bazel/build_defs:generate_cargo_toml.bzl", "generate_cargo_toml")

rust_library(
    name = "my_crate",
    srcs = glob(["src/**/*.rs"]),
    deps = [],
)

rust_test(
    name = "my_crate_test",
    size = "small",
    crate = ":my_crate",
    deps = [],
)

generate_cargo_toml(
    name = "cargo_toml",
    crate = ":my_crate",
    tests = [":my_crate_test"],
)
```
3) Add `//bazel/path/to/my_crate:cargo_toml` to `//bazel:cargo_workspace`'s deps.
4) You will now have failing tests. They'll give you instructions on how to fix them, but the TLDR is:
* `bazel run //bazel/path/to/my_crate:generate_cargo_toml`
* `bazel run //bazel:generate_cargo_files`


## Adding dependencies on crates
Note that when adding dependencies, you will probably break some diff tests. However, you don't need to worry too much, since `SKIP_PORTAGE_TESTS=1 bazel/portage/tools/run_tests.sh` should pick up on these

#### 1st-party crates
Add the corresponding target to your deps.

#### 3rd-party crates
Add a dep on `@crates//:<crate name>` (for chromeos) or `@alchemy_crates//:<crate name>` (for infra).

If the crate doesn't exist, or if the version needs to be updated, then:

##### @alchemy_crates
1. Modify `//bazel/rust/alchemy_crates/Cargo.toml` with the new or updated crate dependency.
2. Regenerate the vendored Bazel definitions and lockfile:
   ```bash
   chromite/bin/bazel run //bazel/rust/alchemy_crates:crates_vendor
   ```
   To also update transitive dependencies in the lockfile (`cargo update`):
   ```bash
   chromite/bin/bazel run //bazel/rust/alchemy_crates:crates_vendor -- --repin
   ```
   Or to upgrade a specific package:
   ```bash
   chromite/bin/bazel run //bazel/rust/alchemy_crates:crates_vendor -- --repin=<package_name>
   ```
3. If new crates or crate versions were added, mirror their source tarballs to Google Storage so CI builders can download them hermetically:
   ```bash
   src/bazel/content_mirror/force_mirror.py $(chromite/bin/bazel query 'kind("alias", @alchemy_crates//:*)')
   ```
4. After you've added a crate that uses it, update workspace Cargo files:
   ```bash
   chromite/bin/bazel run //bazel:generate_cargo_files
   ```
   (Don't worry, if you forget this step, a test will fail and tell you you need to run this command).

##### @crates
Modify `//third_party/rust_crates/projects/.../Cargo.toml`, then run `third_party/rust_crates/vendor.py`.
