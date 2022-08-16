load("@bazel_skylib//lib:paths.bzl", "paths")

BinaryPackageInfo = provider(
    "Portage binary package info",
    fields = {
        "file": "File of a binary package file (.tbz2)",
        "build_target_deps": "Depset[File] of binary package files (.tbz2)",
        "runtime_deps": "Depset[File] of binary package files (.tbz2)",
    },
)

OverlayInfo = provider(
    "Portage overlay info",
    fields = {
        "squashfs_file": "File of a squashfs image (.squashfs)",
        "mount_path": "String of a path where the overlay is mounted",
    },
)

OverlaySetInfo = provider(
    "Portage overlay set info",
    fields = {
        "overlays": "OverlayInfo[]",
    },
)

SDKInfo = provider(
    "ChromiumOS SDK info",
    fields = {
        "board": "string",
        "squashfs_files": "File[] of squashfs images (.squashfs)",
    },
)

def _workspace_root(label):
    return paths.join("..", label.workspace_name) if label.workspace_name else ""

def _relative_path_in_package(file):
    owner = file.owner
    if owner == None:
        fail("File does not have an associated owner label")
    return paths.relativize(file.short_path, paths.join(_workspace_root(owner), owner.package))

def _format_file_arg(file):
    return "--file=%s=%s" % (_relative_path_in_package(file), file.path)

def _ebuild_impl(ctx):
    src_basename = ctx.file.src.basename.rsplit(".", 1)[0]
    output = ctx.actions.declare_file(src_basename + ".tbz2")
    sdk = ctx.attr._sdk[SDKInfo]

    args = ctx.actions.args()
    args.add_all([
        "--run-in-container=" + ctx.executable._run_in_container.path,
        "--dumb-init=" + ctx.executable._dumb_init.path,
        "--squashfuse=" + ctx.file._squashfuse.path,
        "--ebuild=" + ctx.file.src.path,
        "--category=" + ctx.attr.category,
        "--output=" + output.path,
        "--board=" + sdk.board,
    ])

    direct_inputs = [
        ctx.executable._build_package,
        ctx.executable._run_in_container,
        ctx.executable._dumb_init,
        ctx.file._squashfuse,
        ctx.file.src,
    ]
    transitive_inputs = []

    args.add_all(sdk.squashfs_files, format_each = "--sdk=%s")
    direct_inputs.extend(sdk.squashfs_files)

    for file in ctx.attr.files:
        args.add_all(file.files, map_each = _format_file_arg)
        transitive_inputs.append(file.files)

    for distfile, name in ctx.attr.distfiles.items():
        files = distfile.files.to_list()
        if len(files) != 1:
            fail("cannot refer to multi-file rule in distfiles")
        file = files[0]
        args.add("--distfile=%s=%s" % (name, file.path))
        direct_inputs.append(file)

    overlays = ctx.attr._overlays[OverlaySetInfo].overlays
    for overlay in overlays:
        args.add("--overlay=%s=%s" % (overlay.mount_path, overlay.squashfs_file.path))
        direct_inputs.append(overlay.squashfs_file)

    # TODO: Consider target/host transitions.
    build_target_deps = depset(
        [dep[BinaryPackageInfo].file for dep in ctx.attr.build_target_deps],
        order = "postorder",
    )
    runtime_deps = depset(
        [dep[BinaryPackageInfo].file for dep in ctx.attr.runtime_deps],
        transitive = [dep[BinaryPackageInfo].runtime_deps for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    args.add_all(build_target_deps, format_each = "--install-target=%s")
    transitive_inputs.extend([build_target_deps])

    ctx.actions.run(
        inputs = depset(direct_inputs, transitive = transitive_inputs),
        outputs = [output],
        executable = ctx.executable._build_package,
        arguments = [args],
        mnemonic = "Ebuild",
        progress_message = "Building " + ctx.file.src.basename,
    )
    return [
        DefaultInfo(files = depset([output])),
        BinaryPackageInfo(
            file = output,
            build_target_deps = build_target_deps,
            runtime_deps = runtime_deps,
        ),
    ]

ebuild = rule(
    implementation = _ebuild_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = [".ebuild"],
        ),
        "category": attr.string(
            mandatory = True,
        ),
        "distfiles": attr.label_keyed_string_dict(
            allow_files = True,
        ),
        "build_target_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "runtime_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "files": attr.label_list(
            allow_files = True,
        ),
        "_overlays": attr.label(
            providers = [OverlaySetInfo],
            default = "//config:overlays",
        ),
        "_build_package": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//ebuild/private/cmd/build_package"),
        ),
        "_run_in_container": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//ebuild/private/cmd/run_in_container"),
        ),
        "_squashfuse": attr.label(
            allow_single_file = True,
            executable = True,
            cfg = "exec",
            default = Label("//third_party/prebuilts/host:squashfuse"),
        ),
        "_dumb_init": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("@dumb_init//file"),
        ),
        "_sdk": attr.label(
            providers = [SDKInfo],
            default = Label("//sdk"),
        ),
    },
)

def _binary_package_impl(ctx):
    src = ctx.file.src

    # TODO: Consider target/host transitions.
    runtime_deps = depset(
        [dep[BinaryPackageInfo].file for dep in ctx.attr.runtime_deps],
        transitive = [dep[BinaryPackageInfo].runtime_deps for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    return [
        DefaultInfo(files = depset([src])),
        BinaryPackageInfo(
            file = src,
            build_target_deps = depset(),
            runtime_deps = runtime_deps,
        ),
    ]

binary_package = rule(
    implementation = _binary_package_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = [".tbz2"],
        ),
        "runtime_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
    },
)

def _format_create_squashfs_arg(file):
    return "%s:%s" % (_relative_path_in_package(file), file.path)

def _create_squashfs_action(ctx, out, exe, files):
    args = ctx.actions.args()
    args.add_all(files, map_each = _format_create_squashfs_arg)
    args.set_param_file_format("multiline")
    args.use_param_file("--specs-from=%s", use_always = True)

    ctx.actions.run(
        inputs = [exe] + files,
        outputs = [out],
        executable = exe.path,
        arguments = ["--output=" + out.path, args],
    )

def _overlay_impl(ctx):
    out = ctx.actions.declare_file(ctx.attr.name + ".squashfs")

    _create_squashfs_action(ctx, out, ctx.executable._create_squashfs, ctx.files.srcs)

    return [
        DefaultInfo(files = depset([out])),
        OverlayInfo(squashfs_file = out, mount_path = ctx.attr.mount_path),
    ]

overlay = rule(
    implementation = _overlay_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
        "mount_path": attr.string(
            mandatory = True,
        ),
        "_create_squashfs": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//ebuild/private/cmd/create_squashfs"),
        ),
    },
)

def _overlay_set_impl(ctx):
    return [
        OverlaySetInfo(
            overlays = [overlay[OverlayInfo] for overlay in ctx.attr.overlays],
        ),
    ]

overlay_set = rule(
    implementation = _overlay_set_impl,
    attrs = {
        "overlays": attr.label_list(
            providers = [OverlayInfo],
        ),
    },
)

def _sdk_impl(ctx):
    base_squashfs_output = ctx.actions.declare_file(ctx.attr.name + "_base.squashfs")
    pkgs_squashfs_output = ctx.actions.declare_file(ctx.attr.name + "_pkgs.squashfs")
    host_installs = [label[BinaryPackageInfo].file for label in ctx.attr.host_deps]
    target_installs = [label[BinaryPackageInfo].file for label in ctx.attr.target_deps]

    ctx.actions.run_shell(
        outputs = [base_squashfs_output],
        inputs = [ctx.file.src],
        # TODO: Avoid -all-root.
        command = "xzcat \"$1\" | mksquashfs - \"$2\" -tar -all-time 0 -all-root",
        arguments = [ctx.file.src.path, base_squashfs_output.path],
        progress_message = "Converting %{input} to squashfs",
    )

    args = ctx.actions.args()
    args.add_all([
        "--run-in-container=" + ctx.executable._run_in_container.path,
        "--dumb-init=" + ctx.executable._dumb_init.path,
        "--squashfuse=" + ctx.file._squashfuse.path,
        "--input-squashfs=" + base_squashfs_output.path,
        "--output-squashfs=" + pkgs_squashfs_output.path,
        "--board=" + ctx.attr.board,
    ])
    args.add_all(host_installs, format_each = "--install-host=%s")
    args.add_all(target_installs, format_each = "--install-target=%s")

    inputs = [
        ctx.executable._build_sdk,
        ctx.executable._run_in_container,
        ctx.file._squashfuse,
        ctx.executable._dumb_init,
        base_squashfs_output,
    ] + host_installs + target_installs

    for overlay in ctx.attr._overlays[OverlaySetInfo].overlays:
        args.add("--overlay=%s=%s" % (overlay.mount_path, overlay.squashfs_file.path))
        inputs.append(overlay.squashfs_file)

    ctx.actions.run(
        inputs = inputs,
        outputs = [pkgs_squashfs_output],
        executable = ctx.executable._build_sdk,
        arguments = [args],
        mnemonic = "Sdk",
        progress_message = "Building SDK",
    )

    outputs = [pkgs_squashfs_output, base_squashfs_output]
    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(board = ctx.attr.board, squashfs_files = outputs),
    ]

sdk = rule(
    implementation = _sdk_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "board": attr.string(
            mandatory = True,
        ),
        "host_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "target_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "_build_sdk": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//ebuild/private/cmd/build_sdk"),
        ),
        "_run_in_container": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//ebuild/private/cmd/run_in_container"),
        ),
        "_squashfuse": attr.label(
            allow_single_file = True,
            executable = True,
            cfg = "exec",
            default = Label("//third_party/prebuilts/host:squashfuse"),
        ),
        "_dumb_init": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("@dumb_init//file"),
        ),
        "_overlays": attr.label(
            providers = [OverlaySetInfo],
            default = "//config:overlays",
        ),
    },
)
