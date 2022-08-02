# rules_ebuild

This is an experiment to build Portage packages with Bazel.

```
$ bazel build //portage-stable/sys-apps/ethtool
$ unsquashfs -l bazel-bin/portage-stable/sys-apps/ethtool/ethtool-4.13.squashfs
squashfs-root
squashfs-root/usr
squashfs-root/usr/sbin
squashfs-root/usr/sbin/ethtool
squashfs-root/usr/share
squashfs-root/usr/share/doc
squashfs-root/usr/share/doc/ethtool-4.13
squashfs-root/usr/share/doc/ethtool-4.13/AUTHORS.bz2
squashfs-root/usr/share/doc/ethtool-4.13/ChangeLog.bz2
squashfs-root/usr/share/doc/ethtool-4.13/NEWS.bz2
squashfs-root/usr/share/doc/ethtool-4.13/README
squashfs-root/usr/share/man
squashfs-root/usr/share/man/man8
squashfs-root/usr/share/man/man8/ethtool.8.bz2
```
