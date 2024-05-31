# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Creates a self-extracting hermetic launcher for a given binary."""

import io
import os
import sys
import tarfile

from cros.bazel.module_extensions.toolchains.hermetic_launcher import (
    elf_analyzer,
)


SELF_EXTRACTOR = """#!/bin/bash -eu

SELF="${BASH_SOURCE[0]}"

OUT="$(mktemp -d)"

tail --bytes=${N_BYTES} "${SELF}" | tar -x --skip-old-files --no-same-owner -C "${OUT}"

# https://stackoverflow.com/questions/24111981/how-can-i-achieve-bash-exit-trap-when-exec-ing-another-binary
{ while kill -0 $$; do sleep 5; done; rm "${out}"; } >/dev/null 2>/dev/null &

# LD_DEBUG attempts to LD_DEBUG the shell script, so we add LD_HERMETIC_DEBUG
# to debug the binary itself.
LD_DEBUG="${LD_HERMETIC_DEBUG:-}" exec "${OUT}/${NAME}" \
    --argv0 "$0" \
    --library-path "${OUT}/_hermetic_lib" \
    --inhibit-rpath '' \
    "${OUT}/_real_binary" \
    "$@"
"""


def _strip(info: tarfile.TarInfo):
    info.uid = 0
    info.gid = 0
    info.uname = ""
    info.gname = ""
    return info


def main(real_bin: str, out: str, interp: str, libs: list[str]):
    # TODO: Optimize this by calculating the dependency map in another action,
    # instead of doing this in *every* compilation action.
    dep_map = elf_analyzer.get_dependency_map(libs)
    required_libs = set()
    for dep in elf_analyzer.get_elf_metadata(real_bin).deps:
        required_libs.update(dep_map.get(dep, []))

    lib_dir = os.path.dirname(libs[0])
    libs = [os.path.join(lib_dir, lib) for lib in sorted(required_libs)]

    name = os.path.basename(out)
    tarball_io = io.BytesIO()
    with tarfile.open(fileobj=tarball_io, mode="w") as tarball:
        # This allows us to bypass the need for LD_ARGV0_REL
        tarball.add(os.path.realpath(interp), arcname=name, filter=_strip)
        tarball.add(
            os.path.realpath(real_bin), arcname="_real_binary", filter=_strip
        )
        for lib in libs:
            tarball.add(
                os.path.realpath(lib),
                arcname=f"_hermetic_lib/{os.path.basename(lib)}",
                filter=_strip,
            )

    tarball_io.seek(0)
    tarball_bytes = tarball_io.read()

    with open(out, "wb") as out_f:
        extractor = SELF_EXTRACTOR.replace(
            "${N_BYTES}", str(len(tarball_bytes))
        ).replace("${NAME}", name)
        out_f.write(extractor.encode("utf-8"))
        out_f.write(tarball_bytes)


if __name__ == "__main__":
    main(
        real_bin=sys.argv[1],
        out=sys.argv[2],
        interp=sys.argv[3],
        libs=sys.argv[4:],
    )
