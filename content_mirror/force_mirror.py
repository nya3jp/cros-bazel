#!/usr/bin/env python3
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Starts up an HTTPS server and writes a bazelrc file that uses that server.

The bazelrc configuration attempts to read from the mirror, and if that fails,
attempts to read from the local server.

The local server attempts to mirror the file, then returns a 301 redirect to the
mirror.
"""

import argparse
import enum
import http
import http.server
import logging
import pathlib
import ssl
import subprocess
import tempfile
from typing import Optional
import urllib
import urllib.request


_MIRROR_PREFIX = "chromeos-localmirror/cros-bazel/mirror"

_BAZELRC = """
common --experimental_downloader_config={downloader_config}
startup --host_jvm_args=-Djavax.net.ssl.trustStore={cacerts}
startup --host_jvm_args=-Djavax.net.ssl.trustStorePassword=changeit
"""


class Mode(enum.Enum):
    """What to do with the next line in a file"""

    UNTOUCHED = 0
    COMMENT_NEXT = 1
    UNCOMMENT_NEXT = 2


_LINES_TO_MODE = {
    "# Comment-during-mirroring:": Mode.COMMENT_NEXT,
    "# Uncomment-during-mirroring:": Mode.UNCOMMENT_NEXT,
}


def _mirror(src, dst):
    logging.info("Mirroring %s to %s", src, dst)
    with tempfile.TemporaryDirectory() as temp_dir:
        local = pathlib.Path(temp_dir, "file")
        urllib.request.urlretrieve(src, local)
        subprocess.run(
            ["gsutil", "cp", "-n", "-a", "public-read", str(local), dst],
            check=True,
        )


class _RequestHandler(http.server.BaseHTTPRequestHandler):
    """A handler that mirrors all requests then redirects to the mirror."""

    def respond(self, code: int, hdrs):
        self.send_response(code)
        for k, v in hdrs.items():
            self.send_header(k, v)
        self.end_headers()

    def do_GET(self):
        path = self.path[1:]
        orig_url = f"https://{path}"
        mirror_url = (
            f"https://commondatastorage.googleapis.com/{_MIRROR_PREFIX}/{path}"
        )
        mirror_gs_url = f"gs://{_MIRROR_PREFIX}/{path}"

        try:
            _mirror(orig_url, mirror_gs_url)
        except urllib.error.HTTPError as e:
            logging.error(
                "Error while trying to access original file %s", orig_url
            )
            self.respond(e.code, e.hdrs)

        # Avoid caching
        self.respond(301, {"Location": mirror_url})

    def do_HEAD(self):
        # We never send the file content back to the user.
        # Thus, HEAD and GET are equivelant for us.
        self.do_GET()


def _start_server(port: int, certfile: pathlib.Path, keyfile: pathlib.Path):
    server_address = ("localhost", port)
    httpd = http.server.HTTPServer(server_address, _RequestHandler)
    ctx = ssl.SSLContext(protocol=ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(certfile=certfile, keyfile=keyfile)
    httpd.socket = ctx.wrap_socket(httpd.socket, server_side=True)
    httpd.serve_forever()


def _generate_config(src: pathlib.Path, port: int, dst: pathlib.Path):
    content = src.read_text(encoding="utf-8").replace("{PORT}", str(port))
    new_config = []
    mode = Mode.UNTOUCHED
    for line in content.splitlines():
        if mode == Mode.UNTOUCHED:
            new_config.append(line)
        elif mode == Mode.COMMENT_NEXT:
            new_config.append(f"# {line}")
        elif mode == Mode.UNCOMMENT_NEXT:
            new_config.append(line.lstrip("# "))
        mode = _LINES_TO_MODE.get(line, Mode.UNTOUCHED)
    dst.write_text("\n".join(new_config), encoding="utf-8")


def main(
    port: int, certs_dir: Optional[pathlib.Path], out: Optional[pathlib.Path]
):
    src_dir = pathlib.Path(__file__).parent

    with tempfile.TemporaryDirectory() as temp_dir:
        temp_dir = pathlib.Path(temp_dir)
        if not certs_dir:
            certs_dir = temp_dir / "certs"
        if not certs_dir.is_dir():
            gen_certs = src_dir / "gen_certs.sh"
            subprocess.run([str(gen_certs), str(certs_dir)], check=True)

        mirror_downloader_out = temp_dir / "mirror_downloader.cfg"
        _generate_config(
            src=src_dir / "bazel_downloader.cfg",
            port=port,
            dst=mirror_downloader_out,
        )

        out_real = temp_dir / "force_mirror.bazelrc"

        out_real.write_text(
            _BAZELRC.format(
                downloader_config=mirror_downloader_out,
                cacerts=certs_dir / "cacerts",
            )
        )
        if out:
            # Ensure that the symlink is broken when the temporary directory is
            # cleaned up.
            out.unlink(missing_ok=True)
            out.symlink_to(out_real)
        else:
            out = out_real

        print(f"Run bazel --bazelrc={out} to force mirroring")

        _start_server(
            port=port,
            certfile=certs_dir / "BazelContentMirrorServer.crt",
            keyfile=certs_dir / "BazelContentMirrorServer.key",
        )


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--port", help="The port to start the server on", default=4443
    )
    parser.add_argument(
        "--certs_dir",
        help="A directory generated by gen_certs.sh",
        type=pathlib.Path,
        default=None,
    )
    parser.add_argument(
        "--out",
        help="The path to the bazelrc file to output",
        type=pathlib.Path,
        default=None,
    )

    main(**vars(parser.parse_args()))
