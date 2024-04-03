# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generates Cargo.toml files from dependencies specified in the build file."""

load("@rules_rust//rust:rust_common.bzl", "CrateInfo", "DepInfo")
load("//bazel/build_defs/jinja_template:render_template.bzl", "render_template_to_source")

visibility("//bazel/build_defs")

def _relative_path(path, relative_to):
    common = 0
    for i in range(min(len(path), len(relative_to))):
        if path[i] == relative_to[i]:
            common += 1
        else:
            break
    return "../" * (len(relative_to) - common) + "/".join(path[common:])

def _calculate_dependencies(dep_info, base_dir):
    deps = {}
    for dep in dep_info.direct_crates.to_list():
        dir = dep.dep.output.short_path.rsplit("/", 1)[0]
        dir_parts = dir.split("/")

        # Entries defined by crate.from_cargo will be of the form
        # ../rules_rust~~crate~<name>__crate.
        # For example, ../rules_rust~~crate~alchemy_crates__tempfile-3.4.0.
        has_manifest = dir.startswith("../rules_rust~~crate~")

        # If the path starts with "..", then it was defined in a repo rule.
        # Thus, there's no standard relative path from this package to the
        # path in the repo rule.
        relative = None
        if dir_parts[0] != "..":
            relative = _relative_path(dir_parts, relative_to = base_dir)

        # msta@ created a crate on crates.io called runfiles so the IDE can
        # better support this.
        if dir_parts[-3:] == ["rules_rust~", "tools", "runfiles"]:
            has_manifest = True
        deps[dep.dep.name] = struct(
            relative = relative,
            alias = dep.name,
            has_manifest = has_manifest,
        )

    return deps

def _crate_metadata_impl(ctx):
    crate = ctx.attr.crate[CrateInfo]
    dir = crate.output.short_path.split("/")[:-1]

    deps = _calculate_dependencies(ctx.attr.crate[DepInfo], dir)
    dev_deps = {
        k: v
        for k, v in _calculate_dependencies(ctx.attr.test[DepInfo], dir).items()
        if k not in deps
    }

    metadata = struct(
        name = crate.name,
        dir = dir,
        deps = deps,
        dev_deps = dev_deps,
        label = str(ctx.label.same_package_label(ctx.attr.alias)),
        edition = crate.edition,
        features = {},
    )

    out = ctx.actions.declare_file(ctx.label.name + ".json")
    ctx.actions.write(out, json.encode(metadata))

    return [DefaultInfo(files = depset([out]))]

_crate_metadata = rule(
    implementation = _crate_metadata_impl,
    attrs = dict(
        alias = attr.string(mandatory = True),
        crate = attr.label(mandatory = True),
        test = attr.label(),
    ),
)

def generate_cargo_toml(*, name, crate, test = None):
    """Generates a Cargo.toml file, and writes it back to version control.

    Args:
        name: (str) The name of the rule
        crate: (Label) The label for the binary / library.
        test: (Optional[Label]) The label for the test.
    """
    metadata_name = "_%s_metadata" % name
    generated_name = "_%s_generated" % name

    _crate_metadata(
        name = metadata_name,
        alias = name,
        testonly = True,
        crate = crate,
        test = test,
        visibility = ["//visibility:private"],
    )

    render_template_to_source(
        name = name,
        out = "Cargo.toml",
        template = "//bazel/build_defs/generate_cargo_toml:cargo_toml",
        vars_file = metadata_name,
        testonly = True,
    )
