# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive", "http_file")
load("//bazel/repo_defs:cipd.bzl", "cipd_file")
load("//bazel/repo_defs:gs.bzl", "gs_file")
load("//bazel/repo_defs:prebuilt_binpkg.bzl", "prebuilt_binpkg")

_SELECT_ALIAS_OR_SYMLINK_ERR = """

Use ":{k}_alias" or ":{k}_symlink" instead of ":{k}".
":{k}_symlink" is required if you need to look up the file using rlocation.
Otherwise, ":{k}_alias" is generally preferred.

"""

_EXTRA_ALIAS_ERR = """

{k} is marked as alias_only, rather than alias_and_symlink.
Use ":{k}" instead of ":{k}_alias",

"""

_SYMLINK_IN_ALIAS_ONLY_ERR = """

{k} is marked as alias_only, rather than alias_and_symlink.
If you don't need a symlink, use ":{k}".
If you do, update the repo rule to use alias_and_symlink, then update usages of
":{k}" with ":{k}_alias".

"""

# This follows the hub-and-spoke model recommended by bzlmod in
# https://github.com/bazelbuild/bazel/issues/17048#issuecomment-1357752280
def _hub_repo_impl(repo_ctx):
    content = [
        'load("@cros//bazel/module_extensions/private:symlink.bzl", "symlink")',
        'load("@cros//bazel/build_defs:always_fail.bzl", "always_fail")',
    ]

    for name, kwargs in sorted(repo_ctx.attr.invocations.items()):
        kwargs = json.decode(kwargs)
        content.extend([
            "",
            "%s(" % kwargs.pop("rule"),
            '    name="%s",' % name,
        ])

        for k, v in sorted(kwargs.items()):
            content.append("    %s = %r," % (k, v))
        content.extend([
            '    visibility=["//visibility:public"]',
            ")",
        ])

    repo_ctx.file("BUILD.bazel", "\n".join(content))

_hub_repo = repository_rule(
    implementation = _hub_repo_impl,
    attrs = dict(
        invocations = attr.string_dict(),
    ),
)

def hub_init():
    invocations = {}

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

        def alias_only(name, targets = None, **kwargs):
            for k, v in labels(name, targets).items():
                invocations[k] = dict(rule = "alias", actual = v)

                # Some people may assume that this is an 'alias_and_symlink'
                # rule. Give them a helpful error message.
                invocations[k + "_alias"] = dict(
                    rule = "always_fail",
                    message = _EXTRA_ALIAS_ERR.format(k = k),
                )
                invocations[k + "_symlink"] = dict(
                    rule = "always_fail",
                    message = _SYMLINK_IN_ALIAS_ONLY_ERR.format(k = k),
                )
            wrapper(name = name, **kwargs)

        def alias_and_symlink(name, targets = None, **kwargs):
            for k, v in labels(name, targets).items():
                # People will (quite reasonably) assume that the label has no
                # suffix. Give them a helpful error message in this case.
                invocations[k] = dict(
                    rule = "always_fail",
                    message = _SELECT_ALIAS_OR_SYMLINK_ERR.format(k = k),
                )
                invocations[k + "_alias"] = dict(rule = "alias", actual = v)
                invocations[k + "_symlink"] = dict(rule = "symlink", out = k, actual = v)
            wrapper(name = name, **kwargs)

        return struct(
            alias_only = alias_only,
            alias_and_symlink = alias_and_symlink,
        )

    def generate_hub_repo(name, **kwargs):
        _hub_repo(
            name = name,
            invocations = {k: json.encode(v) for k, v in invocations.items()},
            **kwargs
        )

    return struct(
        wrap_rule = wrap,
        http_file = wrap(http_file, default_targets = "//file"),
        http_archive = wrap(http_archive, default_targets = "//:src"),
        gs_file = wrap(gs_file, default_targets = "//file"),
        cipd_file = wrap(cipd_file, default_targets = "//file"),
        prebuilt_binpkg = wrap(prebuilt_binpkg, default_targets = "//:binpkg"),
        generate_hub_repo = generate_hub_repo,
    )
