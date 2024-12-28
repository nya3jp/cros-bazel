# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _chromite_impl(repo_ctx):
    # Callers can use checked-out Chromite by invoking bazel with
    # `USE_PINNED_CHROMITE=false`
    use_pinned_chromite = repo_ctx.os.environ.get("USE_PINNED_CHROMITE") not in ["false", "False"]

    if use_pinned_chromite:
        repo_ctx.download_and_extract(
            url = "https://storage.googleapis.com/chromeos-localmirror/chromite-bundles/chromite-20241220_162229-68a28e35463cc96b8f2dd4be623d357a7cb0227a.tar.zst",
            sha256 = "1efa9b4040385d670422bff0c1de29fbb0b4de53a9d1c6db8004e4e91fc1e028",
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
