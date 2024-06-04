# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")

def _binary_package_info_init(*, slot, **kwargs):
    if len(slot.split("/")) != 2:
        fail("Invalid SLOT value: %s" % slot)

    return dict(slot = slot, **kwargs)

BinaryPackageInfo, _new_binary_package_info = provider(
    """
    Describes a Portage binary package.

    A rule providing BinaryPackageInfo must also provide BinaryPackageSetInfo
    that covers all transitive runtime dependencies of this package.

    All fields in this provider must have immutable types (i.e. no list/dict)
    because a BinaryPackageInfo is always accompanied by a BinaryPackageSetInfo
    referencing it with a depset.
    """,
    fields = {
        "category": """
            str: The category of this package, e.g. "chromeos-base".
        """,
        "contents": """
            ContentLayerTypesInfo: Locates the full/internal and
            installed/staged contents layers that are used to implement fast
            package installation.

            Since a BinaryPackageInfo contains exactly one
            ContentLayerTypesInfo, a binary package can be installed only to a
            single sysroot directory that is specified statically in the rule
            building BinaryPackageInfo. An implication of this is that, when
            building a host package, we have to statically choose whether to
            install it to "/" or "/build/amd64-host".
        """,
        "direct_runtime_deps": """
            tuple[File]: Direct runtime dependencies of the package.
            See the provider description for why this field is a tuple, not a
            list.
        """,
        "metadata": """
            File: A json file containing metadata about the package that cannot
            be determined during the analysis phase.
        """,
        "package_name": """
            str: The short name of this package, e.g. "chromeos-chrome".
        """,
        "partial": """
            File: A binary package file (.tbz2) of this package. This package
            doesn't contain *DEPEND XPAK entries suitable for installation
            into a portage sysroot. See `ebuild_install_action` for how to
            populate the XPAK entries and install this package into a portage
            sysroot.
        """,
        "slot": """
            str: The slot value of this package in the form of "main/sub",
            e.g. "0/1.2.3".
        """,
        "version": """
            str: The version of this package, e.g. "2.5.1-r2".
        """,
    },
    init = _binary_package_info_init,
)

ContentLayerTypesInfo = provider(
    """
    Locates a the full/internal contents layers for a package.

    This is an essentially a named struct. It always appears in an attribute of
    BinaryPackageInfo.
    """,
    fields = {
        "full": """
            ContentLayersInfo: The layers to use when building an SDK tarball
            or a full OS image. These layers contain the full vdb that is
            required for `emerge` to function correctly.
        """,
        "internal": """
            ContentLayersInfo: The layers to use when building other packages
            using bazel. These layers have a vdb that drops the revision number
            and non-essential entries.
        """,
        "sysroot": """
            str: A sysroot directory path where the package is installed. It
            must be either "/build/<board>" or "/".
        """,
    },
)

ContentLayersInfo = provider(
    """
    Locates an installed/staged contents layer for a package.
    """,
    fields = {
        "installed": """
            File: A durable tree directory containing an installed contents
            layer.
        """,
        "interface": """
            Optional[File]: A durable tree directory containing interface
            libraries derived from the installed contents layer.
        """,
        "staged": """
            File: A durable tree directory containing a staged contents layer.
        """,
    },
)

BinaryPackageSetInfo = provider(
    """
    Represents a set of Portage binary packages.

    A package set represented by this provider is always closed over transitive
    runtime dependencies. That is, if the set contains a package X, it also
    contains all transitive dependencies of the package X.

    A rule providing BinaryPackageInfo must also provide BinaryPackageSetInfo
    that covers all transitive runtime dependencies of this package.
    """,
    fields = {
        "packages": """
            Depset[BinaryPackageInfo]: All Portage binary packages included in
            this set.

            The Depset must be constructed in a way so that its to_list()
            returns packages in a valid installation order, i.e. a package's
            runtime dependencies are fully satisfied by packages that appear
            before it.
        """,
        "partials": """
            Depset[File]: All Portage binary package files included in this set.
            These binary packages don't have the required metadata to be
            installed by portage.

            The Depset must be constructed in a way so that its to_list()
            returns packages in a valid installation order, i.e. a package's
            runtime dependencies are fully satisfied by packages that appear
            before it.
        """,
    },
)

OverlayInfo = provider(
    "Portage overlay info",
    fields = {
        "layer": """
            File: A file which represents an overlay layer. A layer
            file can be a tar file (.tar or .tar.zst).
        """,
        "path": """
            String: Path inside the container where the overlay's ebuilds are
            mounted.
        """,
    },
)

OverlaySetInfo = provider(
    "Portage overlay set info",
    fields = {
        "layers": """
            File[]: A list of files each of which represents an overlay. A layer
            file can be a directory or a tar file (.tar or .tar.zst). Layers are
            ordered from lower to upper; in other words, a file from a layer can
            be overridden by one in another layer that appears later in the
            list.
        """,
    },
)

BashrcInfo = provider(
    "Portage bashrc info",
    fields = {
        "layer": """
            File: A file which represents an overlay layer. A layer
            file can be a tar file (.tar or .tar.zst).
        """,
        "path": """
            String: Path inside the container where the bashrc is mounted.
        """,
    },
)

SDKInfo = provider(
    """
    Contains information necessary to mount an ephemeral CrOS SDK.
    """,
    fields = {
        "layers": """
            SDKLayer[]: A list of filesystem layers. A layer file can be a
            directory or a tar file (.tar or .tar.zst). Layers are ordered from
            lower to upper; in other words, a file from a layer can be
            overridden by one in another layer that appears later in the list.
        """,
        "packages": """
            Depset[BinaryPackageInfo]: The packages that are installed in the
            SDK layer.
        """,
    },
)

SDKLayer = provider(
    """
    Represents a single layer in an SDK.
    """,
    fields = {
        "file": """
            File: Represents a filesystem layer of the SDK. A layer file can be
            a directory or a tar file (.tar or .tar.zst).
        """,
        "interface_file": """
            Option[File]: If present, it contains an interface library layer
            derived from the `file`.
        """,
    },
)
SysrootInfo = provider(
    "A sysroot in the permanent SDK where packages are installed.",
    fields = {
        "output": "File: (Empty) file to chain dependencies.",
    },
)

EbuildLibraryInfo = provider(
    "Ebuild library info",
    fields = {
        "headers": """
            Depset[File]: Headers provided by the package.
        """,
        "pkg_configs": """
            Depset[File]: .pc files provided by the package.
        """,
        "shared_libs": """
            Depset[File]: .so files provided by the package.
        """,
        "static_libs": """
            Depset[File]: .a files provided by the package.
        """,
        "strip_prefix": """
            str: The prefix to strip off the files when installing into the sdk.
        """,
    },
)

ExtraSourcesInfo = provider(
    """
    Describes extra source codes provided to ephemeral containers.

    This provider can be produced only by the `extra_sources` rule which
    essentially wraps rules_pkg while enforcing that files included in the
    tarball have exactly the same file paths as the original ChromeOS source
    checkout. This ensures that the source code layout in ephemeral containers
    does not deviate from that of Portage-based builds.
    """,
    fields = {
        "tar": "File: A .tar.zst file containing extra source codes.",
    },
)

TransitiveLogsInfo = provider(
    """
    Collects log files in transitive dependencies.

    This provider is used by an aspect to collect logs from all transitive
    dependencies of the targets specified in the command line.
    """,
    fields = {
        "files": """
            Depset[File]: Log files of the current target and its transitive
                dependencies.
        """,
    },
)

# rustc flags to enable debug symbols.
RUSTC_DEBUG_FLAGS = ["--codegen=debuginfo=2"]

def _workspace_root(label):
    return paths.join("..", label.workspace_name) if label.workspace_name else ""

def relative_path_in_label(file, label):
    return paths.relativize(file.short_path, paths.join(_workspace_root(label), label.package))

def relative_path_in_package(file):
    owner = file.owner
    if owner == None:
        fail("File does not have an associated owner label")
    return relative_path_in_label(file, owner)

def compute_file_arg(file, use_runfiles):
    """
    Computes a parameter compatible with ctx.actions.Args for a given file.

    This function helps you to refer to input file path correctly in the two
    major different working directory configurations: execroot and runfiles.

    When you are going to use a file in a build action run by "bazel build",
    pass use_runfiles=False. The function will just return the file.
    We return the file instead of the path because ctx.actions.Args should
    always prefer being passed the file instead of the path.

    When you are going to use a file in a binary file invoked for "bazel run"
    or "bazel test", pass use_runfiles=True and make sure to include the file
    in the runfiles of the binary. Then this function will return a file path
    you can refer to the file in the runfile tree of the binary.

    Args:
        file: File: An input file.
        use_runfiles: bool: Whether to refer to the input file in a path
            relative to execroot or runfiles directory.

    Returns:
        If use_runfiles is false, returns the input file.
        Otherwise, returns a path referring to the given file.
    """
    if file.owner == None:
        fail("Unable to compute a path for a file not associated with a label")
    if use_runfiles:
        return paths.join(_workspace_root(file.owner), file.short_path)
    else:
        return file

def single_binary_package_set_info(self_package, package_sets):
    """
    Creates BinaryPackageSetInfo for a single binary package.

    Args:
        self_package: BinaryPackageInfo: BinaryPackageInfo of the given package.
        package_sets: list[BinaryPackageSetInfo]: Transitive runtime
            dependencies of direct runtime dependencies of the given package.

    Returns:
        BinaryPackageSetInfo: A provider representing all transitive runtime
            dependencies of the given binary package.
    """
    return BinaryPackageSetInfo(
        packages = depset(
            [self_package],
            transitive = [s.packages for s in package_sets],
            order = "postorder",
        ),
        partials = depset(
            [self_package.partial],
            transitive = [s.partials for s in package_sets],
        ),
    )

def get_all_deps(ctx):
    """
    Collects all dependencies specified in the attributes of the current rule.

    Args:
        ctx: ctx: Context passed to the current rule implementation function.

    Returns:
        list[Depset[Target]]: All dependency targets.
    """
    deps = []
    for name in dir(ctx.rule.attr):
        attr = getattr(ctx.rule.attr, name)
        if type(attr) == "Target":
            deps.append(attr)
        elif type(attr) == "list":
            for value in attr:
                if type(value) == "Target":
                    deps.append(value)
        elif type(attr) == "dict":
            for key, value in attr.items():
                if type(key) == "Target":
                    deps.append(key)
                if type(value) == "Target":
                    deps.append(value)
    return deps

def sdk_to_layer_list(sdk, interface_layers = False):
    """
    Returns a list of filesystem layers that make up the SDK.

    Args:
        sdk: SDKInfo: The SDK Info.
        interface_layers: bool: Prefer returning interface layers if they are
            available.

    Returns:
        list[File]: All the layers for the SDK.
    """
    layers = []
    for layer in sdk.layers:
        interface_file = getattr(layer, "interface_file", None)
        if interface_layers and interface_file:
            layers.append(interface_file)
        else:
            layers.append(layer.file)
    return layers
