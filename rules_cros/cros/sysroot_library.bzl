# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""sysroot_library is a repository rule for importing libraries from a Chrome OS sysroot."""

PATH_SEPARATOR = ":"

def _execute_bash(repo_ctx, cmd):
    return repo_ctx.execute(["/bin/bash", "-c", cmd]).stdout.strip("\n")

def _find_compiler(repo_ctx):
    compiler_name = repo_ctx.os.environ.get("CC")
    compiler = _execute_bash(repo_ctx, "type -p {}".format(compiler_name))
    if compiler:
        return compiler
    else:
        fail("Unable to locate compiler: {}".format(compiler_name))

def _get_system_include_paths(repo_ctx):
    compiler = _find_compiler(repo_ctx)
    sysroot = repo_ctx.os.environ.get("SYSROOT")
    cmd = """
        SYSROOT="{sysroot}" \\
        {compiler} -print-search-dirs | \\
        sed -n -e 's/^libraries: =//p'
    """.format(
        sysroot = sysroot,
        compiler = compiler,
    )
    paths = _execute_bash(repo_ctx, cmd)
    return paths.split(PATH_SEPARATOR)

def _find_lib_path(repo_ctx, archive_names):
    compiler = _find_compiler(repo_ctx)
    sysroot = repo_ctx.os.environ.get("SYSROOT")
    for archive_name in archive_names:
        cmd = """
            SYSROOT="{sysroot}" \\
            {compiler} -x c - -o /dev/null -Wl,--trace -l:{archive_name} -nostdlib <<<'int main(){{}}' | \\
            sed -e '/.*\\.o$/d'
            """.format(
            sysroot = sysroot or "",
            compiler = compiler,
            archive_name = archive_name,
        )
        path = _execute_bash(repo_ctx, cmd)
        if path:
            return (archive_name, path)
    return (None, None)

def _find_header_path(repo_ctx, header_name, includes):
    system_includes = _get_system_include_paths(repo_ctx)
    all_includes = includes + system_includes

    for directory in all_includes:
        cmd = """
              test -f "{dir}/{hdr}" && echo "{dir}/{hdr}"
              """.format(dir = directory, hdr = header_name)
        result = _execute_bash(repo_ctx, cmd)
        if result:
            return result
    return None

def _sysroot_library_impl(repo_ctx):
    repo_name = repo_ctx.attr.name
    includes = repo_ctx.attr.includes
    hdrs = repo_ctx.attr.hdrs
    optional_hdrs = repo_ctx.attr.optional_hdrs
    deps = repo_ctx.attr.deps
    static_lib_names = repo_ctx.attr.static_lib_names
    shared_lib_names = repo_ctx.attr.shared_lib_names

    static_lib_name, static_lib_path = _find_lib_path(
        repo_ctx,
        static_lib_names,
    )
    shared_lib_name, shared_lib_path = _find_lib_path(
        repo_ctx,
        shared_lib_names,
    )

    if not static_lib_path and not shared_lib_path:
        fail("Library {} could not be found".format(repo_name))

    hdr_names = []
    hdr_paths = []
    for hdr in hdrs:
        hdr_path = _find_header_path(repo_ctx, hdr, includes)
        if hdr_path:
            repo_ctx.symlink(hdr_path, hdr)
            hdr_names.append(hdr)
            hdr_paths.append(hdr_path)
        else:
            fail("Could not find required header {}".format(hdr))

    for hdr in optional_hdrs:
        hdr_path = _find_header_path(repo_ctx, hdr, includes)
        if hdr_path:
            repo_ctx.symlink(hdr_path, hdr)
            hdr_names.append(hdr)
            hdr_paths.append(hdr_path)

    hdrs_param = "hdrs = {},".format(str(hdr_names))

    # This is needed for the case when quote-includes and system-includes
    # alternate in the include chain, i.e.
    # #include <SDL2/SDL.h> -> #include "SDL_main.h"
    # -> #include <SDL2/_real_SDL_config.h> -> #include "SDL_platform.h"
    # The problem is that the quote-includes are assumed to be
    # in the same directory as the header they are included from -
    # they have no subdir prefix ("SDL2/") in their paths
    include_subdirs = {}
    for hdr in hdr_names:
        path_segments = hdr.split("/")
        path_segments.pop()
        current_path_segments = ["external", repo_name]
        for segment in path_segments:
            current_path_segments.append(segment)
            current_path = "/".join(current_path_segments)
            include_subdirs.update({current_path: None})

    includes_param = "includes = {},".format(str(include_subdirs.keys()))

    deps_names = []
    for dep in deps:
        dep_name = repr("@" + dep)
        deps_names.append(dep_name)
    deps_param = "deps = [{}],".format(",".join(deps_names))

    link_hdrs_command = "mkdir -p $(RULEDIR)/remote \n"
    remote_hdrs = []
    for path, hdr in zip(hdr_paths, hdr_names):
        remote_hdr = "remote/" + hdr
        remote_hdrs.append(remote_hdr)
        link_hdrs_command += "cp {path} $(RULEDIR)/{hdr}\n ".format(
            path = path,
            hdr = remote_hdr,
        )

    link_remote_static_lib_genrule = ""
    link_remote_shared_lib_genrule = ""
    remote_static_library_param = ""
    remote_shared_library_param = ""
    static_library_param = ""
    shared_library_param = ""

    if static_lib_path:
        repo_ctx.symlink(static_lib_path, static_lib_name)
        static_library_param = "static_library = \"{}\",".format(
            static_lib_name,
        )
        remote_static_library = "remote/" + static_lib_name
        link_library_command = """
            mkdir -p $(RULEDIR)/remote && cp {path} $(RULEDIR)/{lib}""".format(
            path = static_lib_path,
            lib = remote_static_library,
        )
        remote_static_library_param = """
static_library = "remote_link_static_library","""
        link_remote_static_lib_genrule = """
genrule(
     name = "remote_link_static_library",
     outs = ["{remote_static_library}"],
     cmd = {link_library_command}
)
""".format(
            link_library_command = repr(link_library_command),
            remote_static_library = remote_static_library,
        )

    if shared_lib_path:
        repo_ctx.symlink(shared_lib_path, shared_lib_name)
        shared_library_param = "shared_library = \"{}\",".format(
            shared_lib_name,
        )
        remote_shared_library = "remote/" + shared_lib_name
        link_library_command = """
mkdir -p $(RULEDIR)/remote && cp {path} $(RULEDIR)/{lib}""".format(
            path = shared_lib_path,
            lib = remote_shared_library,
        )
        remote_shared_library_param = """
shared_library = "remote_link_shared_library","""
        link_remote_shared_lib_genrule = """
genrule(
        name = "remote_link_shared_library",
        outs = ["{remote_shared_library}"],
        cmd = {link_library_command}
)
""".format(
            link_library_command = repr(link_library_command),
            remote_shared_library = remote_shared_library,
        )

    repo_ctx.file(
        "BUILD",
        executable = False,
        content =
            """
load("@bazel_tools//tools/build_defs/cc:cc_import.bzl", "cc_import")
cc_import(
    name = "local_includes",
    {static_library}
    {shared_library}
    {hdrs}
    {deps}
    {includes}
)
genrule(
    name = "remote_link_headers",
    outs = {remote_hdrs},
    cmd = {link_hdrs_command}
)
{link_remote_static_lib_genrule}
{link_remote_shared_lib_genrule}
cc_import(
    name = "remote_includes",
    hdrs = [":remote_link_headers"],
    {remote_static_library}
    {remote_shared_library}
    {deps}
    {includes}
)
alias(
    name = "{name}",
    actual = select({{
        "@bazel_tools//src/conditions:remote": "remote_includes",
        "//conditions:default": "local_includes",
    }}),
    visibility = ["//visibility:public"],
)
""".format(
                static_library = static_library_param,
                shared_library = shared_library_param,
                hdrs = hdrs_param,
                deps = deps_param,
                hdr_names = str(hdr_names),
                link_hdrs_command = repr(link_hdrs_command),
                name = repo_name,
                includes = includes_param,
                remote_hdrs = remote_hdrs,
                link_remote_static_lib_genrule = link_remote_static_lib_genrule,
                link_remote_shared_lib_genrule = link_remote_shared_lib_genrule,
                remote_static_library = remote_static_library_param,
                remote_shared_library = remote_shared_library_param,
            ),
    )

sysroot_library = repository_rule(
    implementation = _sysroot_library_impl,
    local = True,
    remotable = True,
    attrs = {
        "deps": attr.string_list(doc = """
List of names of system libraries this target depends upon.
"""),
        "hdrs": attr.string_list(
            mandatory = True,
            allow_empty = False,
            doc = """
List of the library's public headers which must be imported.
""",
        ),
        "includes": attr.string_list(doc = """
List of directories that should be browsed when looking for headers.
"""),
        "optional_hdrs": attr.string_list(doc = """
List of the library's headers that should be imported if present.
"""),
        "shared_lib_names": attr.string_list(doc = """
List of possible shared library names in order of preference.
"""),
        "static_lib_names": attr.string_list(doc = """
List of possible static library names in order of preference.
"""),
    },
    doc =
        """sysroot_library is a repository rule for importing non-Bazel libraries
from a Chrome OS sysroot.

Currently `sysroot_library` requires two experimental flags:
--experimental_starlark_cc_import
--experimental_repo_remote_exec

""",
)
