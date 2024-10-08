# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Module extensions to generate the @portage repo.

We have to split this into 2 extensions, because module extensions cannot read
any files generated by repos declared in their own module extension (this would
create circular dependencies). However, they can read files generated by repos
declared in other module extensions."""

load("//bazel/module_extensions/portage:portage.bzl", _portage = "portage")
load("//bazel/module_extensions/portage:portage_digest.bzl", "portage_digest")
load("//bazel/module_extensions/portage:remoteexec_info.bzl", "remoteexec_info")
load("//bazel/module_extensions/private:hub_repo.bzl", "hub_init")
load("//bazel/portage/bin/alchemist/src/bin/alchemist:repo_rule_srcs.bzl", "ALCHEMIST_REPO_RULE_SRCS")
load("//bazel/portage/repo_defs/chrome:cros_chrome_repository.bzl", _cros_chrome_repository = "cros_chrome_repository")
load("//bazel/repo_defs:nested_bazel.bzl", "nested_bazel")
load("//bazel/repo_defs:preflight_checks.bzl", "portage_preflight_checks")
load("//bazel/repo_defs:repo_repository.bzl", _repo_repository = "repo_repository")

def _portage_impl(module_ctx):
    portage_preflight_checks(
        name = "portage_preflight_checks",
    )
    nested_bazel(
        name = "alchemist",
        target = "//bazel/portage/bin/alchemist/src/bin/alchemist",
        srcs = ALCHEMIST_REPO_RULE_SRCS,
    )
    portage_digest(
        name = "portage_digest",
        alchemist = "@alchemist//:alchemist",
        preflight_checks_ok = "@portage_preflight_checks//:ok.bzl",
    )
    remoteexec_info(
        name = "remoteexec_info",
    )

    _portage(
        name = "portage",
        board = "@portage_digest//:board",
        profile = "@portage_digest//:profile",
        digest = "@portage_digest//:digest",
        alchemist = "@alchemist//:alchemist",
    )

portage = module_extension(
    implementation = _portage_impl,
    environ = ["NESTED_ALCHEMIST"],
)

def _portage_deps_impl(module_ctx):
    deps_path = module_ctx.path(Label("@portage//:deps.json"))

    deps = json.decode(module_ctx.read(deps_path))
    hub = hub_init()
    cros_chrome_repository = hub.wrap_rule(
        _cros_chrome_repository,
        default_targets = {
            "cipd-cache": "//:cipd-cache",
            "src": "//:src",
        },
    )
    repo_repository = hub.wrap_rule(
        _repo_repository,
        default_targets = {"src": "//:src"},
    )

    for repo in deps:
        for rule, kwargs in repo.items():
            name = kwargs["name"]
            if rule == "HttpFile":
                hub.http_file.alias_only(**kwargs)
            elif rule == "GsFile":
                hub.gs_file.alias_only(**kwargs)
            elif rule == "RepoRepository":
                repo_repository.alias_only(**kwargs)
            elif rule == "CipdFile":
                hub.cipd_file.alias_only(**kwargs)
            elif rule == "CrosChromeRepository":
                cros_chrome_repository.alias_only(**kwargs)
            else:
                fail("Unknown rule %s" % rule)

    hub.generate_hub_repo(
        name = "portage_deps",
        visibility = ["@portage//:__subpackages__"],
    )

portage_deps = module_extension(
    implementation = _portage_deps_impl,
)
