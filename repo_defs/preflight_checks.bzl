# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Repository rules that perform preflight checks.
"""

def _prominent_fail(message):
    """Similar to fail, but prints a message prominently."""
    fail("\n\n" + "*" * 80 + "\n" + message + "\n" + "*" * 80 + "\n\n")

def _preflight_check_fail(reason):
    """Reports a preflight check failure."""
    _prominent_fail(
        reason +
        "\n\nPlease read the following guide to set up your environment " +
        "properly:\n" +
        "https://chromium.googlesource.com/chromiumos/bazel/+/HEAD/README.md",
    )

def _preflight_checks_impl(repo_ctx):
    """Performs preflight checks."""
    failure = None

    if repo_ctx.os.environ.get("ALCHEMY_EXPERIMENTAL_OUTSIDE_CHROOT") != "1":
        if not repo_ctx.path("/etc/cros_chroot_version").exists:
            failure = "Bazel was run outside CrOS chroot."

    if repo_ctx.os.environ.get("CHROMITE_BAZEL_WRAPPER") != "1":
        failure = "Bazel was run without the proper wrapper."

    llvm_path = repo_ctx.workspace_root.get_child("third_party/llvm-project")
    if not llvm_path.exists:
        failure = (
            "third_party/llvm-project is not checked out.\n" +
            "Did you run `repo init` with `-g default,bazel`?",
        )

    if repo_ctx.os.environ.get("IS_NESTED_BAZEL") == "1":
        failure = None

    if failure != None:
        _preflight_check_fail(failure)

    # Create an empty repository.
    repo_ctx.file("BUILD.bazel", "")
    repo_ctx.file("ok.bzl", "ok = True")

preflight_checks = repository_rule(
    implementation = _preflight_checks_impl,
    attrs = {},
    environ = [
        "ALCHEMY_EXPERIMENTAL_OUTSIDE_CHROOT",
        "CHROMITE_BAZEL_WRAPPER",
        "IS_NESTED_BAZEL",
    ],
    local = True,
    doc = """
    Performs preflight checks to provide friendly diagnostics to users.

    This rule should be called at the very beginning of WORKSPACE.bazel.
    """,
)

def _symlinks_unavailable_impl(_repo_ctx):
    _preflight_check_fail("Bazel was run without the proper wrapper.")

symlinks_unavailable = repository_rule(
    implementation = _symlinks_unavailable_impl,
    local = True,
    doc = """
    Reports a failure because symlinks are missing.

    This rule should be called in WORKSPACE (the fallback for WORKSPACE.bazel
    that is loaded when symlinks are not properly set up).
    """,
)
