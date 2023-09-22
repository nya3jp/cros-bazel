# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_GOMA_INFO_REPO_BUILD_FILE = """
exports_files(["goma_info"])
"""

def _goma_info_repository_impl(repo_ctx):
    """Repository rule to generate info needed to use goma."""

    goma_info_dict = {
        "use_goma": repo_ctx.os.environ.get("USE_GOMA") == "true",
    }

    # Use GOMA_OAUTH2_CONFIG_FILE as the oauth2 config file path if specified.
    # Otherwise, use "$HOME/.goma_client_oauth2_config" if exists.
    oauth2_config_file = repo_ctx.os.environ.get("GOMA_OAUTH2_CONFIG_FILE")
    if not oauth2_config_file:
        home = repo_ctx.os.environ.get("HOME")
        if home:
            default_oauth2_config_file = home + "/.goma_client_oauth2_config"
            if repo_ctx.path(default_oauth2_config_file).exists:
                oauth2_config_file = default_oauth2_config_file
    if oauth2_config_file:
        goma_info_dict["oauth2_config_file"] = oauth2_config_file

    luci_context = repo_ctx.os.environ.get("LUCI_CONTEXT")
    if luci_context:
        goma_info_dict["luci_context"] = luci_context

    if goma_info_dict["use_goma"]:
        print("Goma is enabled. Going to use goma to build chromeos-chrome.")
        print("luci_context=" + str(goma_info_dict.get("luci_context")))
        print("oauth2_config_file=" + str(goma_info_dict.get("oauth2_config_file")))

    goma_info = json.encode(goma_info_dict)

    repo_ctx.file("goma_info", content = goma_info)
    repo_ctx.file("BUILD.bazel", content = _GOMA_INFO_REPO_BUILD_FILE)

goma_info = repository_rule(
    implementation = _goma_info_repository_impl,
    environ = [
        "GOMA_OAUTH2_CONFIG_FILE",
        "HOME",
        "LUCI_CONTEXT",
        "USE_GOMA",
    ],
    local = True,
)
