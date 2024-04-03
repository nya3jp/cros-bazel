# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Rules for rendering templates."""

load("@aspect_bazel_lib//lib:write_source_files.bzl", "write_source_files")
load(":jinja_template.bzl", "JinjaTemplateInfo")

visibility("public")

def _render_template_impl(ctx):
    if bool(ctx.attr.vars) == bool(ctx.attr.vars_file):
        fail("Exactly one of vars and vars_file must be provided")
    if ctx.attr.vars_file:
        vars_file = ctx.file.vars_file
    else:
        vars_file = ctx.actions.declare_file("_" + ctx.label.name + "_vars.json")
        ctx.actions.write(vars_file, content = ctx.attr.vars)

    out = ctx.actions.declare_file(ctx.attr.out or ctx.label.name)

    template = ctx.attr.template[JinjaTemplateInfo]
    args = ctx.actions.args()
    args.add_all([
        template.root,
        vars_file,
        out,
        ctx.label.same_package_label(ctx.attr.regen_name),
    ])
    ctx.actions.run(
        executable = ctx.executable._expand,
        arguments = [args],
        inputs = template.include.to_list() + [vars_file],
        outputs = [out],
    )

    return [DefaultInfo(
        files = depset([out]),
        runfiles = ctx.runfiles(files = [out]),
    )]

_render_template = rule(
    implementation = _render_template_impl,
    attrs = dict(
        vars = attr.string(doc = "Json-encoded mapping from variable to content"),
        vars_file = attr.label(allow_single_file = [".json"]),
        regen_name = attr.string(),
        template = attr.label(providers = [JinjaTemplateInfo], mandatory = True),
        out = attr.string(),
        _expand = attr.label(
            default = "@@//bazel/build_defs/jinja_template:render_template",
            executable = True,
            cfg = "exec",
        ),
    ),
)

def render_template(*, name, vars = None, **kwargs):
    """Expands a jinja2 template.

    Args:
        name: (str) The name of the build rule
        vars: (Mapping[str, json-able]) A mapping from variable names to values.
          Values must be able to be converted to json.
        **kwargs: kwargs to pass to _expand_template
    """
    if type(vars) == "dict":
        vars = json.encode(vars)
    elif vars != None:
        fail("vars must be a dictionary mapping from variable name to value")

    _render_template(
        name = name,
        vars = vars,
        **kwargs
    )

def render_template_to_source(*, name, out, template, vars = None, vars_file = None, **kwargs):
    """Expands a jinja2 template to a source file.

    Args:
        name: (str) The name of the build rule
        out: (str) The filename to output to.
        template: (Label) The jinja_template target to render.
        vars: (Mapping[str, json-able]) A mapping from variable names to values.
          Values must be able to be converted to json.
        vars_file: (Label) json file containing variables
        **kwargs: kwargs to pass to _expand_template
    """
    kwargs.setdefault("visibility", ["//visibility:private"])
    generated_name = "_%s_generated" % name
    render_template(
        name = generated_name,
        regen_name = name,
        vars = vars,
        vars_file = vars_file,
        template = template,
        **kwargs
    )

    write_source_files(
        name = name,
        files = {
            out: generated_name,
        },
        **kwargs
    )
