load("//bazel/third_party/openssl:openssl_repositories.bzl", "openssl_repositories")
load("//bazel/third_party/zlib:zlib_repositories.bzl", "zlib_repositories")

def third_party_repositories():
    openssl_repositories()
    zlib_repositories()
