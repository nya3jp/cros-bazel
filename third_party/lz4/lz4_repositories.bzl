load("//bazel/third_party:github_archive.bzl", "github_archive")

VERSION = "1.9.3"
CHECKSUM = "030644df4611007ff7dc962d981f390361e6c97a34e5cbc393ddfbe019ffe2c1"

def lz4_repositories():
    github_archive(
        name = "lz4",
        build_file = Label("//bazel/third_party:BUILD.all_srcs.bazel"),
        checksum = CHECKSUM,
        github_user = "lz4",
        github_repo = "lz4",
        tag = VERSION,
    )
