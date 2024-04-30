#!/bin/bash
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# This script contains unit tests for functions provided in the ebuild
# environment. Define unit test cases as test_*, then they will be run
# automatically in pkg_setup.

assert() {
  if [[ "$1" == "!" ]]; then
    shift
    "$@" && die "FAIL: ${*@Q} succeeded, but should have failed"
  else
    "$@" || die "FAIL: ${*@Q} failed but should have succeeded"
  fi
}

test_has_version() {
  assert has_version pkg/aaa
  assert ! has_version pkg/bbb
  assert has_version pkg/ccc

  assert has_version -r pkg/aaa
  assert ! has_version -r pkg/bbb
  assert has_version -r pkg/ccc

  assert has_version -d pkg/aaa
  assert ! has_version -d pkg/bbb
  assert has_version -d pkg/ccc

  assert has_version --host-root pkg/aaa
  assert has_version --host-root pkg/bbb
  assert ! has_version --host-root pkg/ccc

  assert has_version -b pkg/aaa
  assert has_version -b pkg/bbb
  assert ! has_version -b pkg/ccc
}

test_best_version() {
  assert test "$(best_version pkg/aaa)" == "pkg/aaa-1.2.3"
  assert test -z "$(best_version pkg/bbb)"
  assert test "$(best_version pkg/ccc)" == "pkg/ccc-3.4.5"

  assert test "$(best_version -r pkg/aaa)" == "pkg/aaa-1.2.3"
  assert test -z "$(best_version -r pkg/bbb)"
  assert test "$(best_version -r pkg/ccc)" == "pkg/ccc-3.4.5"

  assert test "$(best_version -d pkg/aaa)" == "pkg/aaa-1.2.3"
  assert test -z "$(best_version -d pkg/bbb)"
  assert test "$(best_version -d pkg/ccc)" == "pkg/ccc-3.4.5"

  assert test "$(best_version --host-root pkg/aaa)" == "pkg/aaa-1.2.3"
  assert test "$(best_version --host-root pkg/bbb)" == "pkg/bbb-2.3.4"
  assert test -z "$(best_version --host-root pkg/ccc)"

  assert test "$(best_version -b pkg/aaa)" == "pkg/aaa-1.2.3"
  assert test "$(best_version -b pkg/bbb)" == "pkg/bbb-2.3.4"
  assert test -z "$(best_version -b pkg/ccc)"
}

test_keepdir() {
  set -x
  keepdir /foo

  ls -alh "${ED}/foo"
  assert test -f "${ED}/foo/.keep_${CATEGORY}_${PN}_${SLOT}"
}

pkg_setup() {
  local cases=() f
  # Gather a list of functions prefixed with "test_".
  readarray -t cases < <(compgen -A function test_)
  for f in "${cases[@]}"; do
    echo "Running ${f}..." >&2
    "${f}"
  done
}
