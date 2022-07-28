# rules_ebuild

This is an experiment to build Portage packages with Bazel.

```
$ bazel build //example/experimental/ethtool
$ file bazel-bin/example/experimental/ethtool/ethtool-4.13.tbz2
bazel-bin/example/experimental/ethtool/ethtool-4.13.tbz2: Gentoo binary package (XPAK)
```
