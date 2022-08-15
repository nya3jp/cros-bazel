#!/bin/bash -ex

# HACK: Print all outputs to stderr to avoid shuffled logs in Bazel output.
if [[ $# -gt 0 ]]; then
  exec >&2
fi

mkdir -p /build/target/etc/portage
# TODO: Avoid hard-coding the default profile path.
ln -sf /stage/overlays/chromiumos-overlay/profiles/default/linux/amd64/10.0/sdk /etc/portage/make.profile
ln -sf /stage/overlays/chromiumos-overlay/profiles/default/linux/amd64/10.0/sdk /build/target/etc/portage/make.profile

# HACK: Do not modify directory owners.
# TODO: Consider using fakeroot-like approach to emulate file permissions.
sed -i -e '/dir_mode_map = {/,/}/s/False/True/' /usr/lib/python3.6/site-packages/portage/package/ebuild/config.py

# HACK: Do not use namespaces in ebuild(1).
# TODO: Find a better way.
sed -i "/keywords\['unshare/d" /usr/lib/python3.6/site-packages/portage/package/ebuild/doebuild.py

read -ra atoms <<<"${INSTALL_ATOMS_HOST}"
if (( ${#atoms[@]} )); then
  # TODO: emerge is too slow! Find a way to speed up.
  time ROOT=/ SYSROOT=/ PORTAGE_CONFIGROOT=/ PKGDIR="${PKGDIR_HOST}" emerge --oneshot --usepkgonly --nodeps "${atoms[@]}"
fi

read -ra atoms <<<"${INSTALL_ATOMS_TARGET}"
if (( ${#atoms[@]} )); then
  # TODO: emerge is too slow! Find a way to speed up.
  time emerge --oneshot --usepkgonly --nodeps "${atoms[@]}"
fi

if [[ $# = 0 ]]; then
  exec bash
fi
exec "$@"
