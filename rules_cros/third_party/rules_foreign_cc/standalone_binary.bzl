load("@rules_foreign_cc//foreign_cc:providers.bzl", "ForeignCcDepsInfo")
load("//rules_cros/toolchains/bash:defs.bzl", "BASH_RUNFILES_ATTRS", "runfiles_path", "wrap_binary_with_args")

_ENV_VARS = "export LD_LIBRARY_PATH={extra_libraries}:${{LD_LIBRARY_PATH:-}}"

def _foreign_cc_standalone_binary_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name)

    artifacts = ctx.attr.src[ForeignCcDepsInfo].artifacts.to_list()

    exe_files = ctx.attr.src[DefaultInfo].files.to_list()
    exe = None
    for f in exe_files:
        if f.basename == out.basename:
            exe = f

    if exe == None:
        fail("Unable to find binary '{bin}' in '{exe_files}'".format(
            bin = ctx.attr.bin_file,
            exe_files = exe_files,
        ))

    lib_paths = []
    deps = []
    for artifact in artifacts:
        deps.append(artifact.gen_dir)
        lib_paths.append("$(rlocation {artifact_dir}/{lib_dir})".format(
            artifact_dir = runfiles_path(ctx, artifact.gen_dir),
            lib_dir = artifact.lib_dir_name,
        ))

    return wrap_binary_with_args(
        ctx,
        out = out,
        binary = exe,
        args = [],
        content_prefix = _ENV_VARS.format(
            extra_libraries = ":".join(lib_paths),
        ),
        runfiles = ctx.runfiles(files = deps),
    )

foreign_cc_standalone_binary = rule(
    _foreign_cc_standalone_binary_impl,
    attrs = dict(
        src = attr.label(mandatory = True, providers = [ForeignCcDepsInfo]),
        **BASH_RUNFILES_ATTRS
    ),
    doc = "Creates a binary that uses shared libraries provided by foreign_cc.",
    executable = True,
)
