load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("//bazel/third_party:github_archive.bzl", "github_archive")

PY3_URL = "https://github.com/indygreg/python-build-standalone/releases/download/20221002/cpython-3.10.7+20221002-x86_64_v3-unknown-linux-gnu-pgo+lto-full.tar.zst"
PY3_CHECKSUM = "c54217b3df5f398e52e26e16683f642b245e36232d190ee9fec45a04923de9ca"

RULES_VERSION = "0.5.0"
RULES_CHECKSUM = "cd6730ed53a002c56ce4e2f396ba3b3be262fd7cb68339f0377a45e8227fe332"

BUILD_FILE_CONTENT = """
filegroup(
    name = "files",
    srcs = glob(["install/**"], exclude = ["**/* *"]),
    visibility = ["//visibility:public"],
)

filegroup(
    name = "interpreter",
    srcs = ["python/{interpreter_path}"],
    visibility = ["//visibility:public"],
)
"""

def _python_build_standalone_interpreter_impl(repository_ctx):
    repository_ctx.download_and_extract(
        url = [repository_ctx.attr.url],
        sha256 = repository_ctx.attr.checksum,
    )

    # NOTE: 'json' library is only available in Bazel 4.*.
    python_build_data = json.decode(repository_ctx.read("python/PYTHON.json"))

    repository_ctx.file("BUILD.bazel", BUILD_FILE_CONTENT.format(
        interpreter_path = python_build_data["python_exe"],
    ))

python_build_standalone_interpreter = repository_rule(
    implementation = _python_build_standalone_interpreter_impl,
    attrs = dict(
        url = attr.string(mandatory = True),
        checksum = attr.string(mandatory = True),
    ),
)

def python_repositories():
    github_archive(
        name = "rules_python",
        github_user = "bazelbuild",
        github_repo = "rules_python",
        strip_prefix = "",
        tag = RULES_VERSION,
        checksum = RULES_CHECKSUM,
    )

    python_build_standalone_interpreter(name = "python3_interpreter", url = PY3_URL, checksum = PY3_CHECKSUM)
