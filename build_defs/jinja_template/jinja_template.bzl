# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Rules for jinja template files."""

visibility("public")

JinjaTemplateInfo = provider(
    doc = "Metadata about jinja2 templates",
    fields = {
        "deps": "(depset[JinjaTemplateInfo]): Templates transitively included by the root template",
        "include": "(depset[File]): Files transitively included by the root template",
        "root": "(File): The root template file.",
    },
)

def _jinja_template_impl(ctx):
    templates = [attr[JinjaTemplateInfo] for attr in ctx.attr.deps]

    info = JinjaTemplateInfo(
        root = ctx.file.src,
        deps = depset(templates, transitive = [
            template.deps
            for template in templates
        ]),
        include = depset([ctx.file.src], transitive = [
            template.include
            for template in templates
        ]),
    )

    return [
        info,
        DefaultInfo(
            files = depset([info.root]),
            runfiles = ctx.runfiles(transitive_files = info.include),
        ),
    ]

jinja_template = rule(
    implementation = _jinja_template_impl,
    attrs = dict(
        src = attr.label(allow_single_file = [".jinja2"], mandatory = True),
        deps = attr.label_list(providers = [JinjaTemplateInfo]),
    ),
    provides = [JinjaTemplateInfo],
)
