# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")
load(":common.bzl", "BinaryPackageInfo", "BinaryPackageSetInfo", "OverlaySetInfo", "SDKInfo", "SDKLayer", "sdk_to_layer_list")
load(":install_deps.bzl", "compute_install_list", "install_deps")

# Print CSV-formatted package installation statistics for each SDK.
#
# This is useful e.g. when iterating on ways to reuse package installations
# from dependencies (b/342012804).
#
# Tip: When filtering out Bazel logs (e.g. Bazel's stdout), use ansifilter
# (https://gitlab.com/saalen/ansifilter) to remove ANSI terminal escape codes,
# such as color changes. Tools like grep might not detect newlines correctly in
# the presence of escape codes.
#
# Only set this to True when iterating locally.
_PRINT_PACKAGE_INSTALLATION_STATS = False

def _sdk_from_archive_impl(ctx):
    output_prefix = ctx.attr.out or ctx.attr.name
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._sdk_from_archive,
        "--input",
        ctx.file.src,
        "--output",
        output_root,
    ], expand_directories = False)

    inputs = [ctx.executable._sdk_from_archive, ctx.file.src]
    outputs = [output_root, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_from_archive],
        arguments = [args],
        mnemonic = "SdkFromArchive",
        progress_message = ctx.attr.progress_message,
    )

    return [
        DefaultInfo(files = depset([output_root])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
        SDKInfo(
            layers = [
                SDKLayer(file = output_root),
            ],
            packages = depset(),
        ),
    ]

sdk_from_archive = rule(
    implementation = _sdk_from_archive_impl,
    attrs = {
        "out": attr.string(
            doc = "Output directory name. Defaults to the target name.",
        ),
        "progress_message": attr.string(
            default = "Extracting SDK archive",
        ),
        "src": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_sdk_from_archive": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/sdk_from_archive"),
        ),
    },
)

def _sdk_update_impl(ctx):
    output_prefix = ctx.attr.out or ctx.attr.name
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._sdk_update,
        "--output",
        output_root,
    ], expand_directories = False)

    base_sdk = ctx.attr.base[SDKInfo]
    layer_inputs = sdk_to_layer_list(base_sdk)
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    args.add_all(ctx.files.extra_tarballs, format_each = "--install-tarball=%s")

    inputs = depset(
        [ctx.executable._sdk_update] + layer_inputs + ctx.files.extra_tarballs,
    )

    outputs = [output_root, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_update],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since sdk_update runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "SdkUpdate",
        progress_message = "Building %{label}",
    )

    return [
        DefaultInfo(files = depset([output_root])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
        SDKInfo(
            layers = base_sdk.layers + [
                SDKLayer(file = output_root),
            ],
            packages = base_sdk.packages,
        ),
    ]

sdk_update = rule(
    implementation = _sdk_update_impl,
    attrs = {
        "base": attr.label(
            mandatory = True,
            providers = [SDKInfo],
        ),
        "extra_tarballs": attr.label_list(
            allow_files = True,
        ),
        "out": attr.string(
            doc = "Output directory name. Defaults to the target name.",
        ),
        "progress_message": attr.string(
            doc = """
            Progress message for this target.
            If the message contains `{dep_count}' it will be replaced with the
            total number of dependencies that need to be installed.
            """,
            default = "Updating SDK",
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_sdk_update": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/sdk_update"),
        ),
    },
)

_SDK_INSTALL_DEPS_COMMON_ATTRS = {
    "base": attr.label(
        doc = """
        Base SDK to derive a new SDK from.
        """,
        mandatory = True,
        providers = [SDKInfo],
    ),
    "out": attr.string(
        doc = "Output directory name. Defaults to the target name.",
    ),
    "progress_message": attr.string(
        doc = """
        Progress message for this target.
        If the message contains `{dep_count}' it will be replaced with the
        total number of dependencies that need to be installed.
        """,
        default = "Installing {dep_count} packages into %{label}",
    ),
    "target_deps": attr.label_list(
        doc = """
        Target packages to install in the SDK.
        """,
        providers = [BinaryPackageSetInfo],
    ),
    "_action_wrapper": attr.label(
        executable = True,
        cfg = "exec",
        default = Label("//bazel/portage/bin/action_wrapper"),
    ),
    "_fast_install_packages": attr.label(
        executable = True,
        cfg = "exec",
        default = Label("//bazel/portage/bin/fast_install_packages"),
    ),
}

def _sdk_install_deps_impl(ctx):
    sdk = ctx.attr.base[SDKInfo]

    install_set = depset(
        transitive = [dep[BinaryPackageSetInfo].packages for dep in ctx.attr.target_deps],
        order = "postorder",
    )

    if _PRINT_PACKAGE_INSTALLATION_STATS:
        # CSV-formatted line with the following columns:
        #  - SDK label,
        #  - Number of packages to install.
        # buildifier: disable=print
        print("%s,%d" % (
            ctx.label,
            len(install_set.to_list()),
        ))

    deps = install_deps(
        ctx = ctx,
        output_prefix = ctx.attr.out or ctx.attr.name,
        board = ctx.attr.board,
        sdk = sdk,
        overlays = ctx.attr.overlays[OverlaySetInfo],
        portage_configs = ctx.files.portage_config,
        install_set = install_set,
        executable_action_wrapper = ctx.executable._action_wrapper,
        executable_fast_install_packages =
            ctx.executable._fast_install_packages,
        progress_message = ctx.attr.progress_message,
        contents = ctx.attr.contents,
    )

    return [
        DefaultInfo(files = depset(
            [layer.file for layer in deps.layers],
        )),
        OutputGroupInfo(
            logs = depset([deps.log_file]),
            traces = depset([deps.trace_file]),
        ),
        SDKInfo(
            layers = sdk.layers + deps.layers,
            packages = depset(transitive = [sdk.packages, install_set]),
        ),
    ]

sdk_install_deps = rule(
    implementation = _sdk_install_deps_impl,
    doc = "Installs packages on top of an existing SDK, yielding a new SDK.",
    attrs = dict(
        board = attr.string(
            doc = """
            If set, the packages are installed into the board's sysroot,
            otherwise they are installed into the host's sysroot.
            """,
        ),
        overlays = attr.label(
            providers = [OverlaySetInfo],
            mandatory = True,
        ),
        portage_config = attr.label_list(
            providers = [PackageArtifactInfo],
            doc = """
            The portage config for the host and optionally the target. This should
            at minimum contain a make.conf file.
            """,
            mandatory = True,
        ),
        contents = attr.string(
            doc = """
            Specifies the how complete the dependency layers are.

            Valid options:
            * full: The layers will contain a full vdb and complete contents.
                Use this when calling emerge or exporting the layers in an
                image or tarball.
            * sparse: The layers will contain a sparse vdb that is only suitable
                for handling `has_version`. It contain all files in their
                original state.
            * interface: When set, it has the same effect as `sparse`, but in
                addition it will also enable interface library layers for the
                specified packages. An interface library layer is a layer with a
                sparse vdb and interface shared objects (shared objects without
                any code). All binaries and any non-essential files have also
                been removed. This layer should only be used for building
                dynamically linked libraries.

                An ebuild can choose to build with interface layers by setting
                `use_interface_libraries=True`.
            """,
            default = "sparse",
            values = ["full", "sparse", "interface"],
        ),
        **_SDK_INSTALL_DEPS_COMMON_ATTRS
    ),
)

def _package_description(p):
    """Returns a human-readable string describing a package."""
    return "%s/%s-%s at %s" % (p.category, p.package_name, p.version, p.contents.sysroot)

def _find_best_base_sdk(host_deps, target_deps, default_base_sdk):
    """Finds the best base SDK upon which to install a set of packages.

    This function takes a set of packages to install on top of a default base
    SDK, and tries to find a "better" base SDK among the reusable SDKs
    associated with the packages to install. The "best" base SDK in a set of
    candidate base SDKs is the one which minimizes the number of required
    package installations; that is, it is the one that already includes the
    largest portion of the packages in the install set. If all the candidate
    base SDKs are worse than the given base SDK in terms of required package
    installations, or if none of the candidates are deemed viable, this
    function returns the given base SDK; otherwise it returns the best base SDK
    found.

    A candidate base SDK is deemed viable if and only if it does not introduce
    any unwanted packages. Put differently, a candidate base SDK is viable iff
    the result of installing the given packages on top of the candidate base
    SDK is identical to the result of installing those packages on top of the
    given base SDK (modulo layer ordering).

    Note: This helper is used by sdk_install_host_and_target_deps. As such, it
    relies on the fact that said function installs packages via the
    install_deps function, which skips any packages already present in the base
    SDK.

    Args:
        host_deps: list[str]: Host packages to install.
        target_deps: list[str]: Target packages to install.
        default_base_sdk: SDKInfo: Base SDK to use if no better candidate is
            found among the reusable SDKs of the host and target packages to
            install.

    Returns: A struct with the following fields:
        - sdk: SDKInfo for the best base SDK, which will be the
          default_base_sdk if no better candidate is found.
        - description: Human-readable description of the best
          base SDK.
        - num_installs: Actual number of installations required when installing
          the given host and target dependencies on top of the best base SDK.
        - default_base_sdk_num_installs: Actual number of installations
          required when installing the given host and target dependencies on
          top of the default base SDK.
    """

    # The packages to install, and their transitive dependencies.
    install_set = depset(
        transitive = [
            dep[BinaryPackageSetInfo].packages
            for dep in host_deps
        ] + [
            dep[BinaryPackageSetInfo].packages
            for dep in target_deps
        ],
        order = "postorder",
    )

    # All the packages we expect in the output SDK; that is, the packages in
    # the base SDK plus the packages in the install set.
    packages_in_output_sdk_path_set = {
        package.partial.path: True
        for package in depset(
            transitive = [default_base_sdk.packages, install_set],
            order = "postorder",
        ).to_list()
    }

    # The list of candidate base SDKs consists of all the reusable SDKs
    # associated with the packages we want to install.
    candidate_base_sdks = [
        struct(
            package_desc = _package_description(package),
            sdk = package.reusable_sdk,
        )
        for package in install_set.to_list()
        if package.reusable_sdk
    ]

    # Number of packages we would install on top of the default base SDK.
    #
    # Note: this is computed using the same helper as the install_deps
    # function.
    default_base_sdk_install_list_size = len(compute_install_list(default_base_sdk, install_set))

    # Best base SDK found so far.
    best_base_sdk = default_base_sdk

    # Human-readable description of the best base SDK so far.
    best_base_sdk_description = "default base SDK (%d installs required)" % default_base_sdk_install_list_size

    # Number of packages we would install on top of the best base SDK so far.
    best_base_sdk_install_list_size = default_base_sdk_install_list_size

    for candidate_sdk in candidate_base_sdks:
        # Filter out any candidate SDKs that introduce unwanted packages.
        bad_candidate = False
        for package in candidate_sdk.sdk.packages.to_list():
            if package.partial.path not in packages_in_output_sdk_path_set:
                bad_candidate = True
                break
        if bad_candidate:
            continue

        # Number of packages we would install on top of this candidate SDK.
        #
        # Note: this is computed using the same helper as the install_deps
        # function.
        candidate_sdk_install_list = compute_install_list(
            candidate_sdk.sdk,
            install_set,
            fail_on_slot_conflict = False,  # Returns None in this case.
        )
        if not candidate_sdk_install_list:
            continue

        # The candidate with the fewest required installations wins.
        if len(candidate_sdk_install_list) < best_base_sdk_install_list_size:
            best_base_sdk = candidate_sdk.sdk
            best_base_sdk_install_list_size = len(candidate_sdk_install_list)
            best_base_sdk_description = "reusable SDK from package %s (%d -> %d installs required)" % (
                candidate_sdk.package_desc,
                default_base_sdk_install_list_size,
                len(candidate_sdk_install_list),
            )

    return struct(
        sdk = best_base_sdk,
        description = best_base_sdk_description,
        num_installs = best_base_sdk_install_list_size,
        default_base_sdk_num_installs = default_base_sdk_install_list_size,
    )

def _sdk_install_host_and_target_deps_impl(ctx):
    if ctx.attr.host_deps:
        if not ctx.attr.host_overlays:
            fail("SDK %s requires installing %d host packages, but no \"host_overlays\" attribute was set." % (ctx.label, len(ctx.attr.host_deps)))
        if not ctx.attr.host_portage_config:
            fail("SDK %s requires installing %d host packages, but no \"host_portage_config\" attribute was set." % (ctx.label, len(ctx.attr.host_deps)))

    if ctx.attr.target_deps:
        if not ctx.attr.target_overlays:
            fail("SDK %s requires installing %d target packages, but no \"target_overlays\" attribute was set." % (ctx.label, len(ctx.attr.target_deps)))
        if not ctx.attr.target_portage_config:
            fail("SDK %s requires installing %d target packages, but no \"target_portage_config\" attribute was set." % (ctx.label, len(ctx.attr.target_deps)))

    # Find the best base SDK among the reusable SDKs associated with the host
    # and target dependencies, defaulting to the provided base SDK if no better
    # option is found.
    best_base_sdk = _find_best_base_sdk(
        ctx.attr.host_deps,
        ctx.attr.target_deps,
        ctx.attr.base[SDKInfo],  # Default base SDK.
    )

    if _PRINT_PACKAGE_INSTALLATION_STATS:
        # CSV-formatted line with the following columns:
        #  - SDK label,
        #  - Number of packages we would install on top of the default base SDK.
        #  - Number of packages to install on top of the best base SDK.
        #  - Delta between the last two columns.
        #  - Description of the chosen base SDK.
        # buildifier: disable=print
        print("%s,%d,%d,%d,%s" % (
            ctx.label,
            best_base_sdk.default_base_sdk_num_installs,
            best_base_sdk.num_installs,
            best_base_sdk.default_base_sdk_num_installs - best_base_sdk.num_installs,
            best_base_sdk.description,
        ))

    host_packages = depset(
        transitive = [
            dep[BinaryPackageSetInfo].packages
            for dep in ctx.attr.host_deps
        ],
        order = "postorder",
    ).to_list()

    target_packages = depset(
        transitive = [
            dep[BinaryPackageSetInfo].packages
            for dep in ctx.attr.target_deps
        ],
        order = "postorder",
    ).to_list()

    sdk = best_base_sdk.sdk

    layers = []
    log_files = []
    trace_files = []

    if host_packages:
        host_install_set = depset(host_packages)

        deps = install_deps(
            ctx = ctx,
            output_prefix = (ctx.attr.out or ctx.attr.name) + "_host",
            board = None,
            sdk = sdk,
            overlays = ctx.attr.host_overlays[OverlaySetInfo],
            portage_configs = ctx.files.host_portage_config,
            install_set = host_install_set,
            executable_action_wrapper = ctx.executable._action_wrapper,
            executable_fast_install_packages =
                ctx.executable._fast_install_packages,
            progress_message = ctx.attr.progress_message + " (%d host dependencies on top of %s)" % (len(host_packages), best_base_sdk.description),
            contents = ctx.attr.host_contents,
        )

        sdk = SDKInfo(
            layers = sdk.layers + deps.layers,
            packages = depset(transitive = [sdk.packages, host_install_set]),
        )

        layers += deps.layers
        log_files.append(deps.log_file)
        trace_files.append(deps.trace_file)

    if target_packages:
        target_install_set = depset(target_packages)

        deps = install_deps(
            ctx = ctx,
            output_prefix = ctx.attr.out or ctx.attr.name,
            board = ctx.attr.board,
            sdk = sdk,
            overlays = ctx.attr.target_overlays[OverlaySetInfo],
            portage_configs = ctx.files.target_portage_config,
            install_set = target_install_set,
            executable_action_wrapper = ctx.executable._action_wrapper,
            executable_fast_install_packages =
                ctx.executable._fast_install_packages,
            progress_message = ctx.attr.progress_message + " (installing %d target dependencies on top of %s)" % (len(target_packages), best_base_sdk.description),
            contents = ctx.attr.target_contents,
        )

        sdk = SDKInfo(
            layers = sdk.layers + deps.layers,
            packages = depset(transitive = [sdk.packages, target_install_set]),
        )

        layers += deps.layers
        log_files.append(deps.log_file)
        trace_files.append(deps.trace_file)

    return [
        DefaultInfo(files = depset([layer.file for layer in layers])),
        OutputGroupInfo(
            logs = depset(log_files),
            traces = depset(trace_files),
        ),
        sdk,
    ]

sdk_install_host_and_target_deps = rule(
    implementation = _sdk_install_host_and_target_deps_impl,
    doc = """Installs host and target packages into an SDK.

    This rule is similar to sdk_install_deps in that it takes a base SDK and a
    list of packages, and produces a new SDK with the result of installing
    those packages on top of the base SDK. However, there are two key
    differences between this rule and sdk_install_deps:

      1. It supports installing both host and target packages via separate
         host_deps and target_deps attributes.

      2. It leverages the reusable SDKs associated with the packages being
         installed (and their transitive dependencies). A package's reusable
         SDK already includes dependencies required by any dependent packages,
         so extending an existing reusable SDK is often more efficient than
         starting from the base SDK. This avoids the need to install packages
         already present in the reusable SDK. The rule analyzes the list of
         packages provided by each reusable SDK associated with the
         installation targets, and builds the final SDK on top of the most
         suitable reusable SDK whenever possible.

    This rule is separate from sdk_install_deps because it makes it easier to
    gradually roll out support for reusable dependencies. Once we have
    confidence in this approach, we may merge this rule into sdk_install_deps.
    See b/342012804 for additional context.
    """,
    attrs = dict(
        board = attr.string(
            doc = """
            Used to determine the board's sysroot for target packages. Has no
            effect on host packages.
            """,
        ),
        host_overlays = attr.label(
            providers = [OverlaySetInfo],
            doc = """
            Portage overlays to use when installing host packages.

            It may be omitted when installing target packages exclusively.
            """,
        ),
        target_overlays = attr.label(
            providers = [OverlaySetInfo],
            doc = """
            Portage overlays to use when installing target packages.

            It may be omitted when installing host packages exclusively.
            """,
        ),
        host_portage_config = attr.label_list(
            providers = [PackageArtifactInfo],
            doc = """
            Portage configs to use when installing host packages.

            This should at minimum contain a make.conf file.

            It may be omitted when installing target packages exclusively.
            """,
        ),
        target_portage_config = attr.label_list(
            providers = [PackageArtifactInfo],
            doc = """
            Portage configs to use when installing target packages.

            This should at minimum contain a make.conf file.

            It may be omitted when installing host packages exclusively.
            """,
        ),
        host_contents = attr.string(
            doc = """
            Specifies the how complete the host dependency layers are.

            Valid options:
            * full: The layers will contain a full vdb and complete contents.
                Use this when calling emerge or exporting the layers in an
                image or tarball.
            * sparse: The layers will contain a sparse vdb that is only suitable
                for handling `has_version`. It contain all files in their
                original state.
            * interface: Contains a sparse vdb and interface shared objects
                (shared objects without any code). All binaries and any
                non-essential files have also been removed. This layer should
                only be used for building dynamically linked libraries.
            """,
            default = "sparse",
            values = ["full", "sparse", "interface"],
        ),
        target_contents = attr.string(
            doc = """
            Specifies the how complete the target dependency layers are.

            Valid options:
            * full: The layers will contain a full vdb and complete contents.
                Use this when calling emerge or exporting the layers in an
                image or tarball.
            * sparse: The layers will contain a sparse vdb that is only suitable
                for handling `has_version`. It contain all files in their
                original state.
            * interface: Contains a sparse vdb and interface shared objects
                (shared objects without any code). All binaries and any
                non-essential files have also been removed. This layer should
                only be used for building dynamically linked libraries.
            """,
            default = "sparse",
            values = ["full", "sparse", "interface"],
        ),
        host_deps = attr.label_list(
            doc = """
            Host packages to install in the SDK.
            """,
            providers = [BinaryPackageSetInfo],
        ),
        **_SDK_INSTALL_DEPS_COMMON_ATTRS
    ),
)

def _sdk_extend_impl(ctx):
    sdk = ctx.attr.base[SDKInfo]

    sdk = SDKInfo(
        layers = sdk.layers + [
            SDKLayer(file = layer)
            for layer in ctx.files.extra_tarballs
        ],
        packages = sdk.packages,
    )

    return [
        # We don't return any files since this rule doesn't create any.
        DefaultInfo(),
        sdk,
    ]

sdk_extend = rule(
    doc = "Adds extra tarballs to the SDK",
    implementation = _sdk_extend_impl,
    provides = [SDKInfo],
    attrs = {
        "base": attr.label(
            doc = """
            Base SDK to derive a new SDK from.
            """,
            mandatory = True,
            providers = [SDKInfo],
        ),
        "extra_tarballs": attr.label_list(
            allow_files = True,
            mandatory = True,
            doc = """
            Extra files to layer onto the base SDK.
            """,
        ),
    },
)

def _sdk_install_glibc_impl(ctx):
    output_prefix = ctx.attr.out or ctx.attr.name
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._sdk_install_glibc,
        "--output",
        output_root,
        "--board",
        ctx.attr.board,
    ], expand_directories = False)

    base_sdk = ctx.attr.base[SDKInfo]
    layer_inputs = sdk_to_layer_list(base_sdk)
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    glibc = ctx.attr.glibc[BinaryPackageInfo].partial
    args.add("--glibc", glibc)

    inputs = depset(
        [ctx.executable._sdk_install_glibc] + layer_inputs + [glibc],
    )

    outputs = [output_root, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_install_glibc],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since sdk_install_glibc runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "SdkInstallGlibc",
        progress_message = "Installing cross glibc into %{label}",
    )

    return [
        DefaultInfo(files = depset([output_root])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
        SDKInfo(
            layers = base_sdk.layers + [
                SDKLayer(file = output_root),
            ],
            packages = depset(
                [ctx.attr.glibc[BinaryPackageInfo]],
                transitive = [base_sdk.packages],
            ),
        ),
    ]

sdk_install_glibc = rule(
    implementation = _sdk_install_glibc_impl,
    attrs = {
        "base": attr.label(
            mandatory = True,
            providers = [SDKInfo],
        ),
        "board": attr.string(
            doc = "The cross-* package is installed into the board's sysroot.",
            mandatory = True,
        ),
        "glibc": attr.label(
            doc = "The cross-*-cros-linux-gnu/glibc package to install",
            mandatory = True,
            providers = [BinaryPackageInfo],
        ),
        "out": attr.string(
            doc = "Output directory name. Defaults to the target name.",
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_sdk_install_glibc": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/sdk_install_glibc"),
        ),
    },
)

def _remote_toolchain_inputs(ctx):
    output_prefix = ctx.attr.name
    output = ctx.actions.declare_file(output_prefix + ".tar.zst")
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._generate_reclient_inputs,
        "--output",
        output,
    ], expand_directories = False)

    layer_inputs = (
        sdk_to_layer_list(ctx.attr.sdk[SDKInfo]) +
        [ctx.file._chromite_src]
    )
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    inputs = depset(
        [ctx.executable._generate_reclient_inputs] + layer_inputs,
    )

    outputs = [output, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._generate_reclient_inputs],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since generate_reclient_inputs runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "GenerateReclientInputs",
        progress_message = "Building %{label}",
    )

    return [
        DefaultInfo(files = depset([output])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
    ]

remote_toolchain_inputs = rule(
    implementation = _remote_toolchain_inputs,
    attrs = {
        "sdk": attr.label(
            doc = "The SDK to generate the remote_toolchain_inputs file for.",
            mandatory = True,
            providers = [SDKInfo],
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_chromite_src": attr.label(
            default = Label("@chromite//:src"),
            allow_single_file = True,
        ),
        "_generate_reclient_inputs": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/generate_reclient_inputs"),
        ),
    },
)
