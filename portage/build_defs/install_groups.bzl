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

def calculate_install_groups(install_list):
    """
    Splits a package set to install groups.

    Args:
        install_list: list[BinaryPackageInfo]: A list of packages to install.
            This list must be closed over transitive runtime dependencies.

    Returns:
        list[list[BinaryPackageInfo]]: An ordered list containing a list of
            packages that can be installed in parallel.
    """
    groups = []
    remaining_packages = install_list[:]
    seen = {}

    for _ in range(100):
        if len(remaining_packages) == 0:
            break

        satisfied_list = []
        not_satisfied_list = []
        for package in remaining_packages:
            all_seen = True
            for dep in package.direct_runtime_deps:
                if dep.file.path not in seen:
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

    return groups
