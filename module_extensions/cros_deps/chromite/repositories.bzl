# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _chromite_impl(repo_ctx):
    # Callers can use checked-out Chromite by invoking bazel with
    # `USE_PINNED_CHROMITE=false`
    use_pinned_chromite = repo_ctx.os.environ.get("USE_PINNED_CHROMITE") not in ["false", "False"]

    if use_pinned_chromite:
        repo_ctx.download_and_extract(
            url = "https://storage.googleapis.com/chromeos-localmirror/chromite-bundles/chromite-20240809_152114-7fc849f7df1ea015da6bafc7abfae4dbfc215675.tar.zst",
            sha256 = "ab6a68cdea708c8158818bd5c74e7645e38a677ae12170f970e6ad6b030fd83c",
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
