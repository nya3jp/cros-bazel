# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load("//bazel/portage/build_defs:common.bzl", "BinaryPackageInfo")

visibility("public")

def _update_prebuilts_impl(target, ctx):
    metadata = target[BinaryPackageInfo].metadata
    materialized = ctx.actions.declare_file(target.label.name + "_prebuilt_materialized")
    args = ctx.actions.args()
    args.add_all([
        "--metadata",
        metadata,
        "--materialized",
        materialized,
    ])
    disk_cache = ctx.attr._prebuilt_disk_cache[BuildSettingInfo].value
    if disk_cache:
        args.add("--disk_cache", disk_cache)
    ctx.actions.run(
        executable = ctx.executable._update_prebuilts,
        inputs = [metadata],
        outputs = [materialized],
        arguments = [args],
        execution_requirements = {
            "local": "1",
            "no-cache": "1",
            "no-sandbox": "1",
        },
    )
    return [OutputGroupInfo(prebuilt_materialized = depset([materialized]))]

update_prebuilts = aspect(
    implementation = _update_prebuilts_impl,
    attr_aspects = ["*"],
    required_providers = [BinaryPackageInfo],
    attrs = dict(
        _update_prebuilts = attr.label(
            executable = True,
            cfg = "exec",
            default = "//bazel/portage/build_defs:update_prebuilts",
        ),
        _prebuilt_disk_cache = attr.label(
            default = Label("//bazel/portage:prebuilt_disk_cache"),
            providers = [BuildSettingInfo],
        ),
    ),
)
