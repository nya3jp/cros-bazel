# Debugging Build Issues

[TOC]

## How to debug build errors

### Entering ephemeral CrOS SDK containers

Sometimes you want to enter an ephemeral CrOS SDK container where a package
build is failing to inspect the environment interactively.

To enter an ephemeral CrOS SDK container, run the following command:

```
$ BOARD=arm64-generic bazel run @portage//target/sys-apps/attr:debug -- --login=after
```

This command will give you an interactive shell after building a package.
You can also specify other values to `--login` to choose the timing to enter
an interactive console:

- `--login=before`: before building the package
- `--login=after`: after building the package (default)
- `--login=after-fail`: after failing to build the package

### Bad cache results when non-hermetic inputs change

Bazel is able to correctly reuse content from the cache when all inputs are
identified to it so it can detect when they change. Since our toolchain and our
host tools (e.g. gsutil) are not yet fully hermetic, it's possible that you'll
run into problems when tools not yet tracked by Bazel are updated. In these
situations we've found it useful to run `bazel clean --expunge` to clear cached
artifacts that seem not to be cleared without the `--expunge` flag.

If you find you need the `--expunge` flag, please file a bug to let the
Bazelification team know about the non-hermeticity so we can fix the problem.

## Common build errors

TODO: Write this section.
