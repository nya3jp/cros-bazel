# rules_ebuild

This is an experiment to build Portage packages with Bazel.

```
$ bazel build //third_party/portage-stable/sys-apps/ethtool
$ tar tvf bazel-bin/third_party/portage-stable/sys-apps/ethtool/ethtool-4.13.tbz2
drwxr-xr-x root/root         0 2022-08-17 01:29 ./
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/sbin/
-rwxr-xr-x root/root    249664 2022-08-17 01:29 ./usr/sbin/ethtool
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/lib/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/lib/debug/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/lib/debug/usr/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/lib/debug/usr/sbin/
-rw-r--r-- root/root    566192 2022-08-17 01:29 ./usr/lib/debug/usr/sbin/ethtool.debug
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/lib/debug/.build-id/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/lib/debug/.build-id/dd/
lrwxrwxrwx root/root         0 2022-08-17 01:29 ./usr/lib/debug/.build-id/dd/5fcefbca703539 -> /usr/sbin/ethtool
lrwxrwxrwx root/root         0 2022-08-17 01:29 ./usr/lib/debug/.build-id/dd/5fcefbca703539.debug -> ../../usr/sbin/ethtool.debug
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/share/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/share/doc/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/share/doc/ethtool-4.13/
-rw-r--r-- root/root       127 2022-08-17 01:29 ./usr/share/doc/ethtool-4.13/README
-rw-r--r-- root/root      4344 2022-08-17 01:29 ./usr/share/doc/ethtool-4.13/ChangeLog.gz
-rw-r--r-- root/root       224 2022-08-17 01:29 ./usr/share/doc/ethtool-4.13/AUTHORS.gz
-rw-r--r-- root/root      6185 2022-08-17 01:29 ./usr/share/doc/ethtool-4.13/NEWS.gz
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/share/man/
drwxr-xr-x root/root         0 2022-08-17 01:29 ./usr/share/man/man8/
-rw-r--r-- root/root      8777 2022-08-17 01:29 ./usr/share/man/man8/ethtool.8.gz
```

## Prerequisites

You need several binaries in your host environment to make the build work.

TODO: Build these binaries with Bazel and get rid of host dependencies.

```
sudo apt install squashfs-tools libsquashfs-dev
```
