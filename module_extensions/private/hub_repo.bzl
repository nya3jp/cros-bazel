# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

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

hub_repo = repository_rule(
    implementation = _hub_repo_impl,
    attrs = dict(
        symlinks = attr.string_dict(),
        aliases = attr.string_dict(),
    ),
)
