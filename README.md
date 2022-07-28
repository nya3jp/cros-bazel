# rules_ebuild

This is an experiment to build Portage packages with Bazel.

```
$ bazel build //example/experimental/hello
$ file bazel-bin/example/experimental/hello/hello-1.0.tbz2
bazel-bin/example/experimental/hello/hello-1.0.tbz2: Gentoo binary package (XPAK)
```
