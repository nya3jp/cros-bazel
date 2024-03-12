# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Rules for expanding templates."""

visibility("public")

def _expand_template_impl(ctx):
    if bool(ctx.attr.vars) == bool(ctx.attr.vars_file):
        fail("Exactly one of vars and vars_file must be provided")
    if ctx.attr.vars_file:
        vars_file = ctx.file.vars_file
    else:
        vars_file = ctx.actions.declare_file("_" + ctx.label.name + "_vars.json")
        ctx.actions.write(vars_file, content = ctx.attr.vars)

    out = ctx.actions.declare_file(ctx.attr.out or ctx.label.name)

    args = ctx.actions.args()
    args.add_all([ctx.file.template, vars_file, out])
    ctx.actions.run(
        executable = ctx.executable._expand,
        arguments = [args],
        inputs = [ctx.file.template, vars_file],
        outputs = [out],
    )

    return [DefaultInfo(
        files = depset([out]),
        runfiles = ctx.runfiles(files = [out]),
    )]

_expand_template = rule(
    implementation = _expand_template_impl,
    attrs = dict(
        vars = attr.string(doc = "Json-encoded mapping from variable to content"),
        vars_file = attr.label(allow_single_file = [".json"]),
        template = attr.label(allow_single_file = [".jinja2"], mandatory = True),
        out = attr.string(),
        _expand = attr.label(
            default = "@@//bazel/build_defs/expand_template",
            executable = True,
            cfg = "exec",
        ),
    ),
)

def expand_template(*, name, vars = None, **kwargs):
    """Expands a jinja2 template.

    Args:
        name: (str) The name of the build rule
        vars: (Mapping[str, json-able]) A mapping from variable names to values.
          Values must be able to be converted to json.
        **kwargs: kwargs to pass to _expand_template"""
    if type(vars) == "dict":
        vars = json.encode(vars)
    elif vars != None:
        fail("vars must be a dictionary mapping from variable name to value")

    _expand_template(
        name = name,
        vars = vars,
        **kwargs
    )
