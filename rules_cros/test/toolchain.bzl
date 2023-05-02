def _transition_impl(settings, attr):
    _ignore = [settings, attr]

    return {
        "linux_x86_64": {
            "//command_line_option:platforms": "//cros/platforms:linux_x86_64"
        },
        "cros_x86_64": {
            "//command_line_option:platforms": "//cros/platforms:cros_x86_64"
        },
    }

full_toolchain_fanout = transition(
    implementation = _transition_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

_QEMU_TEST_TEMPLATE = "//test:qemu_test_template.sh"

def _qemu_test_impl(ctx):
    emu_toolchain = ctx.toolchains["//cros/toolchain/emulation:toolchain_type"]
    emulator_path = emu_toolchain.emuinfo.emulator_path

    ctx.actions.expand_template(
        template = ctx.file._template,
        output = ctx.outputs.executable,
        substitutions = {
            "%EMULATOR%": emulator_path,
            "%PATH%": ctx.files.dep[0].short_path,
        },
    )

    runfiles = ctx.runfiles(ctx.files.dep)
    return [DefaultInfo(runfiles = runfiles)]

qemu_test = rule(
    implementation = _qemu_test_impl,
    attrs = {
        "dep": attr.label(mandatory = True, executable = True, cfg = "target"),
        "_template": attr.label(
            default = Label(_QEMU_TEST_TEMPLATE),
            allow_single_file = True,
        )
    },
    toolchains = ["//cros/toolchain/emulation:toolchain_type"],
    test = True,
)

def _run_tests(test, args):
    return """
{test} {args}
((err+=$?))
""".format(test = test, args = " ".join(args))

def _toolchain_test_impl(ctx):
    script = ""
    symlinks = {}

    for platform in ctx.split_attr.deps:
        for d in ctx.split_attr.deps[platform]:
            executable = d.files_to_run.executable
            # Need to specify symlinks with platform appended for the runfiles,
            # otherwise they will overwrite eachother for each platform.
            test_path = "{}.{}".format(executable.short_path, platform)
            symlinks[test_path] = executable
            args = []
            for f in d.default_runfiles.files.to_list():
                if f != executable:
                    rf_path = "{}.{}".format(f.short_path, platform)
                    args.append(rf_path)
                    symlinks[rf_path] = f

            script = "\n".join(
                [script] +
                [_run_tests(test_path, args)]
            )

    script = "\n".join(
        ["#!/bin/bash"] +
        ["err=0"] +
        [script] +
        ["exit $err"],
    )

    ctx.actions.write(
        output = ctx.outputs.executable,
        content = script,
    )

    runfiles = ctx.runfiles(symlinks = symlinks)
    return [DefaultInfo(runfiles = runfiles)]

toolchain_test = rule(
    implementation = _toolchain_test_impl,
    attrs = {
        "deps": attr.label_list(cfg = full_toolchain_fanout),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
    test = True,
)
