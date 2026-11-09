#!/bin/bash -ue

# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

if [[ ! -e /etc/cros_chroot_version ]]; then
  echo "Cannot setup board outside the cros SDK chroot."
  exit 1
fi

print_usage_and_exit() {
  exec >&2
  echo "usage: $0 [flags]"
  echo "   -b board - Name of the board"
  echo "   -o output_file - Output file location."
  echo "   --toolchain-pkg category:pkg_name:src_path -" \
       "Toolchain binary package to stage"
  exit 1
}

BOARD=""
OUTPUT=""
TOOLCHAIN_PKGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -b|--board)
      BOARD="$2"
      shift 2
      ;;
    -o|--output)
      OUTPUT="$2"
      shift 2
      ;;
    --toolchain-pkg)
      TOOLCHAIN_PKGS+=("$2")
      shift 2
      ;;
    *)
      print_usage_and_exit
      ;;
  esac
done

if [[ -z "${BOARD}" || -z "${OUTPUT}" ]]; then
  print_usage_and_exit
fi

# `src_unpack` will run as `${PORTAGE_USERNAME}` if specified, or `portage`
# otherwise. If we don't set this variable, we get UID mismatches between the
# `.git` repos and the user running `src_unpack`.
#
# Normally `PORTAGE_USERNAME` is set when entering the chroot, but since we
# are executing as a bazel action, the environment has been stripped of all
# variables.
#
# See b/394378820#comment22.
export PORTAGE_USERNAME
PORTAGE_USERNAME="$(id --user --name)"

# Run setup_board FIRST so that it initializes configurations and wipes
# any old sysroot contents via --force before we stage new packages.
/mnt/host/source/chromite/bin/setup_board \
  -b "${BOARD}" \
  --reuse-configs \
  --force \
  --skip-chroot-upgrade

# Stage toolchain packages AFTER setup_board completes.
if [[ ${#TOOLCHAIN_PKGS[@]} -gt 0 ]]; then
  declare -A MANIFESTS
  for item in "${TOOLCHAIN_PKGS[@]}"; do
    IFS=":" read -r category pkg_name src_path <<< "${item}"
    tc_dir="/build/${BOARD}/var/cache/toolchain-pkgs/${category}"
    sudo mkdir -p "${tc_dir}"
    sudo chown -R "${PORTAGE_USERNAME}:portage" \
      "/build/${BOARD}/var/cache/toolchain-pkgs"

    src_filename="$(basename "${src_path}")"
    dest_filename="${src_filename/.partial.tbz2/.tbz2}"
    dest_path="${tc_dir}/${dest_filename}"
    cp "${src_path}" "${dest_path}"

    # Accumulate manifest entries: category -> {pkg_name: dest_filename}
    prev="${MANIFESTS["${category}"]:-}"
    if [[ -z "${prev}" ]]; then
      prev="{}"
    fi
    MANIFESTS["${category}"]="$(jq -n \
      --argjson prev "${prev}" \
      --arg pkg "${pkg_name}" \
      --arg file "${dest_filename}" \
      '$prev + {($pkg): $file}')"
  done

  for category in "${!MANIFESTS[@]}"; do
    tc_dir="/build/${BOARD}/var/cache/toolchain-pkgs/${category}"
    tmp_manifest="${tc_dir}/manifest.json.tmp.$$"
    echo "${MANIFESTS[${category}]}" > "${tmp_manifest}"
    mv "${tmp_manifest}" "${tc_dir}/manifest.json"
  done
fi

touch "${OUTPUT}"
