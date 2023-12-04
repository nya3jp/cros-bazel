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

    reclient_dir = repo_ctx.os.environ.get("RECLIENT_DIR")
    if reclient_dir:
        remoteexec_info_dict["reclient_dir"] = reclient_dir

    reproxy_cfg = repo_ctx.os.environ.get("REPROXY_CFG")
    if reproxy_cfg:
        remoteexec_info_dict["reproxy_cfg"] = reproxy_cfg

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

    # Temporarily disable remoteexec for CI builders. b/313796569
    if remoteexec_info_dict["should_use_reproxy_cfg_file_for_ci"]:
        print("Temporarily disabling remoteexec for CI builders. b/313796569")
        remoteexec_info_dict["use_remoteexec"] = False

    if remoteexec_info_dict["use_remoteexec"]:
        print("Remoteexec is enabled. Going to use remoteexec to build chromeos-chrome.")
        print("gcloud_config_dir=" + str(remoteexec_info_dict.get("gcloud_config_dir")))
        print("reclient_dir=" + str(remoteexec_info_dict.get("reclient_dir")))
        print("reproxy_cfg=" + str(remoteexec_info_dict.get("reproxy_cfg")))
        print("should_use_reproxy_cfg_file_for_ci=" + str(remoteexec_info_dict.get("should_use_reproxy_cfg_file_for_ci")))
        print("envs=" + str(remoteexec_info_dict.get("envs")))

    remoteexec_info = json.encode(remoteexec_info_dict)

    repo_ctx.file("remoteexec_info", content = remoteexec_info)
    repo_ctx.file("BUILD.bazel", content = _REMOTEEXEC_INFO_REPO_BUILD_FILE)

remoteexec_info = repository_rule(
    implementation = _remoteexec_info_repository_impl,
    environ = _PASSTHROUGH_ENVIRON + [
        "HOME",
        "RECLIENT_DIR",
        "REPROXY_CFG",
        "USE_REMOTEEXEC",
    ],
    local = True,
)
