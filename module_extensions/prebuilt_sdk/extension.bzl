# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/module_extensions/private:hub_repo.bzl", "hub_init")
load(":repo.bzl", "prebuilt_sdk_repo", "repo_name")

visibility("private")

def _prebuilt_sdk_impl(module_ctx):
    for mod in module_ctx.modules:
        for tag in mod.tags.from_url:
            prebuilt_sdk_repo(
                name = tag.name,
                url = tag.url,
                downloaded_file_path = "manifest.json",
            )

_from_url = tag_class(attrs = dict(
    name = attr.string(mandatory = True),
    url = attr.string(mandatory = True),
))

prebuilt_sdk = module_extension(
    implementation = _prebuilt_sdk_impl,
    tag_classes = dict(from_url = _from_url),
)

def _prebuilt_sdk_tarballs_impl(module_ctx):
    hub = hub_init()

    # A "set" of uris (Dict[uri, None]). Used to dedupe.
    uris = {}
    for mod in module_ctx.modules:
        for tag in mod.tags.from_manifests:
            for repo in tag.manifests:
                manifest = json.decode(module_ctx.read(repo))
                for provider in manifest["providers"]:
                    uris[provider["uri"]] = None
                uris[manifest["extractor"]] = None

    for uri in uris:
        hub.gs_file.alias_only(
            name = repo_name(uri),
            url = uri,
        )

    hub.generate_hub_repo(name = "prebuilt_sdk_tarballs")

_from_manifests = tag_class(attrs = dict(
    manifests = attr.label_list(mandatory = True),
))

prebuilt_sdk_tarballs = module_extension(
    implementation = _prebuilt_sdk_tarballs_impl,
    tag_classes = dict(from_manifests = _from_manifests),
)
