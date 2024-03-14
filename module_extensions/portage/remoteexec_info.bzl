# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_REMOTEEXEC_INFO_REPO_BUILD_FILE = """
exports_files(["remoteexec_info"])
"""

_PASSTHROUGH_ENVIRON = [
    "GCE_METADATA_HOST",
    "GCE_METADATA_IP",
    "GCE_METADATA_ROOT",
    "REPROXY_CFG_FILE",
]

def _remoteexec_info_repository_impl(repo_ctx):
    """Repository rule to generate info needed to use remoteexec."""

    remoteexec_info_dict = {
        "envs": {},
        "use_remoteexec": repo_ctx.os.environ.get("USE_REMOTEEXEC") == "true",
    }

    for env in _PASSTHROUGH_ENVIRON:
        var = repo_ctx.os.environ.get(env)
        if var:
            remoteexec_info_dict["envs"][env] = var

    home = repo_ctx.os.environ.get("HOME")
    if home:
        gcloud_config_dir = home + "/.config/gcloud"
        if repo_ctx.path(gcloud_config_dir).exists:
            remoteexec_info_dict["gcloud_config_dir"] = gcloud_config_dir

    # TODO(b/312385225): Replace this logic with a cleaner alternative after b/311706335 is done.
    st = repo_ctx.execute(["whoami"])
    if st.return_code:
        fail("Failed to execute whoami: %s%s" % (st.stdout, st.stderr))
    username = st.stdout.strip()
    remoteexec_info_dict["should_use_reproxy_cfg_file_for_ci"] = username == "chrome-bot"

    if remoteexec_info_dict["use_remoteexec"]:
        print("Remoteexec is enabled. Going to use remoteexec to build chromeos-chrome.")
        print("gcloud_config_dir=" + str(remoteexec_info_dict.get("gcloud_config_dir")))
        print("should_use_reproxy_cfg_file_for_ci=" + str(remoteexec_info_dict.get("should_use_reproxy_cfg_file_for_ci")))
        print("envs=" + str(remoteexec_info_dict.get("envs")))

    remoteexec_info = json.encode(remoteexec_info_dict)

    repo_ctx.file("remoteexec_info", content = remoteexec_info)
    repo_ctx.file("BUILD.bazel", content = _REMOTEEXEC_INFO_REPO_BUILD_FILE)

remoteexec_info = repository_rule(
    implementation = _remoteexec_info_repository_impl,
    environ = _PASSTHROUGH_ENVIRON + [
        "HOME",
        "USE_REMOTEEXEC",
    ],
    local = True,
)
