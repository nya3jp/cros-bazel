load("//bazel/third_party/e2fsprogs:e2fsprogs_repositories.bzl", "e2fsprogs_repositories")
load("//bazel/third_party/fuse:fuse_repositories.bzl", "fuse_repositories")
load("//bazel/third_party/lz4:lz4_repositories.bzl", "lz4_repositories")
load("//bazel/third_party/openssl:openssl_repositories.bzl", "openssl_repositories")
load("//bazel/third_party/squashfuse:squashfuse_repositories.bzl", "squashfuse_repositories")
load("//bazel/third_party/zlib:zlib_repositories.bzl", "zlib_repositories")

def third_party_repositories():
    e2fsprogs_repositories()
    fuse_repositories()
    lz4_repositories()
    openssl_repositories()
    squashfuse_repositories()
    zlib_repositories()
