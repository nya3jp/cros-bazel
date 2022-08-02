# Copyright (c) 2013 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# @ECLASS: user.eclass
# @MAINTAINER:
# The Chromium OS Authors. <chromium-os-dev@chromium.org>
# @BLURB: user management in ebuilds
# @DESCRIPTION:
# Replaces the upstream mechanism of managing users and groups with one that
# manages the database in ${ROOT}, changing the sysroot database
# only when the caller creates the user/group during setup.

# Before we manipulate users at all, we want to make sure that
# passwd/group/shadow is initialized in the first place. That's
# what baselayout does.
#
# We should consider providing a virtual to abstract away this dependency.
# This would allow CrOS builds to completely specify all users and groups,
# instead of accepting the assumption (expressed in baselayout, currently)
# that every build wants groups like wheel, tty and so forth.
if [ "${PN}" != "baselayout" ]; then
	DEPEND="sys-apps/baselayout"
	RDEPEND="sys-apps/baselayout"
fi

# @FUNCTION: _is_cros_device
# @INTERNAL
# @USAGE:
# @DESCRIPTION:
# Used to figure out if we're running on a Chromium OS device.
_is_cros_device() {
	grep -qs ^CHROMEOS_RELEASE_ /etc/lsb-release
}

# @FUNCTION: _assert_pkg_ebuild_phase
# @INTERNAL
# @USAGE: <calling func name>
_assert_pkg_ebuild_phase() {
	case ${EBUILD_PHASE} in
	setup|preinst|postinst) ;;
	*)
		eerror "'$1()' called from '${EBUILD_PHASE}' phase which is not OK:"
		eerror "You may only call from pkg_{setup,preinst,postinst} functions."
		eerror "Package fails at QA and at life.  Please file a bug."
		die "Bad package!  $1 is only for use in some pkg_* functions!"
	esac
}

# Array of paths to existing accounts DBs in overlays that apply for the
# current board. In order of decreasing overlay priority.
ACCOUNTS_DIRS=()

# @FUNCTION: _find_accounts_dirs
# @INTERNAL
# @USAGE:
# @DESCRIPTION:
# Looks for accounts DB under all valid overlays for the current board and
# populates ACCOUNTS_DIRS with fully-qualified paths to all the ones it finds.
# Values are cached, so multiple calls will return quickly without updating
# global ACCOUNTS_DIRS array.
_find_accounts_dirs() {
	[[ ${#ACCOUNTS_DIRS[@]} -gt 0 ]] && return

	# Load the cache from disk.  We don't want to use the env because that
	# will be saved at build time and used when merging binpkgs.  Instead,
	# we need it to be generated at binpkg time too.
	local cache="${T}/_accounts_dir_cache.list"
	if [[ -s ${cache} ]]; then
		einfo "Using cached list ${cache}"
		local dir
		while read -d $'\0' -r dir; do
			einfo "Adding ${dir} to user/group search path."
			ACCOUNTS_DIRS+=("${dir}")
		done <"${cache}"
		return
	fi

	local overlay
	local root="${SYSROOT:-${ROOT:-/}}"
	for overlay in $(_call_portageq get_repos "${root}"); do
		local overlay_dir=$(_call_portageq get_repo_path "${root}" "${overlay}")
		local accounts_dir="${overlay_dir}/profiles/base/accounts"
		if [[ -d "${accounts_dir}" ]] ; then
			einfo "Adding ${accounts_dir} to user/group search path."
			ACCOUNTS_DIRS+=("${accounts_dir}")
		fi
	done

	printf '%s\0' "${ACCOUNTS_DIRS[@]}" >"${cache}"
}

# @FUNCTION: _call_portageq
# @INTERNAL
# @USAGE: <portageq command> [<arg> ...]
_call_portageq() {
	echo $(env -i \
		PATH="${PATH}" \
		LC_ALL="en_US.UTF-8" \
		PORTAGE_USERNAME="${PORTAGE_USERNAME}" \
		PORTAGE_CONFIGROOT="${PORTAGE_CONFIGROOT}" \
		portageq "$@")
}

# @FUNCTION: _find_acct_template
# @INTERNAL
# @USAGE: <db> <key>
# @DESCRIPTION:
# Searches existing account dbs in overlay-inheritance order for <key> in <db>.
_find_acct_template() {
	local db=$1 key=$2
	[[ $# -ne 2 ]] && die "usage: ${FUNCNAME} <db> <key>"
	local accounts_dir
	for accounts_dir in "${ACCOUNTS_DIRS[@]}" ; do
		local template="${accounts_dir}/${db}/${key}"
		if  [[ -e "${template}" ]]; then
			echo "${template}"
			return
		fi
	done

	# If we're still here, something has gone wrong.
	eerror "ACCOUNTS_DIRS = ${ACCOUNTS_DIRS[*]}"
}

# @FUNCTION: _read_db_entry
# @INTERNAL
# @USAGE: <template> <key>
# @DESCRIPTION:
# Read the value from the template database.
_read_db_entry() {
	local template=$1 key=$2
	awk -F':' -v key="${key}" '$1 == key { print $2 }' "${template}"
}

# @FUNCTION: _get_value_for_user
# @INTERNAL
# @USAGE: <user> <key>
# @DESCRIPTION:
# Gets value from appropriate account definition file.
_get_value_for_user() {
	local user=$1 key=$2
	[[ $# -ne 2 ]] && die "usage: ${FUNCNAME} <user> <key>"
	[[ ${#ACCOUNTS_DIRS[@]} -eq 0 ]] && die "Must populate ACCOUNTS_DIRS!"

	case ${key} in
	user|password|uid|gid|gecos|home|shell|defunct) ;;
	*) die "sorry, '${key}' is not a field in the passwd db." ;;
	esac

	local template=$(_find_acct_template user "${user}")
	[[ -z "${template}" ]] && die "No entry for ${user} in any overlay."
	_read_db_entry "${template}" "${key}"
}

# @FUNCTION: _get_value_for_group
# @INTERNAL
# @USAGE: <group> <key>
# @DESCRIPTION:
# Gets value from appropriate account definition file.
_get_value_for_group() {
	local group=$1 key=$2
	[[ $# -ne 2 ]] && die "usage: ${FUNCNAME} <group> <key>"
	[[ ${#ACCOUNTS_DIRS[@]} -eq 0 ]]  && die "Must populate ACCOUNTS_DIRS!"

	case ${key} in
	group|password|gid|users|defunct) ;;
	*) die "sorry, '${key}' is not a field in the group db." ;;
	esac

	local template=$(_find_acct_template group "${group}")
	[[ -z "${template}" ]] && die "No entry for ${group} in any overlay."
	_read_db_entry "${template}" "${key}"
}

# @FUNCTION: _assert_fields_in_sync
# @INTERNAL
# @USAGE: <user|group> <account> <keys>
# @DESCRIPTION:
# Walks all the overlays and makes sure that the keys have the same values in
# all of them.  This is useful for making sure uids/gids don't change in case
# the account name has a collision.
_assert_fields_in_sync() {
	local db=$1 acct=$2 keys=( "${@:3}" )
	local key dir old_dir
	for key in "${keys[@]}"; do
		local value old_value=""
		for dir in "${ACCOUNTS_DIRS[@]}"; do
			local template="${dir}/${db}/${acct}"
			if [[ -e "${template}" ]]; then
				value=$(_read_db_entry "${template}" "${key}")
				if [[ "${old_value:=${value}}" != "${value}" ]]; then
					eerror "${db} account '${acct}' has conflicting ${key} values."
					eerror "${template}: ${key} = ${value}"
					eerror "${old_dir}/${db}/${acct}: ${key} = ${old_value}"
					die "${key} must be kept in sync across overlays"
				fi
				old_dir="${dir}"
			fi
		done
	done
}

# @FUNCTION: _portable_grab_lock
# @INTERNAL
# @USAGE: <lockfile>
# @DESCRIPTION:
# Grabs a lock on <lockfile> in a race-free, portable manner.
# We need to use this mechanism in order to be compatible with the shadow utils
# (groupadd, useradd, etc).
_portable_grab_lock() {
	local lockfile=$1
	local lockfile_1="${lockfile}.${BASHPID}"
	local timeout=$(( 60 * 5 )) # 5 minute timeout

	touch "${lockfile_1}"
	until ln "${lockfile_1}" "${lockfile}" &> /dev/null; do
		sleep 1
		[[ $(( timeout-- )) -le 0 ]] && die "Timeout while trying to lock ${lockfile}"
		[[ $(( timeout % 10 )) -eq 0 ]] && einfo "Waiting for lock on ${dbfile}"
	done
	rm "${lockfile_1}" || die "Failed to lock ${lockfile}."
}

# @FUNCTION: _write_entry_to_db()
# @INTERNAL
# @USAGE: <entry> <database> <root>
# @DESCRIPTION:
# Writes an entry to the specified database under the specified root.
# Supported databases: passwd group shadow
_write_entry_to_db() {
	local entry=$1 db=$2 root=$3
	[[ $# -ne 3 ]] && die "usage: _write_entry_to_db <entry> <database> <root>"

	case ${db} in
	passwd|group|shadow) ;;
	*) die "sorry, database '${db}' not supported." ;;
	esac

	local dbfile=$(readlink -e "${root}/etc/${db}")
	[[ ! -e "${dbfile}" ]] && die "${db} under ${root} does not exist."
	if [[ ! -w "${dbfile}" ]] ; then
		ewarn "Unable to modify ${db} under ${root} due to read-only mount."
		return 1
	fi
	 # Use the same lock file as the shadow utils.
	local lockfile="${dbfile}.lock"

	_portable_grab_lock "${lockfile}"

	# Need to check if the acct exists while we hold the lock, in case
	# another ebuild added it in the meantime.
	local key=$(awk -F':' '{ print $1 }' <<<"${entry}")
	local existing_entry=$(egetent --nolock "${db}" "${key}" "${root}")
	if [[ -z ${existing_entry} ]] ; then
		echo "${entry}" >> "${dbfile}" || die "Could not write ${entry} to ${dbfile}."
	else
		einfo "'${entry}' superceded by '${existing_entry}'"
	fi

	rm "${lockfile}" || die "Failed to release lock on ${lockfile}."
	return 0
}

# @FUNCTION: egetent
# @USAGE: [--nolock] <database> <key> [root]
# @DESCRIPTION:
# Provides getent-like functionality for databases under [root]. Defaults to ${ROOT}.
#
# Supported databases: group passwd shadow
egetent() {
	local use_lock=true
	if [[ $1 == "--nolock" ]]; then
		use_lock=false
		shift
	fi

	if [[ $# -ne 2 && $# -ne 3 ]]; then
		die "usage: egetent <database> <key> [root]"
	fi

	local db=$1 key=$2 root=${3:-"${ROOT}"}

	case ${db} in
	passwd|group|shadow) ;;
	*) die "sorry, database '${db}' not yet supported; file a bug" ;;
	esac

	local dbfile=$(readlink -e "${root}/etc/${db}")
	[[ ! -e "${dbfile}" ]] && die "${db} under ${root} does not exist."
	[[ ! -w "${dbfile}" ]] && use_lock=false  # File can't change anyway!

	local lockfile="${dbfile}.lock"
	${use_lock} && _portable_grab_lock "${lockfile}"

	awk -F':' -v key="${key}" \
		'($1 == key || $3 == key) { print }' \
		"${dbfile}" 2>/dev/null

	if ${use_lock} ; then
		rm "${lockfile}" || die "Failed to release lock on ${lockfile}."
	fi
}

# @FUNCTION: enewuser
# @USAGE: <user> [uid] [shell] [homedir] [groups]
# @DESCRIPTION:
# Same as enewgroup, you are not required to understand how to properly add
# a user to the system.  The only required parameter is the username.
# Default uid is (pass -1 for this) next available, default shell is
# /bin/false, default homedir is /dev/null, and there are no default groups.
enewuser() {
	_assert_pkg_ebuild_phase ${FUNCNAME}

	# get the username
	local euser=$1; shift
	if [[ -z ${euser} ]] ; then
		eerror "No username specified !"
		die "Cannot call enewuser without a username"
	fi

	# Lets see if the username already exists in ${ROOT} or in the system.
	local is_in_root=false
	if [[ -n "$(egetent passwd "${euser}")" ]]; then
		is_in_root=true
	fi

	local is_in_system=false
	if [[ -n "$(egetent passwd "${euser}" /)" ]]; then
		is_in_system=true
	fi

	local should_be_in_system=false
	if [[ "${EBUILD_PHASE}" == "setup" ]]; then
		should_be_in_system=true
	fi

	# We can't support creating accounts on the system yet.
	# https://crbug.com/402673
	if _is_cros_device; then
		ewarn "Skipping user '${euser}' creation due to https://crbug.com/402673"
		return 0
	fi

	# Locate all applicable accounts profiles.
	local ACCOUNTS_DIRS
	_find_accounts_dirs
	if [[ ${#ACCOUNTS_DIRS[@]} -eq 0 ]] ; then
		ewarn "No user/group data files present. Skipping."
		return 0
	fi

	# Check if user entry requires password (has a password of "x").
	# If so, check if shadow file already contains an entry for the user.
	# About passwords in shadow files: src/third_party/eclass-overlay/profiles/base/accounts/README.md
	local epassword=$(_get_value_for_user "${euser}" password)
	: "${epassword:="!"}"
	local should_have_shadow_entry=false
	local is_in_shadow=false
	if [[ ${epassword} == "x" ]]; then
		should_have_shadow_entry=true

		# Make sure shadow file exists.
		if [[ ! -f "${ROOT}/etc/shadow" ]]; then
			touch "${ROOT}/etc/shadow" || die
		fi

		if [[ -n "$(egetent shadow "${euser}")" ]]; then
			is_in_shadow=true
		fi
	fi

	if "${is_in_root}" &&
			(! "${should_have_shadow_entry}" || "${is_in_shadow}") &&
			(! "${should_be_in_system}" || "${is_in_system}") ; then
		return 0
	fi

	# Ensure username exists in profile.
	if [[ -z $(_get_value_for_user "${euser}" user) ]] ; then
		die "'${euser}' does not exist in profile!"
	elif [[ -n $(_get_value_for_user "${euser}" defunct) ]] ; then
		die "'${euser}' was used previously and is now disallowed."
	fi
	_assert_fields_in_sync user "${euser}" user uid gid
	einfo "Adding user '${euser}' to your system ..."

	# Handle uid. Passing no UID is functionally equivalent to passing -1.
	local provided_uid=$(_get_value_for_user "${euser}" uid)
	local euid=$1; shift
	if [[ "${PORTAGE_REPO_NAME}" == "portage-stable" ]] ; then
		# If caller is from portage-stable, ignore specified UID.
		if [[ ${euid:--1} != "-1" ]] ; then
			einfo "Ignoring requested UID ${euid} in portage-stable ebuilds."
		fi
		euid=''
	fi
	if [[ -z ${euid} ]] ; then
		euid=-1
	elif [[ ${euid} -lt -1 ]] ; then
		eerror "Userid given but is not greater than 0 !"
		die "${euid} is not a valid UID."
	fi
	# Now, ${euid} is set and >= -1.
	if [[ -n ${provided_uid} ]] ; then
		# If profile has UID and caller specified '' or -1, use profile.
		# If profile has UID and caller specified different, barf.
		# If profile has UID and caller specified same, OK.
		if [[ ${euid} == -1 ]] ; then
			euid=${provided_uid}
		elif [[ ${euid} != "${provided_uid}" ]] ; then
			eerror "Userid differs from the profile!"
			die "${euid} != ${provided_uid} from profile."
			# else...they're already equal, so do nothing.
		fi
	else
		# If profile has no UID and caller did not specify, barf.
		if [[ ${euid} == -1 ]] ; then
			die "No UID specified in profile!"
		fi
		# If profile has no entry w/UID and caller specified one, OK.
	fi

	einfo " - Userid: ${euid}"

	# See if there's a provided gid and use it if so.
	local provided_gid=$(_get_value_for_user "${euser}" gid)
	local egid=${provided_gid:-${euid}}
	einfo " - Groupid: ${egid}"

	# handle shell
	local eshell=$1; shift
	if [[ -n ${eshell} && ${eshell} != "-1" ]] ; then
		# We might need to relax this for portage-stable if there
		# are any packages that we want to allow to set a custom shell.
		eerror "Do not specify ${eshell} yourself, use -1"
		die "Pass '-1' as the shell parameter"
	else
		eshell=$(_get_value_for_user "${euser}" shell)
		${eshell:=/bin/false}
	fi
	if [[ ${eshell} != */false && ${eshell} != */nologin ]] ; then
		if [[ ! -e ${ROOT}${eshell} ]] ; then
			eerror "A shell was specified but it does not exist !"
			die "${eshell} does not exist in ${ROOT}"
		fi
	fi
	einfo " - Shell: ${eshell}"

	# handle homedir
	local ehome=$1; shift
	if [[ ${ehome:--1} != "-1" ]] ; then
		if [[ "${PORTAGE_REPO_NAME}" != "portage-stable" ]] ; then
			die "Pass -1 as the home directory"
		else
			# If caller is from portage-stable, ignore specified homedir.
			einfo "Ignoring requested homedir ${ehome} in portage-stable ebuilds."
			ehome=''
		fi
	fi
	if [[ -z ${ehome} || ${ehome} == "-1" ]] ; then
		ehome=$(_get_value_for_user "${euser}" home)
	fi
	einfo " - Home: ${ehome}"

	# Grab groups for later handling.
	local egroups=$1; shift

	# Check groups.
	local g egroups_arr
	IFS="," read -r -a egroups_arr <<<"${egroups}"
	shift
	for g in "${egroups_arr[@]}" ; do
		enewgroup "${g}"
	done
	einfo " - Groups: ${egroups:-(none)}"

	local comment
	if [[ $# -gt 0 ]] ; then
		die "extra arguments no longer supported; please file a bug."
	else
		comment=$(_get_value_for_user "${euser}" gecos)
		einfo " - GECOS: ${comment}"
	fi

	local entry="${euser}:${epassword}:${euid}:${egid}:${comment}:${ehome}:${eshell}"
	local sentry="${euser}:x:::::::"

	if ! "${is_in_system}" && "${should_be_in_system}" ; then
		_write_entry_to_db "${entry}" passwd / || die "Must be able to add users during setup."
	fi

	local is_in_system_shadow=false

	# Make sure shadow file exists.
	if [[ ! -f "/etc/shadow" ]]; then
		touch "/etc/shadow" || die
	fi

	if [[ -n "$(egetent shadow "${euser}" /)" ]]; then
		is_in_system_shadow=true
	fi

	if ! "${is_in_system_shadow}" && "${should_be_in_system}" && "${should_have_shadow_entry}" ; then
		_write_entry_to_db "${sentry}" shadow / || die "Must be able to add users during setup."
	fi

	if ! "${is_in_root}" ; then
		if _write_entry_to_db "${entry}" passwd "${ROOT}" ; then
			if [[ ! -e ${ROOT}/${ehome} ]] ; then
				einfo " - Creating ${ehome} in ${ROOT}"
				mkdir -p "${ROOT}/${ehome}"
				chown "${euser}" "${ROOT}/${ehome}"
				chmod 755 "${ROOT}/${ehome}"
			fi
		fi
		if "${should_have_shadow_entry}" ; then
			_write_entry_to_db "${sentry}" shadow "${ROOT}"
		fi
	fi
}

# @FUNCTION: enewgroup
# @USAGE: <group> [gid]
# @DESCRIPTION:
# This function does not require you to understand how to properly add a
# group to the system.  Just give it a group name to add and enewgroup will
# do the rest.  You may specify the gid for the group or allow the group to
# allocate the next available one.
enewgroup() {
	_assert_pkg_ebuild_phase ${FUNCNAME}

	# Get the group.
	local egroup=$1; shift
	if [[ -z ${egroup} ]] ; then
		eerror "No group specified !"
		die "Cannot call enewgroup without a group"
	fi

	# Lets see if the group already exists in ${ROOT} or in the system.
	local is_in_root=false
	[[ -n "$(egetent group "${egroup}")" ]] && is_in_root=true
	local is_in_system=false
	[[ -n "$(egetent group "${egroup}" /)" ]] && is_in_system=true
	local should_be_in_system=false
	[[ "${EBUILD_PHASE}" == "setup" ]] && should_be_in_system=true

	if "${is_in_root}" && (! "${should_be_in_system}" || "${is_in_system}") ; then
		return 0
	fi

	# We can't support creating accounts on the system yet.
	# https://crbug.com/402673
	if _is_cros_device; then
		ewarn "Skipping group '${egroup}' creation due to https://crbug.com/402673"
		return 0
	fi

	# Locate all applicable accounts profiles.
	local ACCOUNTS_DIRS
	_find_accounts_dirs
	if [[ ${#ACCOUNTS_DIRS[@]} -eq 0 ]] ; then
		ewarn "No user/group data files present. Skipping."
		return 0
	fi
	# Ensure group exists in profile.
	if [[ -z $(_get_value_for_group "${egroup}" group) ]] ; then
		die "Config for ${egroup} not present in profile!"
	elif [[ -n $(_get_value_for_group "${egroup}" defunct) ]] ; then
		die "'${egroup}' was used previously and is now disallowed."
	fi
	_assert_fields_in_sync group "${egroup}" group gid
	einfo "Adding group '${egroup}' to your system ..."

	# handle gid
	local provided_gid=$(_get_value_for_group "${egroup}" gid)
	local egid=$1; shift
	if [[ "${PORTAGE_REPO_NAME}" == "portage-stable" ]] ; then
		# If caller is from portage-stable, ignore specified GID.
		if [[ ${egid:--1} != "-1" ]] ; then
			einfo "Ignoring requested GID ${egid} in portage-stable ebuilds."
		fi
		egid=''
	fi
	if [[ -z ${egid} ]] ; then
		# If caller specified nothing and profile has GID, use profile.
		# If caller specified nothing and profile has no GID, barf.
		if [[ -n  ${provided_gid} ]] ; then
			egid=${provided_gid}
		else
			die "No gid provided in PROFILE or in args!"
		fi
	else
		if [[ ${egid} -lt 0 ]] ; then
			eerror "Groupid given but is not greater than 0 !"
			die "${egid} is not a valid GID"
		fi

		# If caller specified GID and profile has no GID, OK.
		# If caller specified GID and profile has entry with same, OK.
		if [[ -z ${provided_gid} || ${egid} -eq ${provided_gid} ]] ; then
			provided_gid=${egid}
		fi

		# If caller specified GID but profile has different, barf.
		if [[ ${egid} -ne ${provided_gid} ]] ; then
			eerror "${egid} conflicts with provided ${provided_gid}!"
			die "${egid} conflicts with provided ${provided_gid}!"
		fi
	fi
	einfo " - Groupid: ${egid}"

	# Handle extra.
	if [[ $# -gt 0 ]] ; then
		die "extra arguments no longer supported; please file a bug"
	fi

	# Allow group passwords, if profile asks for it.
	local epassword=$(_get_value_for_group "${egroup}" password)
	: "${epassword:="!"}"
	einfo " - Password entry: ${epassword}"

	# Pre-populate group with users.
	local eusers=$(_get_value_for_group "${egroup}" users)
	einfo " - User list: ${eusers}"

	# Add the group.
	local entry="${egroup}:${epassword}:${egid}:${eusers}"
	if ! "${is_in_system}" && "${should_be_in_system}" ; then
		_write_entry_to_db "${entry}" group / || die "Must be able to add groups during setup."
	fi
	if ! "${is_in_root}" ; then
		_write_entry_to_db "${entry}" group "${ROOT}"
	fi
	einfo "Done with group: '${egroup}'."
}

# @FUNCTION: egethome
# @USAGE: <user>
# @DESCRIPTION:
# Gets the home directory for the specified user.
egethome() {
	[[ $# -eq 1 ]] || die "usage: egethome <user>"
	egetent passwd "$1" | cut -d: -f6
}

# @FUNCTION: egetshell
# @USAGE: <user>
# @DESCRIPTION:
# Gets the shell for the specified user.
egetshell() {
	[[ $# -eq 1 ]] || die "usage: egetshell <user>"
	egetent passwd "$1" | cut -d: -f7
}
