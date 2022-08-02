# Copyright 1999-2020 Gentoo Authors
# Distributed under the terms of the GNU General Public License v2

# @ECLASS: optfeature.eclass
# @MAINTAINER:
# base-system@gentoo.org
# @BLURB: Advertise optional functionality that might be useful to users

case ${EAPI:-0} in
	[0-7]) ;;
	*)     die "Unsupported EAPI=${EAPI} (unknown) for ${ECLASS}" ;;
esac

if [[ -z ${_OPTFEATURE_ECLASS} ]]; then
_OPTFEATURE_ECLASS=1

# Stub this out for CrOS as it's just annoying log spam for us.
optfeature() { :; }

fi
