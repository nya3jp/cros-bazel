# Mapping from crate name to build target.
LOCAL_CRATES = {
    "alchemist": "//bazel/ebuild/private/alchemist",
    "bazelutil": "//bazel/ebuild/private/common/bazelutil:bazelutil_rust",
    "binarypackage": "//bazel/ebuild/private/common/portage/binarypackage:binarypackage_rust",
    "build_package": "//bazel/ebuild/private/cmd/build_package:build_package_rust",
    "cliutil": "//bazel/ebuild/private/common/cliutil:cliutil_rust",
    "fileutil": "//bazel/ebuild/private/common/fileutil:fileutil_rust",
    "hello_world": "//bazel/toolchains/rust/examples/hello_world",
    "local_crate": "//bazel/toolchains/rust/examples/local_crate",
    "makechroot": "//bazel/ebuild/private/common/makechroot:makechroot_rust",
    "mountsdk": "//bazel/ebuild/private/common/mountsdk:mountsdk_rust",
    "processes": "//bazel/ebuild/private/common/processes:processes_rust",
    "symindex": "//bazel/ebuild/private/common/symindex:symindex_rust",
    "use_local_crate": "//bazel/toolchains/rust/examples/use_local_crate",
    "use_third_party_crate": "//bazel/toolchains/rust/examples/use_third_party_crate",
    "version": "//bazel/ebuild/private/common/standard/version:version_rust",
}
