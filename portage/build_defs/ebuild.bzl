# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")
load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")
load("//bazel/bash:defs.bzl", "BASH_RUNFILES_ATTR", "wrap_binary_with_args")
load("//bazel/portage/build_defs:common.bzl", "BashrcInfo", "BinaryPackageInfo", "BinaryPackageSetInfo", "EbuildLibraryInfo", "ExtraSourcesInfo", "OverlayInfo", "OverlaySetInfo", "SDKInfo", "compute_file_arg", "relative_path_in_package", "single_binary_package_set_info")
load("//bazel/portage/build_defs:interface_lib.bzl", "add_interface_library_args", "generate_interface_libraries")
load("//bazel/portage/build_defs:package_contents.bzl", "generate_contents")
load("//bazel/transitions:primordial.bzl", "primordial_transition")

_CCACHE_DIR_LABEL = "//bazel/portage:ccache_dir"

# The stage1 SDK will need to be built with ebuild_primordial.
# After that, they can use the ebuild rule.
# This ensures that we don't build the stage1 targets twice.
def maybe_primordial_rule(attrs, **kwargs):
    return (
        rule(attrs = attrs, **kwargs),
        rule(cfg = primordial_transition, attrs = dict(
            _allowlist_function_transition = attr.label(
                default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
            ),
            **attrs
        ), **kwargs),
    )

# Attributes common to the `ebuild`/`ebuild_debug`/`ebuild_test` rule.
_EBUILD_COMMON_ATTRS = dict(
    ebuild = attr.label(
        mandatory = True,
        allow_single_file = [".ebuild"],
    ),
    overlay = attr.label(
        mandatory = True,
        providers = [OverlayInfo],
        doc = """
        The overlay this package belongs to.
        """,
    ),
    eclasses = attr.label_list(
        providers = [PackageArtifactInfo],
        doc = """
        The eclasses this package inherits from (including transitive ones).
        """,
    ),
    category = attr.string(
        mandatory = True,
        doc = """
        The category of this package.
        """,
    ),
    package_name = attr.string(
        mandatory = True,
        doc = """
        The name of this package.
        """,
    ),
    version = attr.string(
        mandatory = True,
        doc = """
        The version of this package.
        """,
    ),
    slot = attr.string(
        mandatory = True,
        doc = """
        The slot the package is installed to.
        """,
    ),
    distfiles = attr.label_keyed_string_dict(
        allow_files = True,
    ),
    srcs = attr.label_list(
        doc = "src files used by the ebuild",
        allow_files = True,
    ),
    cache_srcs = attr.label_list(
        doc = "Cache files used by the ebuild",
        allow_files = True,
    ),
    git_trees = attr.label_list(
        doc = """
        The git tree objects listed in the CROS_WORKON_TREE variable.
        """,
        allow_empty = True,
        allow_files = True,
    ),
    use_flags = attr.string_list(
        allow_empty = True,
        doc = """
        The USE flags used to build the package.
        """,
    ),
    inject_use_flags = attr.bool(
        default = False,
        doc = """
        Inject the USE flags into the container as opposed to letting portage
        compute them.
        """,
    ),
    files = attr.label_list(
        allow_files = True,
    ),
    extra_srcs = attr.label_list(
        doc = """
        Extra source codes provided to ephemeral containers.
        """,
        providers = [ExtraSourcesInfo],
    ),
    runtime_deps = attr.label_list(
        providers = [BinaryPackageInfo, BinaryPackageSetInfo],
    ),
    shared_lib_deps = attr.label_list(
        doc = """
        The shared libraries this target will link against.
        """,
        providers = [EbuildLibraryInfo],
    ),
    allow_network_access = attr.bool(
        default = False,
        doc = """
        Allows the build process to access the network. This should be True only
        when the package explicitly requests network access, e.g.
        RESTRICT=network-sandbox.
        """,
    ),
    board = attr.string(
        doc = """
        The target board name to build the package for. If unset, then the host
        will be targeted.
        """,
    ),
    incremental_cache_marker = attr.label(
        allow_single_file = True,
        doc = """
        Marker file for the directory for incremental build caches.
        Cache directories are created as siblings of the marker file.

        If set, mounts a persistent directory for the ebuild to reuse
        build artifacts across multiple Bazel orchestrated ebuilds invocations.
        This allows incremental builds for the internal build system
        such as make, ninja, etc.

        If unset, no cache is persisted between ebuild invocations.
        """,
    ),
    sdk = attr.label(
        providers = [SDKInfo],
        mandatory = True,
    ),
    overlays = attr.label(
        providers = [OverlaySetInfo],
        mandatory = True,
    ),
    portage_config = attr.label_list(
        providers = [PackageArtifactInfo],
        doc = """
        The portage config for the host and optionally the target board. This
        should at minimum contain a make.conf file.
        """,
        mandatory = True,
    ),
    bashrcs = attr.label_list(
        providers = [BashrcInfo],
        doc = """
        The bashrc files to execute for the package.
        """,
    ),
    _action_wrapper = attr.label(
        executable = True,
        cfg = "exec",
        default = Label("//bazel/portage/bin/action_wrapper"),
    ),
    _build_package = attr.label(
        executable = True,
        cfg = "exec",
        default = Label("//bazel/portage/bin/build_package"),
    ),
    _remoteexec_info = attr.label(
        allow_single_file = True,
        default = Label("@remoteexec_info//:remoteexec_info"),
    ),
    _remotetool = attr.label(
        default = Label("@files//:remotetool"),
        executable = True,
        cfg = "exec",
    ),
    ccache = attr.bool(
        doc = """
        Enable ccache for this ebuild.
        """,
        mandatory = True,
    ),
    _ccache_dir = attr.label(
        default = Label(_CCACHE_DIR_LABEL),
        providers = [BuildSettingInfo],
    ),
    supports_remoteexec = attr.bool(
        default = False,
        doc = """
        Indicates whether this ebuild supports building with reclient or not.
        """,
    ),
)

def _bashrc_to_path(bashrc):
    return bashrc[BashrcInfo].path

def _ccache_settings(ctx):
    """Helper to get ccache settings.

    Returns a tuple of (ccache, ccache_dir), where:
        ccache: Whether ccache is enabled.
        ccache_dir: Directory to store the ccache.
    """
    ccache = ctx.attr.ccache
    ccache_dir = ctx.attr._ccache_dir[BuildSettingInfo].value

    if ccache and not ccache_dir:
        fail(
            "ccache is enabled for %s but %s not set" % (ctx.label, _CCACHE_DIR_LABEL),
        )
    if ccache_dir and not ccache_dir.startswith("/"):
        fail("%s=%r is not an absolute path" % (_CCACHE_DIR_LABEL, ccache_dir))

    return ccache, ccache_dir

# TODO(b/269558613): Fix all call sites to always use runfile paths and delete `for_test`.
def _compute_build_package_args(ctx, output_file, use_runfiles):
    """
    Computes the arguments to run build_package.

    These arguments should be passed to action_wrapper, not build_package. They
    contain the path to the build_package executable itself, and also may start
    with options to action_wrapper (e.g. --privileged).

    This function can be called only from `ebuild`, `ebuild_debug`, and
    `ebuild_test`. Particularly, the current rule must include
    _EBUILD_COMMON_ATTRS in its attribute definition.

    Args:
        ctx: ctx: A context objected passed to the rule implementation.
        output_file: Optional[File]: A file where an output binary package is
            is saved. If None, a binary package is not saved.
        use_runfiles: bool: Whether to refer to runfiles paths instead of files.
            See compute_file_arg for details.

    Returns:
        struct where:
            args: Args: Arguments to pass to action_wrapper.
            inputs: Depset[File]: Inputs to action_wrapper.
    """
    args = ctx.actions.args()
    direct_inputs = []
    transitive_inputs = []

    # Path to build_package

    args.add(compute_file_arg(ctx.executable._build_package, use_runfiles))

    # Basic arguments
    if ctx.attr.board:
        args.add("--board=" + ctx.attr.board)
    if output_file:
        args.add("--output", output_file)

    # We extract the <category>/<package>/<ebuild> from the file path.
    relative_ebuild_path = "/".join(ctx.file.ebuild.path.rsplit("/", 3)[1:4])
    ebuild_inside_path = "%s/%s" % (ctx.attr.overlay[OverlayInfo].path, relative_ebuild_path)

    # --ebuild
    args.add_joined(
        "--ebuild",
        [
            ebuild_inside_path,
            compute_file_arg(ctx.file.ebuild, use_runfiles),
        ],
        join_with = "=",
    )
    direct_inputs.append(ctx.file.ebuild)

    # --file
    for files in ctx.attr.files:
        for file in files.files.to_list():
            args.add_joined(
                "--file",
                [
                    relative_path_in_package(file),
                    compute_file_arg(file, use_runfiles),
                ],
                join_with = "=",
            )
        transitive_inputs.append(files.files)

    # --distfile
    for distfile, distfile_name in ctx.attr.distfiles.items():
        files = distfile.files.to_list()
        if len(files) != 1:
            fail("cannot refer to multi-file rule in distfiles")
        file = files[0]
        args.add_joined(
            "--distfile",
            [distfile_name, compute_file_arg(file, use_runfiles)],
            join_with = "=",
        )
        direct_inputs.append(file)

    # --layer for SDK, overlays and eclasses
    sdk = ctx.attr.sdk[SDKInfo]
    overlays = ctx.attr.overlays[OverlaySetInfo]
    layer_inputs = (
        sdk.layers +
        overlays.layers +
        ctx.files.eclasses +
        ctx.files.portage_config +
        ctx.files.bashrcs
    )
    args.add_all(
        [compute_file_arg(f, use_runfiles) for f in layer_inputs],
        before_each = "--layer",
        expand_directories = False,
    )
    direct_inputs.extend(layer_inputs)

    # --layer for source code
    for file in ctx.files.srcs:
        args.add("--layer", compute_file_arg(file, use_runfiles))
        direct_inputs.append(file)

    # --layer for extra source code
    for extra_src in ctx.attr.extra_srcs:
        tar = extra_src[ExtraSourcesInfo].tar
        args.add("--layer", compute_file_arg(tar, use_runfiles))
        direct_inputs.append(tar)

    # --layer for cache files
    # NOTE: We're not adding this file to transitive_inputs because the contents of cache files shouldn't affect the build output.
    for file in ctx.files.cache_srcs:
        args.add("--layer", compute_file_arg(file, use_runfiles))

    args.add_all(
        [compute_file_arg(f, use_runfiles) for f in ctx.files.git_trees],
        before_each = "--git-tree",
    )
    direct_inputs.extend(ctx.files.git_trees)

    # --allow-network-access
    if ctx.attr.allow_network_access:
        args.add("--allow-network-access")

    # --incremental-cache-dir
    if ctx.file.incremental_cache_marker:
        # Use the cache marker file to resolve the cache directory.
        # The caches are not exposed to Bazel as inputs as they are
        # volatile and are not supposed to use as action cache keys.
        # NOTE: build_package assumes the directory is writable.
        # TODO(b/308409815): Make this more robust.
        # See https://crrev.com/c/4989046/comment/196dbd8c_c044dfc0/.
        cache_marker_path_or_file = compute_file_arg(ctx.file.incremental_cache_marker, use_runfiles)
        if type(cache_marker_path_or_file) != "string":
            cache_marker_path = cache_marker_path_or_file.path
        else:
            cache_marker_path = cache_marker_path_or_file
        caches_dir = paths.dirname(cache_marker_path)
        args.add("--incremental-cache-dir=%s/portage" % caches_dir)

    # --ccache, --ccache-dir
    ccache, ccache_dir = _ccache_settings(ctx)
    if ccache:
        args.add("--ccache")
        args.add(ccache_dir, format = "--ccache-dir=%s")

    # --use-flags
    if ctx.attr.inject_use_flags:
        args.add_joined("--use-flags", ctx.attr.use_flags, join_with = ",")

    if ctx.attr.supports_remoteexec:
        args.add_all([
            # NOTE: We're not adding this file to transitive_inputs because the contents of remoteexec_info shouldn't affect the build output.
            "--remoteexec-info",
            ctx.file._remoteexec_info,
        ])

    args.add_all(ctx.attr.bashrcs, before_each = "--bashrc", map_each = _bashrc_to_path)

    # Consume interface libraries.
    interface_library_inputs = add_interface_library_args(
        input_targets = ctx.attr.shared_lib_deps,
        args = args,
        use_runfiles = use_runfiles,
    )
    transitive_inputs.append(interface_library_inputs)

    # Include runfiles in the inputs.
    transitive_inputs.append(ctx.attr._action_wrapper[DefaultInfo].default_runfiles.files)
    transitive_inputs.append(ctx.attr._build_package[DefaultInfo].default_runfiles.files)

    inputs = depset(direct_inputs, transitive = transitive_inputs)
    return struct(
        args = args,
        inputs = inputs,
    )

def _download_prebuilt(ctx, prebuilt, output_binary_package_file):
    args = ctx.actions.args()
    if prebuilt.startswith("http://") or prebuilt.startswith("https://"):
        executable = "wget"
        args.add_all([prebuilt, "-O", output_binary_package_file])
    elif prebuilt.startswith("gs://"):
        executable = Label("@chromite//:src").workspace_root + "/bin/gsutil"
        args.add_all(["cp", prebuilt, output_binary_package_file])
    elif prebuilt.startswith("cas://"):
        # Format: cas://<instance>/<sha256>/<size>
        prebuilt = prebuilt[6:]
        instance, checksum, size = prebuilt.rsplit("/", 2)
        executable = ctx.executable._remotetool
        args.add_all([
            "--service=remotebuildexecution.googleapis.com:443",
            "--use_application_default_credentials",
            "--operation=download_blob",
            "--digest",
            "{checksum}/{size}".format(checksum = checksum, size = size),
            "--instance",
            instance,
            "--path",
            output_binary_package_file,
        ])

    else:
        executable = "cp"
        args.add_all([prebuilt, output_binary_package_file])

    ctx.actions.run(
        inputs = [],
        outputs = [output_binary_package_file],
        executable = executable,
        arguments = [args],
        execution_requirements = {
            "no-remote": "",
            "no-sandbox": "",
            "requires-network": "",
        },
        progress_message = "Downloading %s" % prebuilt,
    )

def _get_basename(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]
    if ctx.attr.suffix:
        src_basename += ctx.attr.suffix

    return src_basename

def _generate_ebuild_validation_action(ctx, binpkg):
    src_basename = _get_basename(ctx)

    validation_file = ctx.actions.declare_file(src_basename + ".validation")

    args = ctx.actions.args()
    args.add_all([
        "validate-package",
        "--touch",
        validation_file,
        "--package",
        binpkg,
    ])
    args.add_joined("--use-flags", ctx.attr.use_flags, join_with = ",", omit_if_empty = False)

    ctx.actions.run(
        inputs = depset([binpkg]),
        outputs = [validation_file],
        executable = ctx.executable._xpaktool,
        arguments = [args],
        mnemonic = "EbuildValidation",
        progress_message = "Validating %{label}",
    )

    return validation_file

def _ebuild_compare_package(ctx, name, packages):
    if len(packages) != 2:
        fail("Expected two packages, got %d" % (len(packages)))

    src_basename = _get_basename(ctx)

    log_file = ctx.actions.declare_file("%s.%s.log" % (src_basename, name))

    args = ctx.actions.args()
    args.add("--log", log_file)
    args.add(compute_file_arg(ctx.executable._xpaktool, use_runfiles = False))
    args.add("compare-packages")
    args.add_all(packages)

    ctx.actions.run(
        inputs = packages,
        outputs = [log_file],
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._xpaktool],
        arguments = [args],
        mnemonic = "EbuildComparePackage",
    )

    return log_file

def _ebuild_impl(ctx):
    src_basename = _get_basename(ctx)

    # Declare outputs.
    output_binary_package_file = ctx.actions.declare_file(
        src_basename + ".partial.tbz2",
    )
    output_log_file = ctx.actions.declare_file(src_basename + ".log")
    output_profile_file = ctx.actions.declare_file(
        src_basename + ".profile.json",
    )

    # Define the main action.
    prebuilt = ctx.attr.prebuilt[BuildSettingInfo].value
    if prebuilt:
        _download_prebuilt(ctx, prebuilt, output_binary_package_file)
        ctx.actions.write(output_log_file, "Downloaded from %s\n" % prebuilt)
        ctx.actions.write(output_profile_file, "[]")
    else:
        # Compute arguments and inputs to run build_package.
        build_package_args = _compute_build_package_args(
            ctx,
            output_file = output_binary_package_file,
            use_runfiles = False,
        )

        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        }
        if ctx.attr.supports_remoteexec:
            # Do not execute remotely when the underlying build is executing remote jobs.
            execution_requirements["no-remote-exec"] = ""

        action_wrapper_args = ctx.actions.args()
        action_wrapper_args.add_all([
            "--banner",
            "Building %s" % ctx.label,
            "--log",
            output_log_file,
            "--profile",
            output_profile_file,
        ])
        ctx.actions.run(
            inputs = build_package_args.inputs,
            outputs = [
                output_binary_package_file,
                output_log_file,
                output_profile_file,
            ],
            executable = ctx.executable._action_wrapper,
            tools = [ctx.executable._build_package],
            arguments = [action_wrapper_args, build_package_args.args],
            execution_requirements = execution_requirements,
            mnemonic = "Ebuild",
            progress_message = "Building %{label}",
        )

    # Generate contents directories.
    contents = generate_contents(
        ctx = ctx,
        binary_package = output_binary_package_file,
        # We use a per-ebuild target unique identifier as the prefix. This
        # allows us to erase the version number of the ebuild in the content
        # layer paths. This way if an ebuild is upreved, it doesn't necessarily
        # cache bust all its reverse dependencies.
        output_prefix = str(ctx.attr.index),
        board = ctx.attr.board,
        executable_action_wrapper = ctx.executable._action_wrapper,
        executable_extract_package = ctx.executable._extract_package,
    )

    # Generate interface libraries.
    interface_library_outputs, interface_library_providers = generate_interface_libraries(
        ctx = ctx,
        input_binary_package_file = output_binary_package_file,
        output_base_dir = src_basename,
        headers = ctx.attr.headers,
        pkg_configs = ctx.attr.pkg_configs,
        shared_libs = ctx.attr.shared_libs,
        static_libs = ctx.attr.static_libs,
        extract_interface_executable = ctx.executable._extract_interface,
        action_wrapper_executable = ctx.executable._action_wrapper,
    )

    metadata = ctx.actions.declare_file(ctx.label.name + "_metadata.json")
    gen_metadata_args = ctx.actions.args()
    gen_metadata_args.add(str(ctx.label))
    gen_metadata_args.add(output_binary_package_file)
    gen_metadata_args.add(metadata)
    ctx.actions.run(
        executable = ctx.executable._gen_metadata,
        arguments = [gen_metadata_args],
        inputs = [output_binary_package_file],
        outputs = [metadata],
        execution_requirements = {
            # Disable remote execution, since it's cheaper to calculate the
            # checksum than it is to transfer the file to a remote builder.
            "no-remote-exec": "",
        },
    )

    # Compute provider data.
    package_info = BinaryPackageInfo(
        partial = output_binary_package_file,
        metadata = metadata,
        contents = contents,
        category = ctx.attr.category,
        package_name = ctx.attr.package_name,
        version = ctx.attr.version,
        slot = ctx.attr.slot,
        direct_runtime_deps = tuple([
            target[BinaryPackageInfo].partial
            for target in ctx.attr.runtime_deps
        ]),
    )

    package_set_info = single_binary_package_set_info(
        package_info,
        [
            target[BinaryPackageSetInfo]
            for target in ctx.attr.runtime_deps
        ],
    )

    validation_files = [
        _generate_ebuild_validation_action(ctx, output_binary_package_file),
    ]

    if ctx.attr.portage_profile_test_package:
        validation_files.append(
            _ebuild_compare_package(
                ctx,
                "portage-profile-test",
                [
                    ctx.attr.portage_profile_test_package[BinaryPackageInfo].partial,
                    output_binary_package_file,
                ],
            ),
        )

    return [
        DefaultInfo(files = depset(
            [output_binary_package_file] +
            interface_library_outputs,
        )),
        OutputGroupInfo(
            logs = depset([output_log_file]),
            traces = depset([output_profile_file]),
            _validation = depset(validation_files),
        ),
        package_info,
        package_set_info,
    ] + interface_library_providers

ebuild, ebuild_primordial = maybe_primordial_rule(
    implementation = _ebuild_impl,
    doc = "Builds a Portage binary package from an ebuild file.",
    attrs = dict(
        headers = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains the public C headers
            exported by this library.
            """,
        ),
        pkg_configs = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains the pkg-config
            (man 1 pkg-config) `pc` files exported by this package.
            The `pc` is used to look up the CFLAGS and LDFLAGS required to link
            to the library.
            """,
        ),
        shared_libs = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains shared object libraries.
            """,
        ),
        static_libs = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains static libraries.
            """,
        ),
        suffix = attr.string(
            doc = """
            Suffix to add to the output file. i.e., libcxx-17.0-r15<suffix>.tbz2
            """,
        ),
        index = attr.int(
            doc = """
            This index is used when generating the "installed" contents layer.
            It must be unique for each ebuild target in this package.
            """,
            mandatory = True,
        ),
        prebuilt = attr.label(providers = [BuildSettingInfo]),
        portage_profile_test_package = attr.label(
            doc = """
            A package built using the standard portage profile configuration.

            Setting this field will add a validator that compares this package
            to the one using standard portage profiles. This is useful to
            validate that alchemist's compiled profiles are valid.
            """,
            providers = [BinaryPackageInfo],
        ),
        _extract_package = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/extract_package"),
        ),
        _extract_interface = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/extract_interface"),
        ),
        _xpaktool = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/xpaktool"),
        ),
        _gen_metadata = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/metadata:gen_metadata"),
        ),
        **_EBUILD_COMMON_ATTRS
    ),
)

_DEBUG_SCRIPT = """
# Arguments passed in during build time are passed in relative to the execroot,
# which means all files passed in are relative paths starting with bazel-out/
# Thus, we cd to the directory in our working directory containing a bazel-out.

wd="$(pwd)"
cd "${wd%%/bazel-out/*}"

# We want to use the same tmpdir as the ebuild action. Otherwise we default to
# /tmp which might be tmpfs.
TMPDIR="$(dirname "${RUNFILES_MANIFEST_FILE}")"
export TMPDIR

# The runfiles manifest file contains relative paths, which are evaluated
# relative to the working directory. Since we provide our own working directory,
# we need to use the RUNFILES_DIR instead.
export RUNFILES_DIR="${RUNFILES_MANIFEST_FILE%_manifest}"
unset RUNFILES_MANIFEST_FILE

if [[ ! -v TERMINFO || ! "${TERMINFO}" =~ ^b64:|^hex: ]]; then
    # We don't want to depend on the terminfo database in the container.
    if TERMINFO="$(infocmp -0 -q -Q2)"; then
        export TERMINFO
    fi
fi
"""

def _ebuild_debug_impl(ctx):
    src_basename = _get_basename(ctx)

    # Declare outputs.
    output_debug_script = ctx.actions.declare_file(src_basename + "_debug.sh")

    # Compute arguments and inputs to run build_package.
    # While we include all relevant input files in the wrapper script's
    # runfiles, we embed execroot paths in the script, not runfiles paths, so
    # that the debug invocation is closer to the real build execution.
    build_package_args = _compute_build_package_args(ctx, output_file = None, use_runfiles = False)

    # Try to add --ccache-dir if the directory is specified but ccache is not enabled.
    # So people can enable ccache with the ebuld_debug script,
    # without worrying about passing the ccache directory.
    ccache, ccache_dir = _ccache_settings(ctx)
    if ccache_dir and not ccache:
        # Only do this when ccache is not enabled, and
        # _compute_build_package_args doesn't set the --ccache-dir flag.
        build_package_args.args.add(ccache_dir, format = "--ccache-dir=%s")

    # An interactive run will make --login default to after.
    # The user can still explicitly set --login=before if they wish.
    build_package_args.args.add("--interactive")

    return wrap_binary_with_args(
        ctx,
        out = output_debug_script,
        binary = ctx.executable._action_wrapper,
        args = build_package_args.args,
        content_prefix = _DEBUG_SCRIPT,
        runfiles = ctx.runfiles(transitive_files = build_package_args.inputs),
    )

# TODO(b/298889830): Remove this rule once chromite starts using install_list.
ebuild_debug, ebuild_debug_primordial = maybe_primordial_rule(
    implementation = _ebuild_debug_impl,
    executable = True,
    doc = "Enters the ephemeral chroot to build a Portage binary package in.",
    attrs = dict(
        _bash_runfiles = BASH_RUNFILES_ATTR,
        suffix = attr.string(
            doc = """
            Suffix to add to the output file. i.e., libcxx-17.0-r15<suffix>_debug.sh
            """,
        ),
        **_EBUILD_COMMON_ATTRS
    ),
)

_EbuildInstalledInfo = provider(fields = dict(
    checksum = "(File) File containing a hash of the transitive runtime deps",
))

def _ebuild_install_action_impl(ctx):
    pkg = ctx.attr.package[BinaryPackageInfo]
    install_log = ctx.actions.declare_file(ctx.label.name + ".log")
    checksum = ctx.actions.declare_file(ctx.label.name + ".sha256sum")

    pkg_name = "%s/%s-%s" % (pkg.category, pkg.package_name, pkg.version)
    args = ctx.actions.args()
    args.add_all([
        "--log",
        install_log,
        ctx.executable._installer,
        "-b",
        pkg.partial,
        "-d",
        "/build/%s/packages/%s.tbz2" % (
            ctx.attr.board,
            pkg_name,
        ),
        "-e",
        "emerge-%s --usepkgonly --nodeps --jobs =%s" % (
            ctx.attr.board,
            pkg_name,
        ),
        "-c",
        checksum,
        "-t",
        ctx.executable._xpaktool,
    ])

    for key in ctx.attr.xpak:
        val = ctx.attr.xpak[key]

        args.add_all(["-x", "%s=%s" % (key, val)])

    inputs = [pkg.partial, ctx.executable._installer, ctx.executable._xpaktool]

    # The only use of this is to ensure that our checksum is a hash of the
    # transitive dependencies rather than just this file.
    # This ensures that if we have foo -> bar -> baz, and we change baz, foo
    # will reinstall itself.
    for dep in ctx.attr.requires:
        inputs.append(dep[_EbuildInstalledInfo].checksum)
        args.add("-s")
        args.add(dep[_EbuildInstalledInfo].checksum)

    ctx.actions.run(
        executable = ctx.executable._action_wrapper,
        inputs = inputs,
        arguments = [args],
        outputs = [checksum, install_log],
        execution_requirements = {
            # This implies no-sandbox and no-remote
            "local": "1",
            # Ideally we should cache this if it matches the most recent hash
            # in the cache, but the disk cache is considered a local cache, and
            # can store older hashes.
            "no-cache": "1",
        },
        mnemonic = "EbuildInstall",
        progress_message = "Installing %s to sysroot" % pkg_name,
    )

    return [
        # We don't really want DefaultInfo, but this forces it to actually
        # execute the installation when we build the target.
        DefaultInfo(files = depset([checksum])),
        _EbuildInstalledInfo(checksum = checksum),
        OutputGroupInfo(logs = [install_log]),
    ]

ebuild_install_action = rule(
    implementation = _ebuild_install_action_impl,
    doc = "Installs the package to the permanent SDK using a bazel action.",
    attrs = dict(
        package = attr.label(
            providers = [BinaryPackageInfo],
            mandatory = True,
        ),
        board = attr.string(
            mandatory = True,
            doc = """
            The target board name to build the package for.
            """,
        ),
        xpak = attr.string_dict(
            doc = """
            Overrides the specified XPAK values in the binary package before
            installing.
            """,
        ),
        requires = attr.label_list(providers = [_EbuildInstalledInfo]),
        _installer = attr.label(
            default = "//bazel/portage/build_defs:ebuild_installer",
            executable = True,
            cfg = "exec",
        ),
        _xpaktool = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/xpaktool"),
        ),
        _action_wrapper = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
    ),
    provides = [_EbuildInstalledInfo],
)

def _ebuild_test_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]

    # Declare outputs.
    output_runner_script = ctx.actions.declare_file(src_basename + "_test.sh")

    # Compute arguments and inputs to run build_package.
    build_package_args = _compute_build_package_args(ctx, output_file = None, use_runfiles = True)
    build_package_args.args.add("--test")

    return wrap_binary_with_args(
        ctx,
        out = output_runner_script,
        binary = ctx.executable._action_wrapper,
        args = build_package_args.args,
        runfiles = ctx.runfiles(transitive_files = build_package_args.inputs),
    )

ebuild_test, ebuild_primordial_test = maybe_primordial_rule(
    implementation = _ebuild_test_impl,
    doc = "Runs ebuild tests.",
    attrs = dict(
        _bash_runfiles = BASH_RUNFILES_ATTR,
        **_EBUILD_COMMON_ATTRS
    ),
    test = True,
)

def _ebuild_compare_package_test_impl(ctx):
    if len(ctx.attr.packages) != 2:
        fail("Expected two packages, got %d" % (len(ctx.attr.packages)))

    inputs = [
        package[BinaryPackageInfo].file
        for package in ctx.attr.packages
    ]

    args = ["compare-packages"]
    for file in inputs:
        args.append(file)

    return wrap_binary_with_args(
        ctx,
        out = ctx.outputs.executable,
        binary = ctx.attr._xpaktool,
        args = args,
        content_prefix = "export RUST_BACKTRACE=1",
        runfiles = ctx.runfiles(transitive_files = depset(inputs)),
    )

ebuild_compare_package_test, ebuild_compare_package_primordial_test = maybe_primordial_rule(
    implementation = _ebuild_compare_package_test_impl,
    doc = """
    Compares two binary packages and ensures they are identical. This test is
    helpful to ensure that ebuild outputs are hermetic.

    Unfortunately this test can't guarantee that two hosts will produce the same
    binary package. i.e., The `make -j <cores>` might get logged into the ebuild
    environment file which is build machine specific.
    """,
    attrs = {
        "packages": attr.label_list(
            doc = "The two binary packages to compare.",
            providers = [BinaryPackageInfo],
            mandatory = True,
        ),
        "_bash_runfiles": BASH_RUNFILES_ATTR,
        "_xpaktool": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/xpaktool"),
        ),
    },
    test = True,
)
