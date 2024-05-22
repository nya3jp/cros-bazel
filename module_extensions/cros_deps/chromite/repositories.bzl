# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _chromite_impl(repo_ctx):
    # Callers can use checked-out Chromite by invoking bazel with
    # `USE_PINNED_CHROMITE=false`
    use_pinned_chromite = repo_ctx.os.environ.get("USE_PINNED_CHROMITE") not in ["false", "False"]

    if use_pinned_chromite:
        repo_ctx.download_and_extract(
            url = "https://storage.googleapis.com/chromeos-localmirror/chromite-bundles/chromite-20240522_212134-5524d70f2294802815ca62ffbef81c4c8fff2c0f.tar.zst",
            sha256 = "54721ab194a6e9a1ce8dda1cd53cb025f957f4b68a111af2c5596768c64c5f77",
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
