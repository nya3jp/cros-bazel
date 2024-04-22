# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Rules to generate files which contain a list of sources of a target."""

load("@rules_rust//rust:rust_common.bzl", "CrateInfo", "DepInfo")

visibility("public")

def _calculate_repo_rule_srcs_impl(ctx):
    # Dict[str, None]
    srcs = {}
    crate_srcs = []
    for dep in ctx.attr.deps:
        if dep.label.workspace_root != ctx.label.workspace_root:
            srcs[str(dep.label)] = None
        elif CrateInfo in dep and DepInfo in dep:
            crates = [dep[CrateInfo]] + dep[DepInfo].transitive_crates.to_list()
            crate_srcs.append(depset(
                transitive =
                    [crate.data for crate in crates] +
                    [crate.srcs for crate in crates] +
                    [crate.compile_data for crate in crates],
            ))
        else:
            fail("Unsupported dependency type for ", dep)

    for src in depset(transitive = crate_srcs).to_list():
        # Can't depend on generated files in repo rules, and depending on the
        # repo rules themselves makes it more annoying to do version uprevs.
        if src.is_source and not src.path.startswith("external/"):
            # Assume the build files are always in the directory above src/
            if src.path.count("/src/") != 1:
                fail("Invalid path", src.path)
            package, src = src.path.rsplit("/src/", 1)

            # The alchemist crate actually has two bazel packages within it
            # (one for the binary and one for the library).
            if src.startswith("bin/alchemist"):
                package = package + "/src/bin/alchemist"
                src = src.split("/", 2)[2]
            else:
                src = "src/" + src

            # Depend on both the source code and the build file.
            srcs["@cros//%s:%s" % (package, src)] = None
            srcs["@cros//%s:BUILD.bazel" % package] = None

    f = ctx.actions.declare_file(ctx.label.name + ".json")
    ctx.actions.write(f, json.encode(struct(
        var = ctx.attr.variable,
        target = ctx.attr.target,
        srcs = sorted(srcs),
    )))
    return [DefaultInfo(files = depset([f]))]

calculate_repo_rule_srcs = rule(
    implementation = _calculate_repo_rule_srcs_impl,
    attrs = dict(
        variable = attr.string(mandatory = True),
        target = attr.string(mandatory = True),
        deps = attr.label_list(mandatory = True, allow_files = True),
    ),
)
