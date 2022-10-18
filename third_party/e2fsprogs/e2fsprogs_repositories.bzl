load("//bazel/third_party:github_archive.bzl", "github_archive")

VERSION = "1.46.5"

CHECKSUM = "0286b718da1491c65c4e51453d33a25d5dad29b0964f915e627c363b4c11cb92"

def e2fsprogs_repositories():
    github_archive(
        name = "e2fsprogs",
        build_file = Label("//bazel/third_party:BUILD.all_srcs.bazel"),
        checksum = CHECKSUM,
        github_repo = "e2fsprogs",
        github_user = "tytso",
        strip_prefix = "e2fsprogs-%s" % VERSION,
        patches = [
            # I filed a PR, so this should hopefully be merged into master at
            # some point.
            # https://github.com/tytso/e2fsprogs/pull/124
            "//bazel/third_party/e2fsprogs:patches/support_offsets.patch",
        ],
        tag = "v%s" % VERSION,
    )
