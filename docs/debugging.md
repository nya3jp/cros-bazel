# Debugging Build Issues

[TOC]

## Asking for help

If you are unsure how to resolve build errors in Bazel-orchestrated builds,
please send an email to chromeos-build-discuss@google.com.

## Common build issues

### Build-time package dependencies are missing

**Cause**:
Only explicitly declared build-time dependency packages are made available in
the ephemeral CrOS SDK container when building a Portage package under Bazel.

**Symptom**:
Missing build-time package dependencies result in a variety of error messages,
including:

- `foobar: command not found`
- `No such file or directory: 'foobar'`
- `Package foobar was not found in the pkg-config search path.`
- `'path/to/foobar.h' file not found`
- `unable to find library -lfoobar`
- `Program 'foobar' not found or not executable`
- `import error: No module named 'foobar'`
- `no matching package named foobar found`
- `cannot find package "foobar" in any of:`

**Solution**:
Make sure you declare proper `DEPEND`/`BDEPEND` in your ebuild/eclasses.

**Example fixes**:
- [Adding a missing DEPEND](https://crrev.com/c/4840362)
- [Adding a missing BDEPEND](https://crrev.com/c/4983365)

### Implicit build-time dependencies are missing

**Cause**:
Ebuilds/eclasses are prohibited to access ChromeOS source checkout via
`/mnt/host/source` unless those dependencies are explicitly declared with
`CROS_WORKON_*`.

**Symptom**:
Implicit build-time dependencies result in a variety of error messages,
including `foobar: command not found`.

**Solution**:
Declare extra sources in [Bazel-specific metadata].

[Bazel-specific metadata]: ./advanced.md#declaring-bazel_specific-ebuild_eclass-metadata

**Example fixes**:
TBD

### Uses sudo

**Cause**:
`sudo` doesn't work in the ephemeral CrOS SDK container used to build Portage
packages as it is unprivileged. In fact, `/usr/bin/sudo` is replaced with
a fake script that just executes the specified command.

**Symptom**:
If your package attempts to run `sudo`, the following message will be printed
to the standard error:

```
fake_sudo: INFO: This is the fake sudo for the ephemeral CrOS SDK.
```

This message doesn't mean an immediate failure, but the subsequent process will
run unprivileged.

**Solution**:
Do not use `sudo` in the package build.

If your package uses `platform2_test.py` to run foreign-architecture
executables on build, pass `--strategy=unprivileged` to run the script without
sudo.

**Example fixes**:
- [Passing `--strategy=unprivileged` to platform2_test.py](https://crrev.com/c/4683119)

## How to debug build errors

### Entering ephemeral CrOS SDK containers

Sometimes you want to enter an ephemeral CrOS SDK container where a package
build is failing to inspect the environment interactively.

To enter an ephemeral CrOS SDK container, run the following command:

```
$ BOARD=amd64-generic bazel run @portage//target/sys-apps/attr:debug -- --login=after
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
