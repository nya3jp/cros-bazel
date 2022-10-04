load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("//bazel/third_party:github_archive.bzl", "github_archive")

PY2_VERSION = "2.7.18"
PY2_CHECKSUM = "da3080e3b488f648a3d7a4560ddee895284c3380b11d6de75edb986526b9a814"

PY3_VERSION = "3.10.1"
PY3_CHECKSUM = "b76117670e7c5064344b9c138e141a377e686b9063f3a8a620ff674fa8ec90d3"

RULES_VERSION = "0.5.0"
RULES_CHECKSUM = "cd6730ed53a002c56ce4e2f396ba3b3be262fd7cb68339f0377a45e8227fe332"

def python_repositories():
    maybe(
        http_archive,
        name = "python2",
        build_file = Label("//bazel/toolchains/python:BUILD.python2.bazel"),
        strip_prefix = "Python-%s" % PY2_VERSION,
        urls = [
            "https://www.python.org/ftp/python/{version}/Python-{version}.tgz".format(version = PY2_VERSION),
        ],
        sha256 = PY2_CHECKSUM,
    )
    maybe(
        http_archive,
        name = "python3",
        build_file = Label("//bazel/toolchains/python:BUILD.python3.bazel"),
        strip_prefix = "Python-%s" % PY3_VERSION,
        urls = [
            "https://www.python.org/ftp/python/{version}/Python-{version}.tgz".format(version = PY3_VERSION),
        ],
        sha256 = PY3_CHECKSUM,
    )

    github_archive(
        name = "rules_python",
        github_user = "bazelbuild",
        github_repo = "rules_python",
        strip_prefix = "",
        tag = RULES_VERSION,
        checksum = RULES_CHECKSUM,
    )
