# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

def github_archive(name, github_user, github_repo, tag, checksum = "", strip_prefix = None, patch_args = None, patch_strip = 1, **kwargs):
    """A rule to download a release from github.

    Args:
        name: The name of the repo (it will be accessed with @name)
        github_user: The github user the repo is hosted under.
        github_repo: The name of the github repo.
        tag: The tag name to download.
        checksum: The sha256 checksum of the archive.
        strip_prefix: If provided, override the default prefix to strip from the
          files in the archive.
        patch_args: If provided, overrides the arguments to the patching tool.
        patch_strip: Adds -p<patch_strip> to patch_args.
        **kwargs: Kwargs to pass through to http_archive.
          See https://bazel.build/rules/lib/repo/http#http_archive

    Useful optional args:
        build_file: Pass build_file = Label("//path/to:BUILD.name.bazel").
          This creates a build file for non bazel repos so you can access them.
        patches: If any patches are required, put them here.
    """
    if strip_prefix == None:
        strip_prefix = "{repo}-{tag}".format(repo = github_repo, tag = tag)
    patch_args = patch_args or []
    patch_args.append("-p%d" % patch_strip)

    maybe(
        http_archive,
        name = name,
        sha256 = checksum,
        strip_prefix = strip_prefix,
        # In the future we may want to add our own mirror here. This could
        # become an issue if repos are deleted.
        urls = [
            "https://github.com/{user}/{repo}/archive/refs/tags/{tag}.tar.gz".format(
                user = github_user,
                repo = github_repo,
                tag = tag,
            ),
        ],
        patch_args = patch_args,
        **kwargs
    )
