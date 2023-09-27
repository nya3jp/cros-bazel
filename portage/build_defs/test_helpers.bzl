# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Helper functions for anyone trying to write tests for their starlark functions.

visibility("public")

def assert_eq(got, want, msg = "Got %r, want %r"):
    if got != want:
        fail(msg % (got, want))

def assert_ne(got, bad, msg = "Got %r, wanted any value other than that"):
    if got == bad:
        fail(msg % got)

def assert_not_none(got, msg = "Got None, wanted a non-None value"):
    if got == None:
        fail(msg)
