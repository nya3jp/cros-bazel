# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _chromite_impl(repo_ctx):
    # Callers can use checked-out Chromite by invoking bazel with
    # `USE_PINNED_CHROMITE=false`
    use_pinned_chromite = repo_ctx.os.environ.get("USE_PINNED_CHROMITE") not in ["false", "False"]

    if use_pinned_chromite:
        repo_ctx.download_and_extract(
            url = "https://storage.googleapis.com/chromeos-localmirror/chromite-bundles/chromite-20240614_180701-7559f18d7844ac156ac74d49a4b5823218079a86.tar.zst",
            sha256 = "38118f49d3d8a04c85a85b3a44dade2e3f0e2b20eaf624d471440c0390acc5d1",
        )
    else:
        # While most repo rules would inject BUILD.project-chromite during the repo
        # rule, since we perform a symlink, doing so would add it to the real
        # chromite directory.
        realpath = str(repo_ctx.workspace_root.realpath).rsplit("/", 1)[0]
        out = repo_ctx.path(".")
        repo_ctx.symlink(realpath + "/chromite", out)

chromite = repository_rule(
    implementation = _chromite_impl,
    environ = ["USE_PINNED_CHROMITE"],
)
