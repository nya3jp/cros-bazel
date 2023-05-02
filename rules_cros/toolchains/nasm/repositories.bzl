"""A module defining the third party dependency OpenSSL"""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

VERSION = "2.15.05"
CHECKSUM = "f5c93c146f52b4f1664fa3ce6579f961a910e869ab0dae431bd871bdd2584ef2"

def nasm_repositories():
    maybe(
        http_archive,
        name = "nasm",
        build_file = Label("//rules_cros/toolchains/nasm:BUILD.nasm.bazel"),
        sha256 = CHECKSUM,
        strip_prefix = "nasm-%s" % VERSION,
        urls = [
            "https://mirror.bazel.build/www.nasm.us/pub/nasm/releasebuilds/{version}/win64/nasm-{version}-win64.zip".format(version = VERSION),
            "https://www.nasm.us/pub/nasm/releasebuilds/{version}/win64/nasm-{version}-win64.zip".format(version = VERSION),
        ],
    )
