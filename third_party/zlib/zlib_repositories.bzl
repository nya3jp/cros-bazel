"""A module defining the third party dependency zlib"""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

VERSION = "1.2.12"
CHECKSUM = "91844808532e5ce316b3c010929493c0244f3d37593afd6de04f71821d5136d9"

def zlib_repositories():
    maybe(
        http_archive,
        name = "zlib",
        build_file = Label("//bazel/third_party/zlib:BUILD.zlib.bazel"),
        sha256 = CHECKSUM,
        strip_prefix = "zlib-%s" % VERSION,
        urls = [
            "https://zlib.net/zlib-%s.tar.gz" % VERSION,
            "https://storage.googleapis.com/mirror.tensorflow.org/zlib.net/zlib-%s.tar.gz" % VERSION,
        ],
    )
