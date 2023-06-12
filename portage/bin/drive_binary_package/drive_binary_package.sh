#!/bin/bash
#
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# A shell script called by fast_install_packages to drive a binary package
# hooks.

################################################################################
# Parse command line arguments
################################################################################

__dbp_print_usage_and_exit() {
  echo "usage: $0 -r ROOT_DIR -d IMAGE_DIR -p CPF [-v] PHASES" >&2
  exit 2
}

__dbp_root_dir=
__dbp_image_dir=
__dbp_cpf=
__dbp_verbose=0
while getopts "r:d:p:v" OPTNAME; do
  case "${OPTNAME}" in
    r) __dbp_root_dir="${OPTARG}";;
    d) __dbp_image_dir="${OPTARG}";;
    p) __dbp_cpf="${OPTARG}";;
    v) __dbp_verbose=1;;
    *) __dbp_print_usage_and_exit;;
  esac
done

shift $(( OPTIND - 1 ))
unset OPTNAME OPTARG OPTIND

if [[ -z "${__dbp_root_dir}" || -z "${__dbp_image_dir}" || -z "${__dbp_cpf}" ]]
then
  __dbp_print_usage_and_exit
fi

if [[ "$#" == 0 ]]; then
  __dbp_print_usage_and_exit
fi

__dbp_vdb_dir="${__dbp_root_dir}/var/db/pkg/${__dbp_cpf}"

################################################################################
# Define ebuild functions
################################################################################

die() {
  echo "FATAL: $1" >&2
  exit 1
}

has() {
  local target="$1"
  shift
  local value
  for value in "$@"; do
    if [[ "${value}" = "${target}" ]]; then
      return 0
    fi
  done
  return 1
}

hasv() {
  if has "$@"; then
    echo "$1"
    return 0
  fi
  return 1
}

hasq() {
  has "$@"
}

use() {
  local name="$1"
  if [[ "${name}" == "!"* ]]; then
    name="${name#!}"
    # shellcheck disable=SC2086
    ! has "${name}" ${USE?}
  else
    # shellcheck disable=SC2086
    has "${name}" ${USE?}
  fi
}

usev() {
  local cond="$1"
  local text="$2"
  if [[ -z "${text}" ]]; then
    text="${cond#!}"
  fi
  if use "${cond}"; then
    echo "${text}"
  fi
}

useq() {
  use "$@"
}

usex() {
  if use "$1"; then
    echo "${2-yes}$4"
  else
    echo "${3-no}$5"
  fi
}

use_with() {
  local name="${2-${1#!}}"
  local suffix="${3+=}$3"
  usex "$1" "--with-${name}${suffix}" "--without-${name}${suffix}"
}

use_enable() {
  local name="${2-${1#!}}"
  local suffix="${3+=}$3"
  usex "$1" "--enable-${name}${suffix}" "--disable-${name}${suffix}"
}

in_iuse() {
  # shellcheck disable=SC2046
  has "$1" $(cat "${__dbp_vdb_dir?}/IUSE_EFFECTIVE")
}

debug-print() {
  : # TODO: Implement
}

debug-print-function() {
  local name="$1"
  shift
  debug-print "${name}: entering function" "$@"
}

debug-print-section() {
  debug-print "now in section $*"
}

einfo() {
  echo "${CATEGORY?}/${PF?}: INFO: $*" >&2
}

einfon() {
  echo -n "${CATEGORY?}/${PF?}: INFO: $*" >&2
}

elog() {
  einfo "$@"
}

ewarn() {
  echo "${CATEGORY?}/${PF?}: WARN: $*" >&2
}

eqawarn() {
  ewarn "$@"
}

eerror() {
  echo "${CATEGORY?}/${PF?}: ERROR: $*" >&2
}

ebegin() {
  local msg="$*"
  einfo "${msg} ..."
}

eend() {
  local code="${1-0}"
  shift
  if [[ "${code}" -eq 0 ]]; then
    einfo "ok."
  else
    eerror "$@"
  fi
  return "${code}"
}

has_version() {
  # TODO: Avoid depending on qlist.
  [[ -n "$(qlist -eIqC "$1")" ]] 2> /dev/null
}

best_version() {
  # TODO: Avoid depending on qlist.
  qlist -evIqC "$1" 2> /dev/null
}

# ver_cut, ver_rs, ver_test are included in environment.

################################################################################
# Define private functions
################################################################################

# Dumps the environment to the standard output.
__dbp_dump_environment() {
  # Start a subshell to avoid rewriting variables.
  (
    # Unset all variables starting with __dbp_.
    # shellcheck disable=SC2046
    unset -f $(compgen -A function __dbp_)
    # shellcheck disable=SC2086
    unset ${!__dbp_*}

    # Dump variables.
    declare -p
    declare -fp
  )
}

__dbp_set_up_temporary_dir() {
  __dbp_temp_dir="/tmp/drive_binary_package/${__dbp_cpf}"
  trap 'rm -rf "${__dbp_temp_dir}"' EXIT
}

__dbp_define_vars() {
  # Define PMS variables.
  # See 11.1 Defined Variables for the list of defined in the environment.
  # https://projects.gentoo.org/pms/8/pms.html#x1-10900011.1

  # Whether directory paths end with a slash differs by EAPI.
  # https://projects.gentoo.org/pms/8/pms.html#x1-11300011.1.4
  export ROOT="${__dbp_root_dir?}"
  export D="${__dbp_image_dir?}"

  case "${EAPI?}" in
  0|1|2|3|4|5|6)
    ROOT="${ROOT%/}/"
    D="${D%/}/"
    ;;
  *)
    ROOT="${ROOT%/}"
    D="${D%/}"
    ;;
  esac

  export FILESDIR="/.filesdir-unavailable"
  export DISTDIR="/.distdir-unavailable"
  export WORKDIR="/.workdir-unavailable"
  export EROOT="${ROOT}"
  export SYSROOT="${ROOT}"
  export ESYSROOT="${ROOT}"
  export BROOT=""
  export T="${__dbp_temp_dir?}"
  export TMPDIR="/tmp"
  export HOME="/.home-unavailable"
  export EPREFIX=""
  export ED="${D?}"
  export MERGE_TYPE="binary"
  export REPLACING_VERSIONS=""

  # Define Portage-specific variables.
  export EBUILD="${__dbp_vdb_dir?}/${PF?}.ebuild"
  export EMERGE_FROM="binary"
  export PORTAGE_ACTUAL_DISTDIR="${DISTDIR}"
  export PORTAGE_BUILD_GROUP="root"
  export PORTAGE_BUILD_USER="root"
  export PORTAGE_BUILDDIR="${T?}"
  export PORTAGE_CONFIGROOT="${ROOT}"
  export PORTAGE_GRPNAME="root"
  export PORTAGE_REPO_NAME
  PORTAGE_REPO_NAME="$(cat "${__dbp_vdb_dir?}/repository")"
  export PORTAGE_TMPDIR="${TMPDIR?}"
  export PORTAGE_USERNAME="root"
  export TEMP="${TMPDIR?}"
  export TMP="${TMPDIR?}"
}

################################################################################
# Main
################################################################################

__dbp_main() {
  if [[ "${__dbp_verbose}" = 1 ]]; then
    set -x
  fi

  __dbp_set_up_temporary_dir
  __dbp_define_vars

  export EBUILD_PHASE EBUILD_PHASE_FUNC

  for EBUILD_PHASE in "$@"; do
    for EBUILD_PHASE_FUNC in {pre_,,post_}"pkg_${EBUILD_PHASE}"; do
      if [[ $(type -t "${EBUILD_PHASE_FUNC}") == function ]]; then
        echo "${CATEGORY}/${PF}: Running ${EBUILD_PHASE_FUNC}" >&2
        "${EBUILD_PHASE_FUNC}"
      fi
    done
  done
}

# Load the environment in the global scope.
# shellcheck disable=SC1090
source <(bzip2 -dc "${__dbp_vdb_dir}/environment.bz2")

__dbp_main "$@"

# Save the environment.
__dbp_dump_environment | bzip2 -zc > "${__dbp_vdb_dir}/environment.bz2"
