# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def add_install_groups(args, install_groups):
    """
    Adds --install-target to the command-line for a list of install groups.

    Args:
        args: ctx.actions.Args: Args to add the install groups to.
        group: list[list[BinaryPackageInfo]]: A list of install groups.
    """
    for group in install_groups:
        args.add_joined(
            "--install-target",
            [binpkg.partial for binpkg in group],
            join_with = ":",
        )

def calculate_install_groups(install_list, provided_packages):
    """
    Splits a package set to install groups and pre-installed layers.

    Args:
        install_list: list[BinaryPackageInfo]: A list of packages to install.
            This list must be closed over transitive runtime dependencies.
        provided_packages: depset(BinaryPackageInfo): The packages that have
            already been installed in previous SDK layers. These packages will
            be filtered out.

    Returns:
        list[list[BinaryPackageInfo]]: An ordered list containing a list of
            packages that can be installed in parallel.
    """

    # The size of provided packages is normally expected to be O(~20) or less.
    seen = {dep.partial.path: True for dep in provided_packages.to_list()}
    remaining_packages = [dep for dep in install_list if dep.partial.path not in seen]

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
            seen[dep.partial.path] = True

        groups.append(satisfied_list)
        remaining_packages = not_satisfied_list

    if len(remaining_packages) > 0:
        fail("Too many dependencies")

    return groups
