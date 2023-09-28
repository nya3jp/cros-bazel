# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "BinaryPackageInfo", "SDKInfo")
load("//bazel/portage/build_defs:ebuild.bzl", "generate_ebuild_validation_action")

HashTracerInfo = provider(
    """
    Because transitive validations created by aspects are not run by bazel, we
    need to attach them all to a real target. This provider contains the files
    that will be included in the top-level target's `_validation` output group.
    """,
    fields = {
        "files": """
            Depset[File]: The validation output files.
        """,
    },
)

def _generate_hash_action(ctx, files):
    output = ctx.actions.declare_file(ctx.rule.attr.name + ".hash")
    args = ctx.actions.args()
    args.add(output)
    args.add_all(files, expand_directories = False)

    ctx.actions.run_shell(
        outputs = [output],
        inputs = files,
        arguments = [args],
        mnemonic = "HashTracer",
        execution_requirements = {
            # We don't cache this because we want to increase the chance of
            # the hash being printed.
            "no-cache": "",
            # Disable the sandbox to avoid creating a symlink forest.
            # The symlink forest will mess up the `find` command.
            "no-sandbox": "",
        },
        command = """set -eu -o pipefail
            out="$1"
            shift
            for FILE in "$@"; do
                if [[ -d "${FILE}" ]]; then
                    pushd "${FILE}" >/dev/null
                    HASH="$(find . -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -f1 -d ' ')"
                    popd >/dev/null
                else
                    HASH="$(sha256sum "${FILE}" | cut -f1 -d ' ')"
                fi
                SIZE="$(du -hs "${FILE}" | cut -f1 -d$'\t')"
                echo "* Hash Tracer: ${FILE} -> ${HASH} (${SIZE})" | tee -a "$out"
            done
        """,
    )

    return output

def _processes_rule(rule):
    """
    Iterates through all the attributes of a rule and extracts all of its
    dependencies' `HashTracerInfo`.
    """
    depsets = []

    def _add(val):
        if type(val) in [
            "bool",
            "int",
            "builtin_function_or_method",
            "string",
            "NoneType",
            "Label",
            "License",
        ]:
            return
        elif type(val) == "Target":
            if HashTracerInfo in val:
                depsets.append(val[HashTracerInfo].files)
        else:
            fail("Unknown type: %s" % (type(val)))

    attrs = dir(rule.attr)
    for key in attrs:
        val = getattr(rule.attr, key)

        # We can't use recursion, so special case these.
        if type(val) == "list":
            for item in val:
                _add(item)
        elif type(val) == "dict":
            # label_keyed_string_dict has the labels as keys
            for key in val.keys():
                _add(key)
        else:
            _add(val)

    return depsets

def _hash_tracer_impl(target, ctx):
    direct_outputs = []
    depsets = []

    # We consider these terminal nodes since we just want to print out the
    # output hash.
    if ctx.rule.kind in [
        "pkg_tar_impl",
        "go_binary",
        "rust_binary",
        "cc_binary",
        "cc_library",
        "py_library",
    ]:
        direct_outputs.append(_generate_hash_action(ctx, target.files))

    elif ctx.rule.kind in ["build_sdk", "sdk_update", "sdk_install_deps", "sdk_from_archive"]:
        layers = target[SDKInfo].layers

        # We only want to hash the layer that the rule created.
        last_layer = layers[-1]

        direct_outputs.append(_generate_hash_action(ctx, [last_layer]))
        depsets.extend(_processes_rule(ctx.rule))

    elif ctx.rule.kind in ["ebuild"]:
        binpkg = target[BinaryPackageInfo].file

        files = [binpkg]

        # Only one rule or aspect can currently generate the _validation output group.
        # So we have this very ebuild specific validator defined here.
        # See https://github.com/bazelbuild/bazel/issues/19624
        # Once this bug is fixed, we can move the validator back to ebuild.bzl.
        direct_outputs.append(generate_ebuild_validation_action(ctx, binpkg))

        direct_outputs.append(_generate_hash_action(ctx, files))

        depsets.extend(_processes_rule(ctx.rule))
    else:
        # For all the intermediary nodes we just propagate the dependencies.
        depsets.extend(_processes_rule(ctx.rule))

    return [HashTracerInfo(files = depset(direct_outputs, transitive = depsets))]

hash_tracer = aspect(
    implementation = _hash_tracer_impl,
    doc = "Prints out the sha256 of all dependent tar, go_binary, and rust_binary targets, etc.",
    attr_aspects = ["*"],
)

def _hash_tracer_validator_impl(target, ctx):
    validations = []
    depsets = []

    if HashTracerInfo in target:
        files = target[HashTracerInfo].files
        return [
            OutputGroupInfo(_validation = files),
        ]
    else:
        return []

hash_tracer_validator = aspect(
    implementation = _hash_tracer_validator_impl,
    doc = """
    Attaches the HashTracerInfo to the top-level build targets as an validation
    action.

    This is necessary because aspects can only attach a validator to a top level
    action. See https://github.com/bazelbuild/bazel/issues/19636.
    """,
    requires = [hash_tracer],
)
