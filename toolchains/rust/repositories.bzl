load("//rules_cros/third_party:github_archive.bzl", "github_archive")

RULES_VERSION = "0.15.0"
CHECKSUM = "d541276e940ee84ab7f1531cc332f8f7320036a15c77379d5634e43fa4ed5f96"

def rust_repositories():
    github_archive(
        name = "rules_rust",
        checksum = CHECKSUM,
        github_user = "bazelbuild",
        github_repo = "rules_rust",
        tag = RULES_VERSION,
        patches = [
            # https://github.com/bazelbuild/rules_rust/pull/1791
            "//bazel/toolchains/rust:patches/path-env.patch",
            "//bazel/toolchains/rust:patches/fix-update-crates.patch",
            "//bazel/toolchains/rust:patches/home-directory-env.patch",
        ],
    )
