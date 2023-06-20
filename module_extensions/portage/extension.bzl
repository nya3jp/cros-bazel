# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
load("//bazel:repo/cipd.bzl", "cipd_file")
load("//bazel:repo/repo_repository.bzl", "repo_repository")
load("//bazel/chrome:cros_chrome_repository.bzl", "cros_chrome_repository")
load("//bazel/module_extensions/portage:alchemist.bzl", "alchemist")
load("//bazel/module_extensions/portage:portage_digest.bzl", "portage_digest")
load("//bazel/module_extensions/portage:portage_tarball.bzl", "portage_tarball")
load("//bazel/module_extensions/private:extract_tarball.bzl", "extract_tarball")
load("//bazel/module_extensions/private:hub_repo.bzl", "hub_repo")

"""Module extensions to generate the @portage repo.

We have to split this into 2 extensions, because module extensions cannot read
any files generated by repos declared in their own module extension (this would
create circular dependencies). However, they can read files generated by repos
declared in other module extensions."""

def _pre_portage_impl(module_ctx):
    alchemist(name = "alchemist")
    portage_digest(
        name = "portage_digest",
        alchemist = "@alchemist//:alchemist",
    )

    # Ideally, we'd just generate the directory from inside the module
    # extension, then read deps.json and generate one repo per dependency.
    # However, until github.com/bazelbuild/bazel/issues/14554 is released,
    # bzlmod won't cache module extension results. This means that if the module
    # extension is rerun, even if it generates the exact same repo rules, it
    # will rerun the repo rules regardless.

    # So for now, we just do a hack where we tar it up and untar it again, to
    # deal with the visibility issues (both @portage and the deps need to be
    # generated in the same module extension for @portage to see the deps).

    # TODO: we can get rid of this tar / untar step once we migrate all deps to
    #  use the dep defined in the hub repo "@portage_deps" instead of the spoke
    #  repos.
    portage_tarball(
        name = "portage_tarball",
        board = "@portage_digest//:board",
        profile = "@portage_digest//:profile",
        digest = "@portage_digest//:digest",
        alchemist = "@alchemist//:alchemist",
    )

pre_portage = module_extension(
    implementation = _pre_portage_impl,
)

def _portage_impl(module_ctx):
    deps_path = module_ctx.path(Label("@portage_tarball//:deps.json"))
    extract_tarball(name = "portage", tarball = "@portage_tarball//:repo.tar.gz")

    deps = json.decode(module_ctx.read(deps_path))

    aliases = {}
    symlinks = {}
    for repo in deps:
        for rule, kwargs in repo.items():
            name = kwargs["name"]
            if rule == "HttpFile":
                http_file(**kwargs)
                symlinks[name] = "@%s//file" % name
            elif rule == "RepoRepository":
                repo_repository(**kwargs)
                aliases["%s_src" % name] = "@%s//:src" % name
            elif rule == "CipdFile":
                cipd_file(**kwargs)
                symlinks[name] = "@%s//file" % name
            elif rule == "CrosChromeRepository":
                cros_chrome_repository(**kwargs)
                aliases["%s_src" % name] = "@%s//:src" % name
                aliases["%s_src_internal" % name] = "@%s//:src_internal" % name
            else:
                fail("Unknown rule %s" % rule)

    hub_repo(
        name = "portage_deps",
        symlinks = symlinks,
        aliases = aliases,
        visibility = ["@portage//:all_packages"],
    )

portage = module_extension(
    implementation = _portage_impl,
)