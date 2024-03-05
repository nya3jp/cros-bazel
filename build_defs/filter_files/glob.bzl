# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Functions for helping with globbing for files in starlark rules"""

visibility([
    "//bazel/build_defs",
    "//bazel/module_extensions/toolchains/files",
])

_NO_MATCH_FMT = """The following entries failed to match anything. If this is \
expected, consider setting allow_entry to True in the filter_files invocation.
{globs}

Example file: "{f}"
"""

def glob_matches(path, glob):
    """Determines whether or not a glob matches a given path.

    This works the same as the glob function in bazel, supporting both * and **.
    Args:
        path: (List[str]): The path to check, encoded as a list
          eg. a/b/c -> ["a", "b", "c"].
        glob: (List[str]): The glob to try, encoded as a list
          eg. a/**/b*/*d* -> ["a", "**", "b*", "*d*"].
    Returns:
        Whether or not the glob matches the path.
    """

    # Thanks to the '**' operator, we need to treat this as a nondeterministic
    # finite state machine, where we can be in multiple states at the same time.
    # Otherwise, we would fail the test "multi_star_with_multiple_options".
    path_uptos = [0]
    for chunk in glob:
        new_uptos = []
        for upto in path_uptos:
            if upto == len(path):
                continue
            dir = path[upto]

            if chunk == "**":
                new_uptos = range(upto, len(path) + 1)
            elif "*" in chunk:
                segments = chunk.split("*")
                if not dir.startswith(segments[0]) or not dir.endswith(segments[-1]):
                    continue

                if len(segments) == 2:
                    new_uptos.append(upto + 1)
                elif len(segments) == 3 and segments[1] in dir[len(segments[0]):len(dir) - len(segments[2])]:
                    new_uptos.append(upto + 1)
                elif len(segments) > 3:
                    fail("Unsupported number of stars")

            elif chunk == dir:
                new_uptos.append(upto + 1)

        if not new_uptos:
            return False
        path_uptos = new_uptos
    return len(path) in path_uptos

def glob(file_map, include, exclude, allow_empty):
    """Filters a file map to just the specified glob.

    Args:
        file_map: (Dict[str, File]) A mapping from human-readable name to file
        include: (List[str]) See native.glob.include
        exclude: (List[str]) See native.glob.exclude
        allow_empty: (bool) See native.glob.allow_empty
    Returns:
        (Dict[str, File]) A subset of file_map containing just the files that
        matched the glob.
    """
    include = [tuple(glob.split("/")) for glob in include]
    exclude = [tuple(glob.split("/")) for glob in exclude]

    matched = {k: False for k in include}
    filtered = {}

    for path_str, f in file_map.items():
        path = path_str.split("/")
        for glob in include:
            if glob_matches(path, glob):
                if not any([glob_matches(path, g) for g in exclude]):
                    filtered[path_str] = f
                    matched[glob] = True
                    break

    missing_matches = [k for k, v in matched.items() if not v]
    if missing_matches and not allow_empty:
        fail(_NO_MATCH_FMT.format(
            globs = ["/".join(glob) for glob in missing_matches],
            f = "/".join(path),
        ))

    return filtered

def _glob_check_impl(ctx):
    path = ctx.attr.path.split("/")
    glob = ctx.attr.glob.split("/")
    got = glob_matches(path, glob)
    want = ctx.attr.want
    if got != want:
        fail("When checking {check}, _matches_glob({path}, {glob}) returned {got}, wanted {want}".format(
            check = ctx.label.name,
            path = path,
            glob = glob,
            got = got,
            want = want,
        ))

glob_check = rule(
    implementation = _glob_check_impl,
    attrs = dict(
        path = attr.string(mandatory = True),
        glob = attr.string(mandatory = True),
        want = attr.bool(mandatory = True),
    ),
)
