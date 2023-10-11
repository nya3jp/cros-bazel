# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def map_install_group(group):
    """
    Computes an --install-target argument for an install group.

    Args:
        group: list[BinaryPackageInfo]: An install group.

    Returns:
        str: A value for the --install-target flag.
    """
    return ":".join([pkg.file.path for pkg in group])

def calculate_install_groups(install_list, provided_packages, use_layers):
    """
    Splits a package set to install groups and pre-installed layers.

    Args:
        install_list: list[BinaryPackageInfo]: A list of packages to install.
            This list must be closed over transitive runtime dependencies.
        provided_packages: depset(BinaryPackageInfo): The packages that have
            already been installed in previous SDK layers. These packages will
            be filtered out.
        use_layers: bool: If true, return a list of layers containing installed
            packages in addition to package groups.

    Returns:
        list[list[BinaryPackageInfo]]: An ordered list containing a list of
            packages that can be installed in parallel.
        list[File]: An ordered list containing the installed binary package.
            This will only be returned if use_layers=True
    """

    # The size of provided packages is normally expected to be O(~20) or less.
    seen = {dep.file.path: True for dep in provided_packages.to_list()}
    install_list = [dep for dep in install_list if dep.file.path not in seen]

    if use_layers:
        layers = [
            dep.layer
            for dep in install_list
            if dep.layer
        ]

        for dep in install_list:
            if dep.layer:
                seen[dep.file.path] = True

        remaining_packages = [
            dep
            for dep in install_list
            if not dep.layer
        ]
    else:
        remaining_packages = install_list

    groups = []
    for _ in range(100):
        if len(remaining_packages) == 0:
            break

        satisfied_list = []
        not_satisfied_list = []
        for package in remaining_packages:
            all_seen = True
            for dep in package.direct_runtime_deps:
                if dep.path not in seen:
                    all_seen = False
                    break

            if all_seen:
                satisfied_list.append(package)
            else:
                not_satisfied_list.append(package)

        if len(satisfied_list) == 0:
            fail("Dependency list is unsatisfiable")

        for dep in satisfied_list:
            seen[dep.file.path] = True

        groups.append(satisfied_list)
        remaining_packages = not_satisfied_list

    if len(remaining_packages) > 0:
        fail("Too many dependencies")

    if use_layers:
        return groups, layers
    else:
        return groups
