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

### Missing files despite correct dependency declarations

**Cause**: Bazel by default will build packages with [Interface Library]
dependencies.

We perform the following modifications to the build-time dependencies that are
"installed" into the ephemeral chroot that is used to build the package:

*   All executables in `/bin`, `/usr/bin`, etc will be omitted.
*   All static libraries (`.a`) in `/lib`, `/usr/lib`, etc will be omitted.
*   All shared libraries (`.so`) in `/lib`, `/usr/lib`, etc will be stripped of
    all code leaving only the public interface.
*   All debug symbols will be omitted.
*   `/usr/share/{doc,info,man}` will be omitted.

**Symptom**: Any of the files listed above are missing.

**Solutions**:

1.  If the dependency only produces static libraries or doesn't benefit from
    creating interface layers, set the `generate_interface_libraries` property
    to `false` in the dependency's [Bazel-specific metadata].
2.  If the dependency produces static helper libraries that always need to be
    linked, in addition to shared objects, there are still benefits from using
    interface libraries. You can add the static libraries to the
    `interface_library_allowlist` in the dependency's [Bazel-specific metadata]
    to keep them in the interface layer.
3.  If your package needs all its dependencies in their pristine form because
    it's bundling them or using qemu to execute the binaries, set the
    `supports_interface_libraries` property to `false` in your package's
    [Bazel-specific metadata].
4.  If you control the dependency, evaluate switching to generating shared
    objects instead of static libraries.

**Example fixes**:

-   [Adding interface library metadata](https://crrev.com/c/5571562)

[Interface Library]: ./advanced.md#interface-libraries

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

### Dump Alchemist's view of a package

It can be useful to see what Alchemist understands about a package to see why
the Bazel build is behaving a certain way. You can use this command to dump
this information for a given package:

`bazel run //:alchemist -- --board ${BOARD} dump-package ${PACKAGE}`

or

`bazel run //:alchemist -- --host dump-package ${PACKAGE}`

Example:

```
$ bazel run //:alchemist -- --board amd64-generic dump-package sys-apps/portage
Path:        /mnt/host/source/src/third_party/chromiumos-overlay/sys-apps/portage/portage-2.3.75-r161.ebuild
Package:    sys-apps/portage
Version:    2.3.75-r161 (Default)
Slot:        0/0
Stable:        true
Readiness:        OK (not masked)
USE:        +abi_x86_64 -alpha +amd64 -amd64-fbsd -amd64-linux -arm -arm-linux -arm64 -build -cros_host -cros_workon_tree_e947dfa7fd9f6280412b8b58481e9dcc226465ca -doc -elibc_AIX -elibc_Cygwin -elibc_Darwin -elibc_DragonFly -elibc_FreeBSD -elibc_HPUX -elibc_Interix -elibc_NetBSD -elibc_OpenBSD -elibc_SunOS -elibc_Winnt -elibc_bionic +elibc_glibc -elibc_mingw -elibc_mintlib -elibc_musl -elibc_uclibc -epydoc -gentoo-dev -hppa -hppa-hpux -ia64 -ia64-hpux -ia64-linux +ipc -kernel_AIX -kernel_Darwin -kernel_FreeBSD -kernel_HPUX -kernel_NetBSD -kernel_OpenBSD -kernel_SunOS -kernel_Winnt -kernel_freemint +kernel_linux -m68k -m68k-mint -mips +native-extensions -nios2 -ppc -ppc-aix -ppc-macos -ppc-openbsd -ppc64 -ppc64-linux -prefix -prefix-guest -prefix-stack -python_targets_python3_6 -python_targets_python3_7 +python_targets_python3_8 -python_targets_python3_9 -riscv -rsync-verify -s390 -selinux -sh -sparc -sparc-fbsd -sparc-solaris -sparc64-freebsd -sparc64-solaris -userland_BSD +userland_GNU -x64-cygwin -x64-freebsd -x64-macos -x64-openbsd -x64-solaris -x86 -x86-cygwin -x86-fbsd -x86-freebsd -x86-interix -x86-linux -x86-macos -x86-netbsd -x86-openbsd -x86-solaris -x86-winnt +xattr
BDEPEND:
  app-misc/jq-1.7_pre20201109-r1::portage-stable
  dev-lang/python-3.8.16_p4-r5::chromiumos
  dev-lang/python-exec-2.4.6-r4::portage-stable
  dev-python/setuptools-44.0.0::portage-stable
  dev-vcs/git-2.39.2-r1::portage-stable
IDEPEND:
DEPEND:
  app-arch/tar-1.34-r3::portage-stable
  dev-lang/python-3.8.16_p4-r5::chromiumos
  dev-lang/python-3.8.16_p4-r5::chromiumos
  dev-lang/python-exec-2.4.6-r4::portage-stable
  dev-lang/python-exec-2.4.6-r4::portage-stable
  sys-apps/sed-4.8::portage-stable
  sys-devel/patch-2.7.6-r5::portage-stable
RDEPEND:
  app-arch/tar-1.34-r3::portage-stable
  app-misc/pax-utils-1.3.3::portage-stable
  app-shells/bash-5.1_p16-r2::portage-stable
  dev-lang/python-3.8.16_p4-r5::chromiumos
  dev-lang/python-exec-2.4.6-r4::portage-stable
  dev-lang/python-exec-2.4.6-r4::portage-stable
  sys-apps/install-xattr-0.8-r1::portage-stable
  sys-apps/sed-4.8::portage-stable
PDEPEND:
  net-misc/rsync-3.2.7-r2::portage-stable
  sys-apps/coreutils-8.32-r2::portage-stable
```

You can also use the `--env` flag to dump the environment variables for the
package, which can be useful for viewing information such as USE flags.

### Bad cache results when non-hermetic inputs change

Bazel is able to correctly reuse content from the cache when all inputs are
identified to it so it can detect when they change. Since our toolchain and our
host tools (e.g. gsutil) are not yet fully hermetic, it's possible that you'll
run into problems when tools not yet tracked by Bazel are updated. In these
situations we've found it useful to run `bazel clean --expunge` to clear cached
artifacts that seem not to be cleared without the `--expunge` flag.

If you find you need the `--expunge` flag, please file a bug to let the
Bazelification team know about the non-hermeticity so we can fix the problem.
