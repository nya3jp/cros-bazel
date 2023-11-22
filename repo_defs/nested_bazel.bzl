# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("public")

def _nested_bazel_impl(repo_ctx):
    srcs = repo_ctx.attr._default_srcs + repo_ctx.attr.srcs

    nested_bazel = repo_ctx.path(repo_ctx.attr._nested_bazel)
    nested_bazel_opts = repo_ctx.os.environ.get("NESTED_BAZEL", None)
    if not nested_bazel_opts:
        fail("NESTED_BAZEL environment variable is required")
    nested_bazel_opts = json.decode(nested_bazel_opts)

    out_dir = repo_ctx.path("")

    repos = {}
    for src in srcs:
        # Empty for main repo.
        if src.workspace_name:
            # Ensures that the repo is up to date.
            repo_ctx.path(src)
            repos[src.workspace_name] = None

    in_file = repo_ctx.path("command_input.json")
    repo_ctx.file(in_file, json.encode(nested_bazel_opts | {
        "build_opts": [
            "--platforms=@@//bazel/platforms:host",
            "--noexperimental_convenience_symlinks",
            "--compilation_mode=opt",
        ],
        "out_dir": str(out_dir),
        "repo_rule_deps": sorted(repos),
        "target": repo_ctx.attr.target,
    }))

    args = [nested_bazel, in_file]
    st = repo_ctx.execute(
        args,
        timeout = repo_ctx.attr.timeout,
        environment = {"SHOW_REPRO": "1"},
        working_directory = str(repo_ctx.workspace_root),
        # This could actually invoke several repo rules and actions under the
        # hood. This ensures that you can see progress.
        quiet = False,
    )
    if st.return_code:
        fail("Error while trying to build %s" % repo_ctx.attr.target)

nested_bazel = repository_rule(
    implementation = _nested_bazel_impl,
    attrs = dict(
        _nested_bazel = attr.label(default = "//bazel/repo_defs:nested_bazel.py"),
        target = attr.string(),
        srcs = attr.label_list(mandatory = True),
        # All nested_bazel invocations have access to the following repos shared
        # from the main invocation.
        _default_srcs = attr.label_list(default = [
            "@aspect_bazel_lib//:BUILD.bazel",
            "@rules_foreign_cc//:BUILD.bazel",
            "@rules_rust//:BUILD.bazel",
            "@toolchain_sdk//:BUILD.bazel",
        ]),
        timeout = attr.int(default = 600),
    ),
    # All nested bazel things should build with the same set of flags as each
    # other. We attempt to match the startup flags of the outer bazel, but don't
    # attempt to match the build flags.
    environ = ["NESTED_BAZEL"],
)
