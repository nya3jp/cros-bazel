# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "BinaryPackageInfo")

visibility("public")

def _extract_interface_impl(ctx):
    files = ctx.attr.files
    binpkg = ctx.attr.pkg[BinaryPackageInfo].file
    args = ctx.actions.args()
    if ctx.attr.patch_elf:
        args.add("--patch-elf")
    args.add("--binpkg", binpkg)
    outs = []
    executable = None
    for k, v in files.items():
        out = ctx.actions.declare_file(v)
        args.add("--output-file=%s=%s" % (k, out.path))
        outs.append(out)
        if k == ctx.attr.executable:
            executable = out

    if ctx.attr.executable and ctx.attr.executable not in files:
        fail("Unable to find executable {} within {}".format(ctx.attr.executable, repr(sorted(files))))

    ctx.actions.run(
        outputs = outs,
        inputs = [binpkg],
        executable = ctx.executable._extract_interface,
        arguments = [args],
    )
    return DefaultInfo(
        files = depset(outs),
        executable = executable,
    )

_EXTRACT_ATTRS = dict(
    _extract_interface = attr.label(
        executable = True,
        default = "//bazel/portage/bin/extract_interface",
        cfg = "exec",
    ),
    pkg = attr.label(mandatory = True, providers = [BinaryPackageInfo]),
    patch_elf = attr.bool(default = True),
    files = attr.string_dict(mandatory = True),
    executable = attr.string(),
)

_extract_interface_noexe = rule(
    implementation = _extract_interface_impl,
    attrs = _EXTRACT_ATTRS,
)

_extract_interface_exe = rule(
    implementation = _extract_interface_impl,
    attrs = _EXTRACT_ATTRS,
    executable = True,
)

def extract_interface(files, executable = False, **kwargs):
    """Extracts files from a tbz2 file to one usable by bazel.

    Args:
      files: (List[str]|Dict[str, str]) A map from path in the tarball to the
        destination path. (eg: {"/bin/foo": "foo"}). If a list is provided, it
        is transformed into a map (eg. ["/bin/foo"] -> {"/bin/foo": "bin/foo"}).
      executable: (bool|string) If False, mark as not executable.
        If True, set to the *only* entry in files.
        If a string, it must correspond to a key in files.
    """
    rule = _extract_interface_exe if executable else _extract_interface_noexe
    if executable == True:
        if len(files) == 1:
            executable = list(files)[0]
        else:
            fail("executable = True is only supported when there is a single entry in files")

    # The rule takes in Option[str], so it doesn't like False.
    if not executable:
        executable = None

    if type(files) == type([]):
        files = {file: file.lstrip("/") for file in files}

    for file in files:
        if not file.startswith("/"):
            fail("All files must be absolute paths")

    rule(
        files = files,
        executable = executable,
        **kwargs
    )
