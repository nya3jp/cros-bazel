# rules_ebuild

This is an experiment to build Portage packages with Bazel.

```
$ bazel build //third_party/portage-stable/sys-apps/ethtool
$ tar tvf bazel-bin/third_party/portage-stable/sys-apps/ethtool/ethtool-4.13.tbz2
drwxr-xr-x root/root         0 2022-08-12 14:41 ./
drwxr-xr-x root/root         0 2022-08-12 14:41 ./usr/
drwxr-xr-x root/root         0 2022-08-12 14:41 ./usr/sbin/
-rwxr-xr-x root/root    332224 2022-08-12 14:41 ./usr/sbin/ethtool
drwxr-xr-x root/root         0 2022-08-12 14:41 ./usr/share/
drwxr-xr-x root/root         0 2022-08-12 14:41 ./usr/share/doc/
drwxr-xr-x root/root         0 2022-08-12 14:41 ./usr/share/doc/ethtool-4.13/
-rw-r--r-- root/root       127 2022-08-12 14:41 ./usr/share/doc/ethtool-4.13/README
-rw-r--r-- root/root       249 2022-08-12 14:41 ./usr/share/doc/ethtool-4.13/AUTHORS.bz2
-rw-r--r-- root/root      4010 2022-08-12 14:41 ./usr/share/doc/ethtool-4.13/ChangeLog.bz2
-rw-r--r-- root/root      5250 2022-08-12 14:41 ./usr/share/doc/ethtool-4.13/NEWS.bz2
drwxr-xr-x root/root         0 2022-08-12 14:41 ./usr/share/man/
drwxr-xr-x root/root         0 2022-08-12 14:41 ./usr/share/man/man8/
-rw-r--r-- root/root      7876 2022-08-12 14:41 ./usr/share/man/man8/ethtool.8.bz2
```
