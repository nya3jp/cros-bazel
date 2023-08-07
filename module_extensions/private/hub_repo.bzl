# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive", "http_file")
load("//bazel/repo_defs:cipd.bzl", "cipd_file")
load("//bazel/repo_defs:gs.bzl", "gs_file")

# This follows the hub-and-spoke model recommended by bzlmod in
# https://github.com/bazelbuild/bazel/issues/17048#issuecomment-1357752280
def _hub_repo_impl(repo_ctx):
    symlinks = repo_ctx.attr.symlinks
    aliases = repo_ctx.attr.aliases
    content = ['load("@cros//bazel/module_extensions/private:symlink.bzl", "symlink")']

    def add_target(rule, name, label):
        content.extend([
            "",
            "%s(" % rule,
            '    name="%s",' % name,
            '    actual="%s",' % label,
            '    visibility=["//visibility:public"]',
            ")",
        ])

    for name, label in sorted(symlinks.items()):
        add_target("symlink", name, label)
    for name, label in sorted(aliases.items()):
        add_target("alias", name, label)

    repo_ctx.file("BUILD.bazel", "\n".join(content))

_hub_repo = repository_rule(
    implementation = _hub_repo_impl,
    attrs = dict(
        symlinks = attr.string_dict(),
        aliases = attr.string_dict(),
    ),
)

def hub_init():
    aliases = {}
    symlinks = {}

    def wrap(wrapper, default_targets):
        def labels(name, targets):
            targets = targets or default_targets
            if type(targets) == type(""):
                targets = {None: targets}

            result = {}
            for alias, target in targets.items():
                alias = (name + "_" + alias) if alias else name
                if not target.startswith("//"):
                    fail("Target must start with '//'")
                result[alias] = "@" + name + target
            return result

        def alias(name, targets = None, **kwargs):
            for k, v in labels(name, targets).items():
                aliases[k] = v
            wrapper(name = name, **kwargs)

        def symlink(name, targets = None, **kwargs):
            for k, v in labels(name, targets).items():
                symlinks[k] = v
            wrapper(name = name, **kwargs)

        return struct(
            alias = alias,
            symlink = symlink,
        )

    def generate_hub_repo(name, **kwargs):
        _hub_repo(name = name, aliases = aliases, symlinks = symlinks, **kwargs)

    return struct(
        wrap_rule = wrap,
        http_file = wrap(http_file, default_targets = "//file"),
        http_archive = wrap(http_archive, default_targets = "//:src"),
        gs_file = wrap(gs_file, default_targets = "//file"),
        cipd_file = wrap(cipd_file, default_targets = "//file"),
        generate_hub_repo = generate_hub_repo,
    )
