MANIFESTS = [
    # Keep this list in sync with the members listed in //:Cargo.toml.
    # I suspect this is needed so the Cargo.toml files are injected into
    # the runtime environment.
    "//:Cargo.toml",
    "//bazel/ebuild/private/alchemist:Cargo.toml",
    "//bazel/ebuild/private/cmd/action_wrapper:Cargo.toml",
    "//bazel/ebuild/private/cmd/build_image:Cargo.toml",
    "//bazel/ebuild/private/cmd/build_package:Cargo.toml",
    "//bazel/ebuild/private/cmd/extract_interface:Cargo.toml",
    "//bazel/ebuild/private/cmd/install_deps:Cargo.toml",
    "//bazel/ebuild/private/cmd/run_in_container:Cargo.toml",
    "//bazel/ebuild/private/cmd/sdk_from_archive:Cargo.toml",
    "//bazel/ebuild/private/cmd/sdk_update:Cargo.toml",
    "//bazel/ebuild/private/common/bazelutil:Cargo.toml",
    "//bazel/ebuild/private/common/cliutil:Cargo.toml",
    "//bazel/ebuild/private/common/fileutil:Cargo.toml",
    "//bazel/ebuild/private/common/makechroot:Cargo.toml",
    "//bazel/ebuild/private/common/mountsdk:Cargo.toml",
    "//bazel/ebuild/private/common/portage/binarypackage:Cargo.toml",
    "//bazel/ebuild/private/common/processes:Cargo.toml",
    "//bazel/ebuild/private/common/run_in_container_lib:Cargo.toml",
    "//bazel/ebuild/private/common/standard/version:Cargo.toml",
    "//bazel/ebuild/private/common/tar:Cargo.toml",
    "//rules_cros/toolchains/rust/examples/hello_world:Cargo.toml",
    "//rules_cros/toolchains/rust/examples/local_crate:Cargo.toml",
    "//rules_cros/toolchains/rust/examples/use_local_crate:Cargo.toml",
    "//rules_cros/toolchains/rust/examples/use_third_party_crate:Cargo.toml",
]
