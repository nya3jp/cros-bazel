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
        "file": """
            File: A binary package file (.tbz2) of this package.
        """,
        "contents": """
            ContentsLayersInfo: Locates installed/staged contents layers that
            are used to implement fast package installation.

            Since a BinaryPackageInfo contains exactly one ContentsLayersInfo,
            a binary package can be installed only to a single sysroot directory
            that is specified statically in the rule building BinaryPackageInfo.
            An implication of this is that, when building a host package, we
            have to statically choose whether to install it to "/" or
            "/build/amd64-host".
        """,
        "category": """
            str: The category of this package, e.g. "chromeos-base".
        """,
        "package_name": """
            str: The short name of this package, e.g. "chromeos-chrome".
        """,
        "version": """
            str: The version of this package, e.g. "2.5.1-r2".
        """,
        "slot": """
            str: The slot value of this package in the form of "main/sub",
            e.g. "0/1.2.3".
        """,
        "direct_runtime_deps": """
            tuple[File]: Direct runtime dependencies of the package.
            See the provider description for why this field is a tuple, not a
            list.
        """,
    },
    init = _binary_package_info_init,
)

ContentsLayersInfo = provider(
    """
    Locates an installed/staged contents layer for a package.

    This is an essentially a named struct. It always appears in an attribute of
    BinaryPackageInfo.
    """,
    fields = {
        "sysroot": """
            str: A sysroot directory path where the package is installed. It
            must be either "/build/<board>" or "/".
        """,
        "installed": """
            File: A durable tree directory containing an installed contents
            layer.
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
        "files": """
            Depset[File]: All Portage binary package files included in this set.

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
        "path": """
            String: Path inside the container where the overlay's ebuilds are
            mounted.
        """,
        "layer": """
            File: A file which represents an overlay layer. A layer
            file can be a tar file (.tar or .tar.zst).
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

SDKInfo = provider(
    """
    Contains information necessary to mount an ephemeral CrOS SDK.
    """,
    fields = {
        "layers": """
            File[]: A list of files each of which represents a file system layer
            of the SDK. A layer file can be a directory or a tar file (.tar or
            .tar.zst). Layers are ordered from lower to upper; in other words,
            a file from a layer can be overridden by one in another layer that
            appears later in the list.
        """,
        "packages": """
            Depset[BinaryPackageInfo]: The packages that are installed in the
            SDK layer.
        """,
    },
)

EbuildLibraryInfo = provider(
    "Ebuild library info",
    fields = {
        "strip_prefix": """
            str: The prefix to strip off the files when installing into the sdk.
        """,
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

def compute_input_file_path(file, use_runfiles):
    """
    Computes a file path referring to the given input file.

    This function helps you to refer to input file path correctly in the two
    major different working directory configurations: execroot and runfiles.

    When you are going to use a file in a build action run by "bazel build",
    pass use_runfiles=False. The function will just return `file.path` that is
    valid in an action execroot.

    When you are going to use a file in a binary file invoked for "bazel run"
    or "bazel test", pass use_runfiles=True and make sure to include the file
    in the runfiles of the binary. Then this function will return a file path
    you can refer to the file in the runfile tree of the binary.

    Args:
        file: File: An input file.
        use_runfiles: bool: Whether to refer to the input file in a path
            relative to execroot or runfiles directory.

    Returns:
        A file path referring to the given file.
    """
    if file.owner == None:
        fail("Unable to compute a path for a file not associated with a label")
    if use_runfiles:
        return paths.join(_workspace_root(file.owner), file.short_path)
    else:
        return file.path

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
        files = depset(
            [self_package.file],
            transitive = [s.files for s in package_sets],
        ),
    )
