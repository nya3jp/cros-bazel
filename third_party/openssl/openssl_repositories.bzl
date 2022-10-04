"""A module defining the third party dependency OpenSSL"""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

VERSION = "1.1.1o"
CHECKSUM = "0f745b85519aab2ce444a3dcada93311ba926aea2899596d01e7f948dbd99981"

_UNDERSCORED = VERSION.replace(".", "_")

def openssl_repositories():
    maybe(
        http_archive,
        name = "openssl",
        build_file = Label("//bazel/third_party/openssl:BUILD.openssl.bazel"),
        sha256 = CHECKSUM,
        strip_prefix = "openssl-OpenSSL_%s" % _UNDERSCORED,
        urls = [
            "https://mirror.bazel.build/www.openssl.org/source/openssl-%s.tar.gz" % VERSION,
            "https://www.openssl.org/source/openssl-%s.tar.gz" % VERSION,
            "https://github.com/openssl/openssl/archive/OpenSSL_%s.tar.gz" % _UNDERSCORED,
        ],
    )
