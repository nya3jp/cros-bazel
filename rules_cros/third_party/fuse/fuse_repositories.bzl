load("//rules_cros/third_party:github_archive.bzl", "github_archive")

VERSION = "2.9.8"
CHECKSUM = "ceadc28f033b29d7aa1d7c3a5a267d51c2b572ed4e7346e0f9e24f4f5889debb"

def fuse_repositories():
    github_archive(
        name = "fuse",
        build_file = Label("//rules_cros/third_party:BUILD.all_srcs.bazel"),
        checksum = CHECKSUM,
        github_user = "libfuse",
        github_repo = "libfuse",
        tag = "fuse-%s" % VERSION,
        patches = [
            "//rules_cros/third_party/fuse:patches/fuse-2.9.8-user-option.patch",
            "//rules_cros/third_party/fuse:patches/fuse-2.9.9-closefrom-glibc-2-34.patch",
        ],
    )
