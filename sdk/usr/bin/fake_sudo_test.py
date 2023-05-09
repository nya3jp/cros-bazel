# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

from typing import Mapping, Optional, Sequence
import unittest

from cros.bazel.sdk.usr.bin import fake_sudo


# Stop unittest from truncating the error messages.
unittest.util._MAX_LENGTH = 999999999

_ENV = dict(USER="chronos", MYVAR="myval", PATH="old_path")
_BASE_ENV = dict(USER="chronos", PATH=fake_sudo.SUDO_PATH)


def layered_env(*args: Mapping[str, str], **kwargs: str):
    result = {}
    for arg in args:
        result.update(arg)
    result.update(kwargs)
    return result


class FakeSudoTest(unittest.TestCase):
    def assert_cmd_matches(
        self,
        args: Sequence[str],
        expected_args: Sequence[str],
        expected_env: Mapping[str, str],
    ):
        self.assertEqual(
            fake_sudo.parse(args, env=_ENV),
            fake_sudo.Cmd(args=expected_args, env=expected_env),
        )

    def test_simple(self):
        self.assertEqual(
            fake_sudo.parse(["echo", "a"], env=_ENV),
            fake_sudo.Cmd(args=["echo", "a"], env=_BASE_ENV),
        )

    def test_dashed(self):
        self.assertEqual(
            fake_sudo.parse(["--", "echo", "a"], env=_ENV),
            fake_sudo.Cmd(args=["echo", "a"], env=_BASE_ENV),
        )

    def test_env(self):
        self.assertEqual(
            fake_sudo.parse(["A=b", "echo", "a"], env=_ENV),
            fake_sudo.Cmd(
                args=["echo", "a"], env=layered_env(_BASE_ENV, A="b")
            ),
        )

    def test_user(self):
        self.assertEqual(
            fake_sudo.parse(["-u", "root", "echo", "a"], env=_ENV),
            fake_sudo.Cmd(args=["echo", "a"], env=_BASE_ENV),
        )

    def test_stops_early(self):
        self.assertEqual(
            fake_sudo.parse(["ls", "-u", "."], env=_ENV),
            fake_sudo.Cmd(args=["ls", "-u", "."], env=_BASE_ENV),
        )

    def test_unknown_arg(self):
        with self.assertRaises(NotImplementedError):
            fake_sudo.parse(["--unknown", "echo", "a"], env=_ENV)

    def test_no_args(self):
        with self.assertRaises(ValueError):
            fake_sudo.parse(["-E", "--"], env=_ENV)

    def test_preserve_env(self):
        self.assertEqual(
            fake_sudo.parse(["-E", "echo", "a"], env=_ENV),
            fake_sudo.Cmd(args=["echo", "a"], env=layered_env(_ENV, _BASE_ENV)),
        )
        self.assertEqual(
            fake_sudo.parse(["-E", "A=b", "echo", "a"], env=_ENV),
            fake_sudo.Cmd(
                args=["echo", "a"], env=layered_env(_ENV, _BASE_ENV, A="b")
            ),
        )

    def test_complex(self):
        self.assertEqual(
            fake_sudo.parse(
                ["--user=root", "-E", "A=b", "--", "echo", "a"], env=_ENV
            ),
            fake_sudo.Cmd(
                args=["echo", "a"], env=layered_env(_ENV, _BASE_ENV, A="b")
            ),
        )


if __name__ == "__main__":
    unittest.main()
