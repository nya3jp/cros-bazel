# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Unit tests for the fake sudo."""

from typing import Mapping
import unittest

from bazel.portage.sdk.usr.bin import fake_sudo


# Stop unittest from truncating the error messages.
# pylint: disable=protected-access
unittest.util._MAX_LENGTH = 999999999

_ENV_PLAIN = {
    "USER": "root",
    "MYVAR": "myval",
    "PATH": "old_path",
}


def _merge_envs(*envs: Mapping[str, str]):
    merged_env = {}
    for env in envs:
        merged_env.update(env)
    return merged_env


class FakeSudoTest(unittest.TestCase):
    """Unit tests for the fake sudo."""

    def test_simple(self):
        self.assertEqual(
            fake_sudo.parse(["echo", "a"], env=_ENV_PLAIN),
            fake_sudo.Cmd(args=["echo", "a"], env=fake_sudo.ENV_FORCED),
        )

    def test_dashed(self):
        self.assertEqual(
            fake_sudo.parse(["--", "echo", "a"], env=_ENV_PLAIN),
            fake_sudo.Cmd(args=["echo", "a"], env=fake_sudo.ENV_FORCED),
        )

    def test_env(self):
        self.assertEqual(
            fake_sudo.parse(["A=b", "echo", "a"], env=_ENV_PLAIN),
            fake_sudo.Cmd(
                args=["echo", "a"],
                env=_merge_envs(fake_sudo.ENV_FORCED, {"A": "b"}),
            ),
        )

    def test_user(self):
        self.assertEqual(
            fake_sudo.parse(["-u", "root", "echo", "a"], env=_ENV_PLAIN),
            fake_sudo.Cmd(args=["echo", "a"], env=fake_sudo.ENV_FORCED),
        )

    def test_stops_early(self):
        self.assertEqual(
            fake_sudo.parse(["ls", "-u", "."], env=_ENV_PLAIN),
            fake_sudo.Cmd(args=["ls", "-u", "."], env=fake_sudo.ENV_FORCED),
        )

    def test_unknown_arg(self):
        with self.assertRaises(NotImplementedError):
            fake_sudo.parse(["--unknown", "echo", "a"], env=_ENV_PLAIN)

    def test_no_args(self):
        with self.assertRaises(ValueError):
            fake_sudo.parse(["-E", "--"], env=_ENV_PLAIN)

    def test_preserve_env(self):
        self.assertEqual(
            fake_sudo.parse(["-E", "echo", "a"], env=_ENV_PLAIN),
            fake_sudo.Cmd(
                args=["echo", "a"],
                env=_merge_envs(_ENV_PLAIN, fake_sudo.ENV_FORCED),
            ),
        )
        self.assertEqual(
            fake_sudo.parse(["-E", "A=b", "echo", "a"], env=_ENV_PLAIN),
            fake_sudo.Cmd(
                args=["echo", "a"],
                env=_merge_envs(_ENV_PLAIN, fake_sudo.ENV_FORCED, {"A": "b"}),
            ),
        )

    def test_complex(self):
        self.assertEqual(
            fake_sudo.parse(
                ["--user=root", "-E", "A=b", "--", "echo", "a"], env=_ENV_PLAIN
            ),
            fake_sudo.Cmd(
                args=["echo", "a"],
                env=_merge_envs(_ENV_PLAIN, fake_sudo.ENV_FORCED, {"A": "b"}),
            ),
        )


if __name__ == "__main__":
    unittest.main()
