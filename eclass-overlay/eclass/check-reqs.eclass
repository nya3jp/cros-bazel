# Copyright 1999-2019 Gentoo Foundation
# Distributed under the terms of the GNU General Public License v2

# This is a stub eclass to disable these checks in CrOS.  We don't generally
# care about these, or need to worry that the system is low on resources.
#
# We only provide stubs for the funcs that appear to be used in the tree rather
# than every function the eclass exports.

if [[ ! ${_CHECK_REQS_ECLASS_} ]]; then

check-reqs_pkg_setup() { :; }
check-reqs_pkg_pretend() { :; }

_CHECK_REQS_ECLASS_=1
fi
