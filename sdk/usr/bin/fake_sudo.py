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
import re
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


SUDO_PATH = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/bin:/mnt/host/source/chromite/bin:/mnt/host/depot_tools"
# It probably expects a value, and a valid one. Since we're not actually
# changing the user, we should preserve these values.
_PRESERVED_KEYS = re.compile("^USER|HOME|SUDO_PRESERVED_.*$")


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
        cmd_env = dict(**env)
    else:
        cmd_env = {}
        for key, val in env.items():
            if _PRESERVED_KEYS.search(key) is not None:
                cmd_env[key] = val

    # When you run sudo <command>, it looks at the /etc/sudoers config file.
    # Some args are passed through, and some are set to explicit values.
    # My /etc/sudoers appears to explicitly set path to this value.
    cmd_env["PATH"] = SUDO_PATH

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
