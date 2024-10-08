#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Implements a fake sudo command for the ephemeral CrOS SDK.

Since the ephemeral CrOS SDK runs in an unprivileged user namespace, the real
sudo command doesn't work. This script pretends like the real one, but actually
does not do anything about privilege.
"""

import dataclasses
import logging
import os
import shlex
import shutil
import sys
from typing import Mapping, Sequence


# Can't use the subprocess class since we're invoking exec directly.
@dataclasses.dataclass
class Cmd:
    """Records the command to run.

    We can't use the subprocess class since we're invoking exec directly.
    """

    args: Sequence[str]
    env: Mapping[str, str]


# The list of environment variables to be forcibly set.
ENV_FORCED = {
    "HOME": "/root",
    "LOGNAME": "root",
    "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/bin:/mnt/host/source/chromite/bin:/mnt/host/depot_tools",
    "SHELL": "/bin/bash",
    "SUDO_GID": "0",
    "SUDO_UID": "0",
    "SUDO_USER": "root",
    "USER": "root",
}

# The set of environment variables to be kept.
# TODO: Keep in sync with chromite.
ENV_KEPT = set(
    (
        "CHROMEOS_OFFICIAL",
        "CHROMEOS_VERSION_AUSERVER",
        "CHROMEOS_VERSION_DEVSERVER",
        "CHROMEOS_VERSION_TRACK",
        "GCE_METADATA_HOST",
        "GIT_AUTHOR_EMAIL",
        "GIT_AUTHOR_NAME",
        "GIT_COMMITTER_EMAIL",
        "GIT_COMMITTER_NAME",
        "GIT_PROXY_COMMAND",
        "GIT_SSH",
        "RSYNC_PROXY",
        "SSH_AGENT_PID",
        "SSH_AUTH_SOCK",
        "TMUX",
        "USE",
        "all_proxy",
        "ftp_proxy",
        "http_proxy",
        "https_proxy",
        "no_proxy",
        "CROS_WORKON_SRCROOT",
        "PORTAGE_USERNAME",
        "TERM",
        "LANG",
    )
)


def parse(orig_args: Sequence[str], env: Mapping[str, str]) -> Cmd:
    logging.info("Arguments: %s", " ".join(shlex.quote(s) for s in orig_args))
    explicit_env = {}
    preserve_env = False
    args = orig_args[:]

    # Don't use argparse, because argparse will attempt to parse unknown args
    # after positional arguments.
    # >>> import argparse
    # >>> parser = argparse.ArgumentParser()
    # >>> parser.add_argument('-u', '--user')
    # >>> parser.parse_known_args(['ls', '-u', '.'])
    # (Namespace(user='.'), ['ls'])
    while args:
        if args[0] == "--":
            args = args[1:]
            break
        elif args[0] in ("-E", "--preserve-env"):
            preserve_env = True
            args = args[1:]
        elif args[0] == "-u" or args[0].startswith("--user"):
            logging.info("Dropping user flag: %s", args[1])
            args = args[1:] if "=" in args[0] else args[2:]
        elif args[0].startswith("-"):
            raise NotImplementedError("Unimplemented sudo arg: %s" % args[0])
        elif "=" in args[0]:
            k, v = args.pop(0).split("=", 1)
            explicit_env[k] = v
        else:
            break
    logging.info("Executing: %s", " ".join(shlex.quote(s) for s in args))

    if not args:
        raise ValueError(f"Command was empty: {orig_args}")

    if preserve_env:
        cmd_env = env.copy()
    else:
        cmd_env = {key: val for key, val in env.items() if key in ENV_KEPT}

    cmd_env.update(ENV_FORCED)

    cmd_env.update(explicit_env)

    logging.info(
        "Environs: %s",
        " ".join(
            "%s=%s" % (shlex.quote(key), shlex.quote(value))
            for key, value in sorted(cmd_env.items())
        ),
    )

    return Cmd(args=args, env=cmd_env)


def main():
    logging.basicConfig(
        stream=sys.stderr,
        level=logging.INFO,
        format="fake_sudo: %(levelname)s: %(message)s",
    )
    logging.info("This is the fake sudo for the ephemeral CrOS SDK.")
    cmd = parse(sys.argv[1:], os.environ)
    exe = shutil.which(cmd.args[0], path=cmd.env["PATH"])
    if exe is None:
        raise OSError(
            "Command not found: %s (PATH=%s)" % (cmd.args[0], cmd.env["PATH"])
        )
    os.execvpe(exe, cmd.args, cmd.env)


if __name__ == "__main__":
    main()
