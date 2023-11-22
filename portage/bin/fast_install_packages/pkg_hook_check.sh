#!/bin/bash
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -eu

# Ensure that all pkg functions do nothing.
for f in {,pre_,post_}pkg_{setup,preinst,postinst}; do
  case "$( (declare -f "${f}" || :) | tr -s '[:space:]' ' ')" in
  "")
    # Undefined.
    ;;
  "${f} () { return } "|"${f} () { : } ")
    # Empty.
    ;;
  "${f} () { cros_stack_hooks ${f} } ")
    # Simple cros_stack_hooks call. This function can be ignored if all stacked
    # hook functions defined are known to be safe to ignore.
    ;;
  'pkg_setup () { cros-workon_pkg_setup "$@" } ')
    # cros-workon_pkg_setup does nothing for binary packages.
    ;;
  'pkg_postinst () { cros-go_pkg_postinst "$@" } ')
    # cros-go_pkg_postinst does nothing for binary packages.
    ;;
  *)
    echo "${f} has a custom definition:"
    declare -f "${f}"
    exit 1
  esac
done

# Check if there is any stacked hook that is not known to be safe to ignore.
for p in cros_{pre_,post_}pkg_{setup,preinst,postinst}_; do
  (compgen -A function "${p}" || :) | while read -r f; do
    case "${f}" in
    cros_pre_pkg_setup_sysroot_build_bin_dir)
      # Just sets $PATH, so can be skipped if there are no other meaningful
      # hooks.
      ;;
    *)
      echo "${f} is defined."
      exit 1
    esac
  done
done

echo "OK: hooks can be ignored"
exit 0
