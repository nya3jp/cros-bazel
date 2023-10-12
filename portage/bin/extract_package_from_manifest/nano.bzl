# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# AUTO GENERATED DO NOT EDIT!
# Regenerate this file using the following command:
# bazel run @@//bazel/portage/bin/extract_package_from_manifest:nano_update
# However, you should never need to run this unless
# bazel explicitly tells you to.

# These three lines ensures that the following json is valid skylark.
false = False
true = True
null = None

NANO_MANIFEST_CONTENT = {
    "root_package": {
        "name": "sys-libs/glibc",
        "slot": "2.2",
    },
    "packages": [
        {
            "name": "app-editors/nano",
            "slot": "0",
            "content": {
                "/bin/nano": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libncursesw.so.6": "/usr/lib64/libncursesw.so.6",
                        "libtinfow.so.6": "/usr/lib64/libtinfow.so.6",
                    },
                },
                "/bin/rnano": {
                    "type": "Symlink",
                    "target": "/bin/nano",
                },
                "/etc/nanorc": {},
                "/usr/share/doc/nano-6.4/AUTHORS.gz": {},
                "/usr/share/doc/nano-6.4/ChangeLog.gz": {},
                "/usr/share/doc/nano-6.4/NEWS.gz": {},
                "/usr/share/doc/nano-6.4/README.gz": {},
                "/usr/share/doc/nano-6.4/THANKS.gz": {},
                "/usr/share/doc/nano-6.4/TODO": {},
                "/usr/share/doc/nano-6.4/html/faq.html": {},
                "/usr/share/doc/nano-6.4/sample.nanorc.gz": {},
                "/usr/share/info/nano.info.gz": {},
                "/usr/share/man/man1/nano.1.gz": {},
                "/usr/share/man/man1/rnano.1.gz": {},
                "/usr/share/man/man5/nanorc.5.gz": {},
                "/usr/share/nano/ada.nanorc": {},
                "/usr/share/nano/asm.nanorc": {},
                "/usr/share/nano/autoconf.nanorc": {},
                "/usr/share/nano/awk.nanorc": {},
                "/usr/share/nano/c.nanorc": {},
                "/usr/share/nano/changelog.nanorc": {},
                "/usr/share/nano/cmake.nanorc": {},
                "/usr/share/nano/css.nanorc": {},
                "/usr/share/nano/default.nanorc": {},
                "/usr/share/nano/elisp.nanorc": {},
                "/usr/share/nano/email.nanorc": {},
                "/usr/share/nano/fortran.nanorc": {},
                "/usr/share/nano/gentoo.nanorc": {},
                "/usr/share/nano/go.nanorc": {},
                "/usr/share/nano/groff.nanorc": {},
                "/usr/share/nano/guile.nanorc": {},
                "/usr/share/nano/haskell.nanorc": {},
                "/usr/share/nano/html.nanorc": {},
                "/usr/share/nano/java.nanorc": {},
                "/usr/share/nano/javascript.nanorc": {},
                "/usr/share/nano/json.nanorc": {},
                "/usr/share/nano/lua.nanorc": {},
                "/usr/share/nano/makefile.nanorc": {},
                "/usr/share/nano/man.nanorc": {},
                "/usr/share/nano/markdown.nanorc": {},
                "/usr/share/nano/nanohelp.nanorc": {},
                "/usr/share/nano/nanorc.nanorc": {},
                "/usr/share/nano/nftables.nanorc": {},
                "/usr/share/nano/objc.nanorc": {},
                "/usr/share/nano/ocaml.nanorc": {},
                "/usr/share/nano/patch.nanorc": {},
                "/usr/share/nano/perl.nanorc": {},
                "/usr/share/nano/php.nanorc": {},
                "/usr/share/nano/po.nanorc": {},
                "/usr/share/nano/povray.nanorc": {},
                "/usr/share/nano/python.nanorc": {},
                "/usr/share/nano/ruby.nanorc": {},
                "/usr/share/nano/rust.nanorc": {},
                "/usr/share/nano/sh.nanorc": {},
                "/usr/share/nano/spec.nanorc": {},
                "/usr/share/nano/sql.nanorc": {},
                "/usr/share/nano/tcl.nanorc": {},
                "/usr/share/nano/tex.nanorc": {},
                "/usr/share/nano/texinfo.nanorc": {},
                "/usr/share/nano/xml.nanorc": {},
                "/usr/share/nano/yaml.nanorc": {},
            },
        },
        {
            "name": "sys-libs/glibc",
            "slot": "2.2",
            "content": {
                "/etc/env.d/00glibc": {},
                "/etc/gai.conf": {},
                "/etc/host.conf": {},
                "/etc/locale.gen": {},
                "/etc/nsswitch.conf": {},
                "/etc/rpc": {},
                "/lib": {
                    "type": "Symlink",
                    "target": "/lib64",
                },
                "/lib32/ld-linux.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib32/libBrokenLocale.so.1": {},
                "/lib32/libanl.so.1": {},
                "/lib32/libc.so.6": {},
                "/lib32/libc_malloc_debug.so.0": {},
                "/lib32/libdl.so.2": {},
                "/lib32/libm.so.6": {},
                "/lib32/libmemusage.so": {},
                "/lib32/libnsl.so.1": {},
                "/lib32/libnss_compat.so.2": {},
                "/lib32/libnss_db.so.2": {},
                "/lib32/libnss_dns.so.2": {},
                "/lib32/libnss_files.so.2": {},
                "/lib32/libnss_hesiod.so.2": {},
                "/lib32/libpcprofile.so": {},
                "/lib32/libpthread.so.0": {},
                "/lib32/libresolv.so.2": {},
                "/lib32/librt.so.1": {},
                "/lib32/libthread_db.so.1": {},
                "/lib32/libutil.so.1": {},
                "/lib64/ld-linux-x86-64.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/ld-linux.so.2": {
                    "type": "Symlink",
                    "target": "/lib32/ld-linux.so.2",
                },
                "/lib64/ld-lsb-x86-64.so.3": {
                    "type": "Symlink",
                    "target": "/lib64/ld-linux-x86-64.so.2",
                },
                "/lib64/libBrokenLocale.so.1": {
                    "type": "SharedLibrary",
                },
                "/lib64/libanl.so.1": {
                    "type": "SharedLibrary",
                },
                "/lib64/libc.so.6": {
                    "type": "SharedLibrary",
                },
                "/lib64/libc_malloc_debug.so.0": {
                    "type": "SharedLibrary",
                },
                "/lib64/libdl.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/libm.so.6": {
                    "type": "SharedLibrary",
                },
                "/lib64/libmemusage.so": {
                    "type": "SharedLibrary",
                },
                "/lib64/libmvec.so.1": {
                    "type": "SharedLibrary",
                },
                "/lib64/libnsl.so.1": {
                    "type": "SharedLibrary",
                },
                "/lib64/libnss_compat.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/libnss_db.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/libnss_dns.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/libnss_files.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/libnss_hesiod.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/libpcprofile.so": {
                    "type": "SharedLibrary",
                },
                "/lib64/libpthread.so.0": {
                    "type": "SharedLibrary",
                },
                "/lib64/libresolv.so.2": {
                    "type": "SharedLibrary",
                },
                "/lib64/librt.so.1": {
                    "type": "SharedLibrary",
                },
                "/lib64/libthread_db.so.1": {
                    "type": "SharedLibrary",
                },
                "/lib64/libutil.so.1": {
                    "type": "SharedLibrary",
                },
                "/sbin/ldconfig": {},
                "/sbin/sln": {},
                "/usr/bin/gencat": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/getconf": {
                    "type": "Symlink",
                    "target": "/usr/lib64/misc/glibc/getconf/POSIX_V7_LP64_OFF64",
                },
                "/usr/bin/getent": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/iconv": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/ld.so": {
                    "type": "Symlink",
                    "target": "/lib64/ld-linux-x86-64.so.2",
                },
                "/usr/bin/ldd": {},
                "/usr/bin/lddlibc4": {
                    "type": "ElfBinary",
                    "interp": "/lib32/ld-linux.so.2",
                    "libs": {
                        "libc.so.6": "/lib32/libc.so.6",
                    },
                },
                "/usr/bin/locale": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/localedef": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/makedb": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/mtrace": {},
                "/usr/bin/pcprofiledump": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/pldd": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/sotruss": {},
                "/usr/bin/sprof": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/bin/xtrace": {},
                "/usr/include/a.out.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/aio.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/aliases.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/alloca.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ar.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/argp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/argz.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/arpa/ftp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/arpa/inet.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/arpa/nameser.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/arpa/nameser_compat.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/arpa/telnet.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/arpa/tftp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/assert.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/a.out.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/argp-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/atomic_wide_counter.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/byteswap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/cmathcalls.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/confname.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/cpu-set.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/dirent.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/dirent_ext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/dl_find_object.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/dlfcn.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/elfclass.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/endian.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/endianness.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/environments.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/epoll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/err-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/errno.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/error-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/error.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/eventfd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/fcntl-linux.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/fcntl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/fcntl2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/fenv.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/floatn-common.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/floatn.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/flt-eval-method.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/fp-fast.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/fp-logb.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/getopt_core.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/getopt_ext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/getopt_posix.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/hwcap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/in.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/indirect-return.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/initspin.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/inotify.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/ioctl-types.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/ioctls.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/ipc-perm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/ipc.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/ipctypes.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/iscanonical.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/libc-header-start.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/libm-simd-decl-stubs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/link.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/link_lavcurrent.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/local_lim.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/locale.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/long-double.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/math-vector.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mathcalls-helper-functions.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mathcalls-narrow.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mathcalls.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mathdef.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mman-linux.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mman-map-flags-generic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mman-shared.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mman.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/monetary-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mqueue.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/mqueue2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/msq.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/netdb.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/param.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/platform/x86.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/poll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/poll2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/posix1_lim.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/posix2_lim.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/posix_opt.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/printf-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/procfs-extra.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/procfs-id.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/procfs-prregset.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/procfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/pthread_stack_min-dynamic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/pthread_stack_min.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/pthreadtypes-arch.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/pthreadtypes.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/ptrace-shared.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/resource.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/rseq.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sched.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/select.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/select2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sem.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/semaphore.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/setjmp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/setjmp2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/shm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/shmlba.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sigaction.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sigcontext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sigevent-consts.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/siginfo-arch.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/siginfo-consts-arch.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/siginfo-consts.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/signal_ext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/signalfd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/signum-arch.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/signum-generic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sigstack.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sigstksz.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sigthread.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sockaddr.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/socket-constants.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/socket.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/socket2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/socket_type.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/ss_flags.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stab.def": {},
                "/usr/include/bits/stat.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/statfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/statvfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/statx-generic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/statx.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdint-intn.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdint-uintn.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdio-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdio.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdio2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdio_lim.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdlib-bsearch.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdlib-float.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdlib-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/stdlib.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/string_fortified.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/strings_fortified.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/struct_mutex.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/struct_rwlock.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/struct_stat.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/struct_stat_time64_helper.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/syscall.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/syslog-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/syslog-path.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/syslog.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/sysmacros.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-baud.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-c_cc.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-c_cflag.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-c_iflag.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-c_lflag.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-c_oflag.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-misc.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-struct.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios-tcflow.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/termios.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/thread-shared-types.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/time.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/time64.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/timerfd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/timesize.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/timex.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/FILE.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/__FILE.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/__fpos64_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/__fpos_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/__locale_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/__mbstate_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/__sigset_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/__sigval_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/clock_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/clockid_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/cookie_io_functions_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/error_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/locale_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/mbstate_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/res_state.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/sig_atomic_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/sigevent_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/siginfo_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/sigset_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/sigval_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/stack_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_FILE.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct___jmp_buf_tag.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_iovec.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_itimerspec.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_msqid64_ds.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_msqid64_ds_helper.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_msqid_ds.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_osockaddr.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_rusage.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_sched_param.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_semid64_ds.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_semid64_ds_helper.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_semid_ds.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_shmid64_ds.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_shmid64_ds_helper.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_shmid_ds.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_sigstack.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_statx.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_statx_timestamp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_timeb.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_timespec.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_timeval.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/struct_tm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/time_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/timer_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types/wint_t.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/types.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/typesizes.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/uintn-identity.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/uio-ext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/uio_lim.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/unistd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/unistd_ext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/utmp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/utmpx.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/utsname.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/waitflags.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/waitstatus.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/wchar-ldbl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/wchar.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/wchar2.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/wctype-wchar.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/wordsize.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/bits/xopen_lim.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/byteswap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/complex.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/cpio.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ctype.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/dirent.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/dlfcn.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/elf.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/endian.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/envz.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/err.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/errno.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/error.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/execinfo.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/fcntl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/features-time64.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/features.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/fenv.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/finclude/math-vector-fortran.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/fmtmsg.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/fnmatch.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/fpu_control.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/fstab.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/fts.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ftw.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gconv.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/getopt.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/glob.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu/lib-names-32.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu/lib-names-64.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu/lib-names.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu/libc-version.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu/stubs-32.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu/stubs-64.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu/stubs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gnu-versions.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/grp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/gshadow.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/iconv.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ieee754.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ifaddrs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/inttypes.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/langinfo.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/lastlog.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/libgen.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/libintl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/limits.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/link.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/locale.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/malloc.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/math.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/mcheck.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/memory.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/mntent.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/monetary.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/mqueue.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/ethernet.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/if.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/if_arp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/if_packet.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/if_ppp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/if_shaper.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/if_slip.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/ppp-comp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/ppp_defs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/net/route.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netash/ash.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netatalk/at.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netax25/ax25.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netdb.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/neteconet/ec.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/ether.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/icmp6.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/if_ether.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/if_fddi.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/if_tr.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/igmp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/in.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/in_systm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/ip.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/ip6.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/ip_icmp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/tcp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netinet/udp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netipx/ipx.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netiucv/iucv.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netpacket/packet.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netrom/netrom.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/netrose/rose.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/nfs/nfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/nl_types.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/nss.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/obstack.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/paths.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/poll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/printf.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/proc_service.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/protocols/routed.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/protocols/rwhod.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/protocols/talkd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/protocols/timed.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/pthread.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/pty.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/pwd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/re_comp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/regex.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/regexp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/resolv.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/rpc/netdb.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sched.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/scsi/scsi.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/scsi/scsi_ioctl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/scsi/sg.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/search.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/semaphore.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/setjmp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sgtty.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/shadow.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/signal.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/spawn.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/stab.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/stdc-predef.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/stdint.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/stdio.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/stdio_ext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/stdlib.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/string.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/strings.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/acct.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/auxv.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/bitypes.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/cdefs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/debugreg.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/dir.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/elf.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/epoll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/errno.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/eventfd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/fanotify.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/fcntl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/file.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/fsuid.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/gmon.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/gmon_out.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/inotify.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/io.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/ioctl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/ipc.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/kd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/klog.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/mman.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/mount.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/msg.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/mtio.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/param.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/pci.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/perm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/personality.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/platform/x86.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/poll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/prctl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/procfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/profil.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/ptrace.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/queue.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/quota.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/random.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/raw.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/reboot.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/reg.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/resource.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/rseq.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/select.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/sem.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/sendfile.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/shm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/signal.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/signalfd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/single_threaded.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/socket.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/socketvar.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/soundcard.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/stat.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/statfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/statvfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/swap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/syscall.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/sysinfo.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/syslog.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/sysmacros.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/termios.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/time.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/timeb.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/timerfd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/times.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/timex.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/ttychars.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/ttydefaults.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/types.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/ucontext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/uio.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/un.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/unistd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/user.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/utsname.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/vfs.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/vlimit.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/vm86.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/vt.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/wait.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sys/xattr.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/syscall.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/sysexits.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/syslog.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/tar.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/termio.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/termios.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/tgmath.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/thread_db.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/threads.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/time.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ttyent.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/uchar.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ucontext.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ulimit.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/unistd.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/utime.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/utmp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/utmpx.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/values.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/wait.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/wchar.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/wctype.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/wordexp.h": {
                    "type": "HeaderFile",
                },
                "/usr/lib32/Mcrt1.o": {},
                "/usr/lib32/Scrt1.o": {},
                "/usr/lib32/audit/sotruss-lib.so": {},
                "/usr/lib32/crt1.o": {},
                "/usr/lib32/crti.o": {},
                "/usr/lib32/crtn.o": {},
                "/usr/lib32/gconv/ANSI_X3.110.so": {},
                "/usr/lib32/gconv/ARMSCII-8.so": {},
                "/usr/lib32/gconv/ASMO_449.so": {},
                "/usr/lib32/gconv/BIG5.so": {},
                "/usr/lib32/gconv/BIG5HKSCS.so": {},
                "/usr/lib32/gconv/BRF.so": {},
                "/usr/lib32/gconv/CP10007.so": {},
                "/usr/lib32/gconv/CP1125.so": {},
                "/usr/lib32/gconv/CP1250.so": {},
                "/usr/lib32/gconv/CP1251.so": {},
                "/usr/lib32/gconv/CP1252.so": {},
                "/usr/lib32/gconv/CP1253.so": {},
                "/usr/lib32/gconv/CP1254.so": {},
                "/usr/lib32/gconv/CP1255.so": {},
                "/usr/lib32/gconv/CP1256.so": {},
                "/usr/lib32/gconv/CP1257.so": {},
                "/usr/lib32/gconv/CP1258.so": {},
                "/usr/lib32/gconv/CP737.so": {},
                "/usr/lib32/gconv/CP770.so": {},
                "/usr/lib32/gconv/CP771.so": {},
                "/usr/lib32/gconv/CP772.so": {},
                "/usr/lib32/gconv/CP773.so": {},
                "/usr/lib32/gconv/CP774.so": {},
                "/usr/lib32/gconv/CP775.so": {},
                "/usr/lib32/gconv/CP932.so": {},
                "/usr/lib32/gconv/CSN_369103.so": {},
                "/usr/lib32/gconv/CWI.so": {},
                "/usr/lib32/gconv/DEC-MCS.so": {},
                "/usr/lib32/gconv/EBCDIC-AT-DE-A.so": {},
                "/usr/lib32/gconv/EBCDIC-AT-DE.so": {},
                "/usr/lib32/gconv/EBCDIC-CA-FR.so": {},
                "/usr/lib32/gconv/EBCDIC-DK-NO-A.so": {},
                "/usr/lib32/gconv/EBCDIC-DK-NO.so": {},
                "/usr/lib32/gconv/EBCDIC-ES-A.so": {},
                "/usr/lib32/gconv/EBCDIC-ES-S.so": {},
                "/usr/lib32/gconv/EBCDIC-ES.so": {},
                "/usr/lib32/gconv/EBCDIC-FI-SE-A.so": {},
                "/usr/lib32/gconv/EBCDIC-FI-SE.so": {},
                "/usr/lib32/gconv/EBCDIC-FR.so": {},
                "/usr/lib32/gconv/EBCDIC-IS-FRISS.so": {},
                "/usr/lib32/gconv/EBCDIC-IT.so": {},
                "/usr/lib32/gconv/EBCDIC-PT.so": {},
                "/usr/lib32/gconv/EBCDIC-UK.so": {},
                "/usr/lib32/gconv/EBCDIC-US.so": {},
                "/usr/lib32/gconv/ECMA-CYRILLIC.so": {},
                "/usr/lib32/gconv/EUC-CN.so": {},
                "/usr/lib32/gconv/EUC-JISX0213.so": {},
                "/usr/lib32/gconv/EUC-JP-MS.so": {},
                "/usr/lib32/gconv/EUC-JP.so": {},
                "/usr/lib32/gconv/EUC-KR.so": {},
                "/usr/lib32/gconv/EUC-TW.so": {},
                "/usr/lib32/gconv/GB18030.so": {},
                "/usr/lib32/gconv/GBBIG5.so": {},
                "/usr/lib32/gconv/GBGBK.so": {},
                "/usr/lib32/gconv/GBK.so": {},
                "/usr/lib32/gconv/GEORGIAN-ACADEMY.so": {},
                "/usr/lib32/gconv/GEORGIAN-PS.so": {},
                "/usr/lib32/gconv/GOST_19768-74.so": {},
                "/usr/lib32/gconv/GREEK-CCITT.so": {},
                "/usr/lib32/gconv/GREEK7-OLD.so": {},
                "/usr/lib32/gconv/GREEK7.so": {},
                "/usr/lib32/gconv/HP-GREEK8.so": {},
                "/usr/lib32/gconv/HP-ROMAN8.so": {},
                "/usr/lib32/gconv/HP-ROMAN9.so": {},
                "/usr/lib32/gconv/HP-THAI8.so": {},
                "/usr/lib32/gconv/HP-TURKISH8.so": {},
                "/usr/lib32/gconv/IBM037.so": {},
                "/usr/lib32/gconv/IBM038.so": {},
                "/usr/lib32/gconv/IBM1004.so": {},
                "/usr/lib32/gconv/IBM1008.so": {},
                "/usr/lib32/gconv/IBM1008_420.so": {},
                "/usr/lib32/gconv/IBM1025.so": {},
                "/usr/lib32/gconv/IBM1026.so": {},
                "/usr/lib32/gconv/IBM1046.so": {},
                "/usr/lib32/gconv/IBM1047.so": {},
                "/usr/lib32/gconv/IBM1097.so": {},
                "/usr/lib32/gconv/IBM1112.so": {},
                "/usr/lib32/gconv/IBM1122.so": {},
                "/usr/lib32/gconv/IBM1123.so": {},
                "/usr/lib32/gconv/IBM1124.so": {},
                "/usr/lib32/gconv/IBM1129.so": {},
                "/usr/lib32/gconv/IBM1130.so": {},
                "/usr/lib32/gconv/IBM1132.so": {},
                "/usr/lib32/gconv/IBM1133.so": {},
                "/usr/lib32/gconv/IBM1137.so": {},
                "/usr/lib32/gconv/IBM1140.so": {},
                "/usr/lib32/gconv/IBM1141.so": {},
                "/usr/lib32/gconv/IBM1142.so": {},
                "/usr/lib32/gconv/IBM1143.so": {},
                "/usr/lib32/gconv/IBM1144.so": {},
                "/usr/lib32/gconv/IBM1145.so": {},
                "/usr/lib32/gconv/IBM1146.so": {},
                "/usr/lib32/gconv/IBM1147.so": {},
                "/usr/lib32/gconv/IBM1148.so": {},
                "/usr/lib32/gconv/IBM1149.so": {},
                "/usr/lib32/gconv/IBM1153.so": {},
                "/usr/lib32/gconv/IBM1154.so": {},
                "/usr/lib32/gconv/IBM1155.so": {},
                "/usr/lib32/gconv/IBM1156.so": {},
                "/usr/lib32/gconv/IBM1157.so": {},
                "/usr/lib32/gconv/IBM1158.so": {},
                "/usr/lib32/gconv/IBM1160.so": {},
                "/usr/lib32/gconv/IBM1161.so": {},
                "/usr/lib32/gconv/IBM1162.so": {},
                "/usr/lib32/gconv/IBM1163.so": {},
                "/usr/lib32/gconv/IBM1164.so": {},
                "/usr/lib32/gconv/IBM1166.so": {},
                "/usr/lib32/gconv/IBM1167.so": {},
                "/usr/lib32/gconv/IBM12712.so": {},
                "/usr/lib32/gconv/IBM1364.so": {},
                "/usr/lib32/gconv/IBM1371.so": {},
                "/usr/lib32/gconv/IBM1388.so": {},
                "/usr/lib32/gconv/IBM1390.so": {},
                "/usr/lib32/gconv/IBM1399.so": {},
                "/usr/lib32/gconv/IBM16804.so": {},
                "/usr/lib32/gconv/IBM256.so": {},
                "/usr/lib32/gconv/IBM273.so": {},
                "/usr/lib32/gconv/IBM274.so": {},
                "/usr/lib32/gconv/IBM275.so": {},
                "/usr/lib32/gconv/IBM277.so": {},
                "/usr/lib32/gconv/IBM278.so": {},
                "/usr/lib32/gconv/IBM280.so": {},
                "/usr/lib32/gconv/IBM281.so": {},
                "/usr/lib32/gconv/IBM284.so": {},
                "/usr/lib32/gconv/IBM285.so": {},
                "/usr/lib32/gconv/IBM290.so": {},
                "/usr/lib32/gconv/IBM297.so": {},
                "/usr/lib32/gconv/IBM420.so": {},
                "/usr/lib32/gconv/IBM423.so": {},
                "/usr/lib32/gconv/IBM424.so": {},
                "/usr/lib32/gconv/IBM437.so": {},
                "/usr/lib32/gconv/IBM4517.so": {},
                "/usr/lib32/gconv/IBM4899.so": {},
                "/usr/lib32/gconv/IBM4909.so": {},
                "/usr/lib32/gconv/IBM4971.so": {},
                "/usr/lib32/gconv/IBM500.so": {},
                "/usr/lib32/gconv/IBM5347.so": {},
                "/usr/lib32/gconv/IBM803.so": {},
                "/usr/lib32/gconv/IBM850.so": {},
                "/usr/lib32/gconv/IBM851.so": {},
                "/usr/lib32/gconv/IBM852.so": {},
                "/usr/lib32/gconv/IBM855.so": {},
                "/usr/lib32/gconv/IBM856.so": {},
                "/usr/lib32/gconv/IBM857.so": {},
                "/usr/lib32/gconv/IBM858.so": {},
                "/usr/lib32/gconv/IBM860.so": {},
                "/usr/lib32/gconv/IBM861.so": {},
                "/usr/lib32/gconv/IBM862.so": {},
                "/usr/lib32/gconv/IBM863.so": {},
                "/usr/lib32/gconv/IBM864.so": {},
                "/usr/lib32/gconv/IBM865.so": {},
                "/usr/lib32/gconv/IBM866.so": {},
                "/usr/lib32/gconv/IBM866NAV.so": {},
                "/usr/lib32/gconv/IBM868.so": {},
                "/usr/lib32/gconv/IBM869.so": {},
                "/usr/lib32/gconv/IBM870.so": {},
                "/usr/lib32/gconv/IBM871.so": {},
                "/usr/lib32/gconv/IBM874.so": {},
                "/usr/lib32/gconv/IBM875.so": {},
                "/usr/lib32/gconv/IBM880.so": {},
                "/usr/lib32/gconv/IBM891.so": {},
                "/usr/lib32/gconv/IBM901.so": {},
                "/usr/lib32/gconv/IBM902.so": {},
                "/usr/lib32/gconv/IBM903.so": {},
                "/usr/lib32/gconv/IBM9030.so": {},
                "/usr/lib32/gconv/IBM904.so": {},
                "/usr/lib32/gconv/IBM905.so": {},
                "/usr/lib32/gconv/IBM9066.so": {},
                "/usr/lib32/gconv/IBM918.so": {},
                "/usr/lib32/gconv/IBM921.so": {},
                "/usr/lib32/gconv/IBM922.so": {},
                "/usr/lib32/gconv/IBM930.so": {},
                "/usr/lib32/gconv/IBM932.so": {},
                "/usr/lib32/gconv/IBM933.so": {},
                "/usr/lib32/gconv/IBM935.so": {},
                "/usr/lib32/gconv/IBM937.so": {},
                "/usr/lib32/gconv/IBM939.so": {},
                "/usr/lib32/gconv/IBM943.so": {},
                "/usr/lib32/gconv/IBM9448.so": {},
                "/usr/lib32/gconv/IEC_P27-1.so": {},
                "/usr/lib32/gconv/INIS-8.so": {},
                "/usr/lib32/gconv/INIS-CYRILLIC.so": {},
                "/usr/lib32/gconv/INIS.so": {},
                "/usr/lib32/gconv/ISIRI-3342.so": {},
                "/usr/lib32/gconv/ISO-2022-CN-EXT.so": {},
                "/usr/lib32/gconv/ISO-2022-CN.so": {},
                "/usr/lib32/gconv/ISO-2022-JP-3.so": {},
                "/usr/lib32/gconv/ISO-2022-JP.so": {},
                "/usr/lib32/gconv/ISO-2022-KR.so": {},
                "/usr/lib32/gconv/ISO-IR-197.so": {},
                "/usr/lib32/gconv/ISO-IR-209.so": {},
                "/usr/lib32/gconv/ISO646.so": {},
                "/usr/lib32/gconv/ISO8859-1.so": {},
                "/usr/lib32/gconv/ISO8859-10.so": {},
                "/usr/lib32/gconv/ISO8859-11.so": {},
                "/usr/lib32/gconv/ISO8859-13.so": {},
                "/usr/lib32/gconv/ISO8859-14.so": {},
                "/usr/lib32/gconv/ISO8859-15.so": {},
                "/usr/lib32/gconv/ISO8859-16.so": {},
                "/usr/lib32/gconv/ISO8859-2.so": {},
                "/usr/lib32/gconv/ISO8859-3.so": {},
                "/usr/lib32/gconv/ISO8859-4.so": {},
                "/usr/lib32/gconv/ISO8859-5.so": {},
                "/usr/lib32/gconv/ISO8859-6.so": {},
                "/usr/lib32/gconv/ISO8859-7.so": {},
                "/usr/lib32/gconv/ISO8859-8.so": {},
                "/usr/lib32/gconv/ISO8859-9.so": {},
                "/usr/lib32/gconv/ISO8859-9E.so": {},
                "/usr/lib32/gconv/ISO_10367-BOX.so": {},
                "/usr/lib32/gconv/ISO_11548-1.so": {},
                "/usr/lib32/gconv/ISO_2033.so": {},
                "/usr/lib32/gconv/ISO_5427-EXT.so": {},
                "/usr/lib32/gconv/ISO_5427.so": {},
                "/usr/lib32/gconv/ISO_5428.so": {},
                "/usr/lib32/gconv/ISO_6937-2.so": {},
                "/usr/lib32/gconv/ISO_6937.so": {},
                "/usr/lib32/gconv/JOHAB.so": {},
                "/usr/lib32/gconv/KOI-8.so": {},
                "/usr/lib32/gconv/KOI8-R.so": {},
                "/usr/lib32/gconv/KOI8-RU.so": {},
                "/usr/lib32/gconv/KOI8-T.so": {},
                "/usr/lib32/gconv/KOI8-U.so": {},
                "/usr/lib32/gconv/LATIN-GREEK-1.so": {},
                "/usr/lib32/gconv/LATIN-GREEK.so": {},
                "/usr/lib32/gconv/MAC-CENTRALEUROPE.so": {},
                "/usr/lib32/gconv/MAC-IS.so": {},
                "/usr/lib32/gconv/MAC-SAMI.so": {},
                "/usr/lib32/gconv/MAC-UK.so": {},
                "/usr/lib32/gconv/MACINTOSH.so": {},
                "/usr/lib32/gconv/MIK.so": {},
                "/usr/lib32/gconv/NATS-DANO.so": {},
                "/usr/lib32/gconv/NATS-SEFI.so": {},
                "/usr/lib32/gconv/PT154.so": {},
                "/usr/lib32/gconv/RK1048.so": {},
                "/usr/lib32/gconv/SAMI-WS2.so": {},
                "/usr/lib32/gconv/SHIFT_JISX0213.so": {},
                "/usr/lib32/gconv/SJIS.so": {},
                "/usr/lib32/gconv/T.61.so": {},
                "/usr/lib32/gconv/TCVN5712-1.so": {},
                "/usr/lib32/gconv/TIS-620.so": {},
                "/usr/lib32/gconv/TSCII.so": {},
                "/usr/lib32/gconv/UHC.so": {},
                "/usr/lib32/gconv/UNICODE.so": {},
                "/usr/lib32/gconv/UTF-16.so": {},
                "/usr/lib32/gconv/UTF-32.so": {},
                "/usr/lib32/gconv/UTF-7.so": {},
                "/usr/lib32/gconv/VISCII.so": {},
                "/usr/lib32/gconv/gconv-modules": {},
                "/usr/lib32/gconv/gconv-modules.d/gconv-modules-extra.conf": {},
                "/usr/lib32/gconv/libCNS.so": {},
                "/usr/lib32/gconv/libGB.so": {},
                "/usr/lib32/gconv/libISOIR165.so": {},
                "/usr/lib32/gconv/libJIS.so": {},
                "/usr/lib32/gconv/libJISX0213.so": {},
                "/usr/lib32/gconv/libKSC.so": {},
                "/usr/lib32/gcrt1.o": {},
                "/usr/lib32/grcrt1.o": {},
                "/usr/lib32/libBrokenLocale.a": {},
                "/usr/lib32/libBrokenLocale.so": {
                    "type": "Symlink",
                    "target": "/lib32/libBrokenLocale.so.1",
                },
                "/usr/lib32/libanl.a": {},
                "/usr/lib32/libanl.so": {
                    "type": "Symlink",
                    "target": "/lib32/libanl.so.1",
                },
                "/usr/lib32/libc.a": {},
                "/usr/lib32/libc.so": {},
                "/usr/lib32/libc_malloc_debug.so": {
                    "type": "Symlink",
                    "target": "/lib32/libc_malloc_debug.so.0",
                },
                "/usr/lib32/libc_nonshared.a": {},
                "/usr/lib32/libdl.a": {},
                "/usr/lib32/libg.a": {},
                "/usr/lib32/libm.a": {},
                "/usr/lib32/libm.so": {
                    "type": "Symlink",
                    "target": "/lib32/libm.so.6",
                },
                "/usr/lib32/libmcheck.a": {},
                "/usr/lib32/libnss_compat.so": {
                    "type": "Symlink",
                    "target": "/lib32/libnss_compat.so.2",
                },
                "/usr/lib32/libnss_db.so": {
                    "type": "Symlink",
                    "target": "/lib32/libnss_db.so.2",
                },
                "/usr/lib32/libnss_hesiod.so": {
                    "type": "Symlink",
                    "target": "/lib32/libnss_hesiod.so.2",
                },
                "/usr/lib32/libpthread.a": {},
                "/usr/lib32/libresolv.a": {},
                "/usr/lib32/libresolv.so": {
                    "type": "Symlink",
                    "target": "/lib32/libresolv.so.2",
                },
                "/usr/lib32/librt.a": {},
                "/usr/lib32/libthread_db.so": {
                    "type": "Symlink",
                    "target": "/lib32/libthread_db.so.1",
                },
                "/usr/lib32/libutil.a": {},
                "/usr/lib32/misc/glibc/getconf/POSIX_V6_ILP32_OFF32": {
                    "type": "Symlink",
                    "target": "/usr/lib32/misc/glibc/getconf/XBS5_ILP32_OFFBIG",
                },
                "/usr/lib32/misc/glibc/getconf/POSIX_V6_ILP32_OFFBIG": {
                    "type": "Symlink",
                    "target": "/usr/lib32/misc/glibc/getconf/XBS5_ILP32_OFFBIG",
                },
                "/usr/lib32/misc/glibc/getconf/POSIX_V7_ILP32_OFF32": {
                    "type": "Symlink",
                    "target": "/usr/lib32/misc/glibc/getconf/XBS5_ILP32_OFFBIG",
                },
                "/usr/lib32/misc/glibc/getconf/POSIX_V7_ILP32_OFFBIG": {
                    "type": "Symlink",
                    "target": "/usr/lib32/misc/glibc/getconf/XBS5_ILP32_OFFBIG",
                },
                "/usr/lib32/misc/glibc/getconf/XBS5_ILP32_OFF32": {
                    "type": "Symlink",
                    "target": "/usr/lib32/misc/glibc/getconf/XBS5_ILP32_OFFBIG",
                },
                "/usr/lib32/misc/glibc/getconf/XBS5_ILP32_OFFBIG": {
                    "type": "ElfBinary",
                    "interp": "/lib32/ld-linux.so.2",
                    "libs": {
                        "libc.so.6": "/lib32/libc.so.6",
                    },
                },
                "/usr/lib32/rcrt1.o": {},
                "/usr/lib64/Mcrt1.o": {},
                "/usr/lib64/Scrt1.o": {},
                "/usr/lib64/audit/sotruss-lib.so": {},
                "/usr/lib64/crt1.o": {},
                "/usr/lib64/crti.o": {},
                "/usr/lib64/crtn.o": {},
                "/usr/lib64/gconv/ANSI_X3.110.so": {},
                "/usr/lib64/gconv/ARMSCII-8.so": {},
                "/usr/lib64/gconv/ASMO_449.so": {},
                "/usr/lib64/gconv/BIG5.so": {},
                "/usr/lib64/gconv/BIG5HKSCS.so": {},
                "/usr/lib64/gconv/BRF.so": {},
                "/usr/lib64/gconv/CP10007.so": {},
                "/usr/lib64/gconv/CP1125.so": {},
                "/usr/lib64/gconv/CP1250.so": {},
                "/usr/lib64/gconv/CP1251.so": {},
                "/usr/lib64/gconv/CP1252.so": {},
                "/usr/lib64/gconv/CP1253.so": {},
                "/usr/lib64/gconv/CP1254.so": {},
                "/usr/lib64/gconv/CP1255.so": {},
                "/usr/lib64/gconv/CP1256.so": {},
                "/usr/lib64/gconv/CP1257.so": {},
                "/usr/lib64/gconv/CP1258.so": {},
                "/usr/lib64/gconv/CP737.so": {},
                "/usr/lib64/gconv/CP770.so": {},
                "/usr/lib64/gconv/CP771.so": {},
                "/usr/lib64/gconv/CP772.so": {},
                "/usr/lib64/gconv/CP773.so": {},
                "/usr/lib64/gconv/CP774.so": {},
                "/usr/lib64/gconv/CP775.so": {},
                "/usr/lib64/gconv/CP932.so": {},
                "/usr/lib64/gconv/CSN_369103.so": {},
                "/usr/lib64/gconv/CWI.so": {},
                "/usr/lib64/gconv/DEC-MCS.so": {},
                "/usr/lib64/gconv/EBCDIC-AT-DE-A.so": {},
                "/usr/lib64/gconv/EBCDIC-AT-DE.so": {},
                "/usr/lib64/gconv/EBCDIC-CA-FR.so": {},
                "/usr/lib64/gconv/EBCDIC-DK-NO-A.so": {},
                "/usr/lib64/gconv/EBCDIC-DK-NO.so": {},
                "/usr/lib64/gconv/EBCDIC-ES-A.so": {},
                "/usr/lib64/gconv/EBCDIC-ES-S.so": {},
                "/usr/lib64/gconv/EBCDIC-ES.so": {},
                "/usr/lib64/gconv/EBCDIC-FI-SE-A.so": {},
                "/usr/lib64/gconv/EBCDIC-FI-SE.so": {},
                "/usr/lib64/gconv/EBCDIC-FR.so": {},
                "/usr/lib64/gconv/EBCDIC-IS-FRISS.so": {},
                "/usr/lib64/gconv/EBCDIC-IT.so": {},
                "/usr/lib64/gconv/EBCDIC-PT.so": {},
                "/usr/lib64/gconv/EBCDIC-UK.so": {},
                "/usr/lib64/gconv/EBCDIC-US.so": {},
                "/usr/lib64/gconv/ECMA-CYRILLIC.so": {},
                "/usr/lib64/gconv/EUC-CN.so": {},
                "/usr/lib64/gconv/EUC-JISX0213.so": {},
                "/usr/lib64/gconv/EUC-JP-MS.so": {},
                "/usr/lib64/gconv/EUC-JP.so": {},
                "/usr/lib64/gconv/EUC-KR.so": {},
                "/usr/lib64/gconv/EUC-TW.so": {},
                "/usr/lib64/gconv/GB18030.so": {},
                "/usr/lib64/gconv/GBBIG5.so": {},
                "/usr/lib64/gconv/GBGBK.so": {},
                "/usr/lib64/gconv/GBK.so": {},
                "/usr/lib64/gconv/GEORGIAN-ACADEMY.so": {},
                "/usr/lib64/gconv/GEORGIAN-PS.so": {},
                "/usr/lib64/gconv/GOST_19768-74.so": {},
                "/usr/lib64/gconv/GREEK-CCITT.so": {},
                "/usr/lib64/gconv/GREEK7-OLD.so": {},
                "/usr/lib64/gconv/GREEK7.so": {},
                "/usr/lib64/gconv/HP-GREEK8.so": {},
                "/usr/lib64/gconv/HP-ROMAN8.so": {},
                "/usr/lib64/gconv/HP-ROMAN9.so": {},
                "/usr/lib64/gconv/HP-THAI8.so": {},
                "/usr/lib64/gconv/HP-TURKISH8.so": {},
                "/usr/lib64/gconv/IBM037.so": {},
                "/usr/lib64/gconv/IBM038.so": {},
                "/usr/lib64/gconv/IBM1004.so": {},
                "/usr/lib64/gconv/IBM1008.so": {},
                "/usr/lib64/gconv/IBM1008_420.so": {},
                "/usr/lib64/gconv/IBM1025.so": {},
                "/usr/lib64/gconv/IBM1026.so": {},
                "/usr/lib64/gconv/IBM1046.so": {},
                "/usr/lib64/gconv/IBM1047.so": {},
                "/usr/lib64/gconv/IBM1097.so": {},
                "/usr/lib64/gconv/IBM1112.so": {},
                "/usr/lib64/gconv/IBM1122.so": {},
                "/usr/lib64/gconv/IBM1123.so": {},
                "/usr/lib64/gconv/IBM1124.so": {},
                "/usr/lib64/gconv/IBM1129.so": {},
                "/usr/lib64/gconv/IBM1130.so": {},
                "/usr/lib64/gconv/IBM1132.so": {},
                "/usr/lib64/gconv/IBM1133.so": {},
                "/usr/lib64/gconv/IBM1137.so": {},
                "/usr/lib64/gconv/IBM1140.so": {},
                "/usr/lib64/gconv/IBM1141.so": {},
                "/usr/lib64/gconv/IBM1142.so": {},
                "/usr/lib64/gconv/IBM1143.so": {},
                "/usr/lib64/gconv/IBM1144.so": {},
                "/usr/lib64/gconv/IBM1145.so": {},
                "/usr/lib64/gconv/IBM1146.so": {},
                "/usr/lib64/gconv/IBM1147.so": {},
                "/usr/lib64/gconv/IBM1148.so": {},
                "/usr/lib64/gconv/IBM1149.so": {},
                "/usr/lib64/gconv/IBM1153.so": {},
                "/usr/lib64/gconv/IBM1154.so": {},
                "/usr/lib64/gconv/IBM1155.so": {},
                "/usr/lib64/gconv/IBM1156.so": {},
                "/usr/lib64/gconv/IBM1157.so": {},
                "/usr/lib64/gconv/IBM1158.so": {},
                "/usr/lib64/gconv/IBM1160.so": {},
                "/usr/lib64/gconv/IBM1161.so": {},
                "/usr/lib64/gconv/IBM1162.so": {},
                "/usr/lib64/gconv/IBM1163.so": {},
                "/usr/lib64/gconv/IBM1164.so": {},
                "/usr/lib64/gconv/IBM1166.so": {},
                "/usr/lib64/gconv/IBM1167.so": {},
                "/usr/lib64/gconv/IBM12712.so": {},
                "/usr/lib64/gconv/IBM1364.so": {},
                "/usr/lib64/gconv/IBM1371.so": {},
                "/usr/lib64/gconv/IBM1388.so": {},
                "/usr/lib64/gconv/IBM1390.so": {},
                "/usr/lib64/gconv/IBM1399.so": {},
                "/usr/lib64/gconv/IBM16804.so": {},
                "/usr/lib64/gconv/IBM256.so": {},
                "/usr/lib64/gconv/IBM273.so": {},
                "/usr/lib64/gconv/IBM274.so": {},
                "/usr/lib64/gconv/IBM275.so": {},
                "/usr/lib64/gconv/IBM277.so": {},
                "/usr/lib64/gconv/IBM278.so": {},
                "/usr/lib64/gconv/IBM280.so": {},
                "/usr/lib64/gconv/IBM281.so": {},
                "/usr/lib64/gconv/IBM284.so": {},
                "/usr/lib64/gconv/IBM285.so": {},
                "/usr/lib64/gconv/IBM290.so": {},
                "/usr/lib64/gconv/IBM297.so": {},
                "/usr/lib64/gconv/IBM420.so": {},
                "/usr/lib64/gconv/IBM423.so": {},
                "/usr/lib64/gconv/IBM424.so": {},
                "/usr/lib64/gconv/IBM437.so": {},
                "/usr/lib64/gconv/IBM4517.so": {},
                "/usr/lib64/gconv/IBM4899.so": {},
                "/usr/lib64/gconv/IBM4909.so": {},
                "/usr/lib64/gconv/IBM4971.so": {},
                "/usr/lib64/gconv/IBM500.so": {},
                "/usr/lib64/gconv/IBM5347.so": {},
                "/usr/lib64/gconv/IBM803.so": {},
                "/usr/lib64/gconv/IBM850.so": {},
                "/usr/lib64/gconv/IBM851.so": {},
                "/usr/lib64/gconv/IBM852.so": {},
                "/usr/lib64/gconv/IBM855.so": {},
                "/usr/lib64/gconv/IBM856.so": {},
                "/usr/lib64/gconv/IBM857.so": {},
                "/usr/lib64/gconv/IBM858.so": {},
                "/usr/lib64/gconv/IBM860.so": {},
                "/usr/lib64/gconv/IBM861.so": {},
                "/usr/lib64/gconv/IBM862.so": {},
                "/usr/lib64/gconv/IBM863.so": {},
                "/usr/lib64/gconv/IBM864.so": {},
                "/usr/lib64/gconv/IBM865.so": {},
                "/usr/lib64/gconv/IBM866.so": {},
                "/usr/lib64/gconv/IBM866NAV.so": {},
                "/usr/lib64/gconv/IBM868.so": {},
                "/usr/lib64/gconv/IBM869.so": {},
                "/usr/lib64/gconv/IBM870.so": {},
                "/usr/lib64/gconv/IBM871.so": {},
                "/usr/lib64/gconv/IBM874.so": {},
                "/usr/lib64/gconv/IBM875.so": {},
                "/usr/lib64/gconv/IBM880.so": {},
                "/usr/lib64/gconv/IBM891.so": {},
                "/usr/lib64/gconv/IBM901.so": {},
                "/usr/lib64/gconv/IBM902.so": {},
                "/usr/lib64/gconv/IBM903.so": {},
                "/usr/lib64/gconv/IBM9030.so": {},
                "/usr/lib64/gconv/IBM904.so": {},
                "/usr/lib64/gconv/IBM905.so": {},
                "/usr/lib64/gconv/IBM9066.so": {},
                "/usr/lib64/gconv/IBM918.so": {},
                "/usr/lib64/gconv/IBM921.so": {},
                "/usr/lib64/gconv/IBM922.so": {},
                "/usr/lib64/gconv/IBM930.so": {},
                "/usr/lib64/gconv/IBM932.so": {},
                "/usr/lib64/gconv/IBM933.so": {},
                "/usr/lib64/gconv/IBM935.so": {},
                "/usr/lib64/gconv/IBM937.so": {},
                "/usr/lib64/gconv/IBM939.so": {},
                "/usr/lib64/gconv/IBM943.so": {},
                "/usr/lib64/gconv/IBM9448.so": {},
                "/usr/lib64/gconv/IEC_P27-1.so": {},
                "/usr/lib64/gconv/INIS-8.so": {},
                "/usr/lib64/gconv/INIS-CYRILLIC.so": {},
                "/usr/lib64/gconv/INIS.so": {},
                "/usr/lib64/gconv/ISIRI-3342.so": {},
                "/usr/lib64/gconv/ISO-2022-CN-EXT.so": {},
                "/usr/lib64/gconv/ISO-2022-CN.so": {},
                "/usr/lib64/gconv/ISO-2022-JP-3.so": {},
                "/usr/lib64/gconv/ISO-2022-JP.so": {},
                "/usr/lib64/gconv/ISO-2022-KR.so": {},
                "/usr/lib64/gconv/ISO-IR-197.so": {},
                "/usr/lib64/gconv/ISO-IR-209.so": {},
                "/usr/lib64/gconv/ISO646.so": {},
                "/usr/lib64/gconv/ISO8859-1.so": {},
                "/usr/lib64/gconv/ISO8859-10.so": {},
                "/usr/lib64/gconv/ISO8859-11.so": {},
                "/usr/lib64/gconv/ISO8859-13.so": {},
                "/usr/lib64/gconv/ISO8859-14.so": {},
                "/usr/lib64/gconv/ISO8859-15.so": {},
                "/usr/lib64/gconv/ISO8859-16.so": {},
                "/usr/lib64/gconv/ISO8859-2.so": {},
                "/usr/lib64/gconv/ISO8859-3.so": {},
                "/usr/lib64/gconv/ISO8859-4.so": {},
                "/usr/lib64/gconv/ISO8859-5.so": {},
                "/usr/lib64/gconv/ISO8859-6.so": {},
                "/usr/lib64/gconv/ISO8859-7.so": {},
                "/usr/lib64/gconv/ISO8859-8.so": {},
                "/usr/lib64/gconv/ISO8859-9.so": {},
                "/usr/lib64/gconv/ISO8859-9E.so": {},
                "/usr/lib64/gconv/ISO_10367-BOX.so": {},
                "/usr/lib64/gconv/ISO_11548-1.so": {},
                "/usr/lib64/gconv/ISO_2033.so": {},
                "/usr/lib64/gconv/ISO_5427-EXT.so": {},
                "/usr/lib64/gconv/ISO_5427.so": {},
                "/usr/lib64/gconv/ISO_5428.so": {},
                "/usr/lib64/gconv/ISO_6937-2.so": {},
                "/usr/lib64/gconv/ISO_6937.so": {},
                "/usr/lib64/gconv/JOHAB.so": {},
                "/usr/lib64/gconv/KOI-8.so": {},
                "/usr/lib64/gconv/KOI8-R.so": {},
                "/usr/lib64/gconv/KOI8-RU.so": {},
                "/usr/lib64/gconv/KOI8-T.so": {},
                "/usr/lib64/gconv/KOI8-U.so": {},
                "/usr/lib64/gconv/LATIN-GREEK-1.so": {},
                "/usr/lib64/gconv/LATIN-GREEK.so": {},
                "/usr/lib64/gconv/MAC-CENTRALEUROPE.so": {},
                "/usr/lib64/gconv/MAC-IS.so": {},
                "/usr/lib64/gconv/MAC-SAMI.so": {},
                "/usr/lib64/gconv/MAC-UK.so": {},
                "/usr/lib64/gconv/MACINTOSH.so": {},
                "/usr/lib64/gconv/MIK.so": {},
                "/usr/lib64/gconv/NATS-DANO.so": {},
                "/usr/lib64/gconv/NATS-SEFI.so": {},
                "/usr/lib64/gconv/PT154.so": {},
                "/usr/lib64/gconv/RK1048.so": {},
                "/usr/lib64/gconv/SAMI-WS2.so": {},
                "/usr/lib64/gconv/SHIFT_JISX0213.so": {},
                "/usr/lib64/gconv/SJIS.so": {},
                "/usr/lib64/gconv/T.61.so": {},
                "/usr/lib64/gconv/TCVN5712-1.so": {},
                "/usr/lib64/gconv/TIS-620.so": {},
                "/usr/lib64/gconv/TSCII.so": {},
                "/usr/lib64/gconv/UHC.so": {},
                "/usr/lib64/gconv/UNICODE.so": {},
                "/usr/lib64/gconv/UTF-16.so": {},
                "/usr/lib64/gconv/UTF-32.so": {},
                "/usr/lib64/gconv/UTF-7.so": {},
                "/usr/lib64/gconv/VISCII.so": {},
                "/usr/lib64/gconv/gconv-modules": {},
                "/usr/lib64/gconv/gconv-modules.d/gconv-modules-extra.conf": {},
                "/usr/lib64/gconv/libCNS.so": {},
                "/usr/lib64/gconv/libGB.so": {},
                "/usr/lib64/gconv/libISOIR165.so": {},
                "/usr/lib64/gconv/libJIS.so": {},
                "/usr/lib64/gconv/libJISX0213.so": {},
                "/usr/lib64/gconv/libKSC.so": {},
                "/usr/lib64/gcrt1.o": {},
                "/usr/lib64/glibc-2.35/libm-2.35.a": {},
                "/usr/lib64/grcrt1.o": {},
                "/usr/lib64/libBrokenLocale.a": {},
                "/usr/lib64/libBrokenLocale.so": {
                    "type": "Symlink",
                    "target": "/lib64/libBrokenLocale.so.1",
                },
                "/usr/lib64/libanl.a": {},
                "/usr/lib64/libanl.so": {
                    "type": "Symlink",
                    "target": "/lib64/libanl.so.1",
                },
                "/usr/lib64/libc.a": {},
                "/usr/lib64/libc.so": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libc_malloc_debug.so": {
                    "type": "Symlink",
                    "target": "/lib64/libc_malloc_debug.so.0",
                },
                "/usr/lib64/libc_nonshared.a": {},
                "/usr/lib64/libdl.a": {},
                "/usr/lib64/libg.a": {},
                "/usr/lib64/libm.a": {},
                "/usr/lib64/libm.so": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libmcheck.a": {},
                "/usr/lib64/libmvec.a": {},
                "/usr/lib64/libmvec.so": {
                    "type": "Symlink",
                    "target": "/lib64/libmvec.so.1",
                },
                "/usr/lib64/libnss_compat.so": {
                    "type": "Symlink",
                    "target": "/lib64/libnss_compat.so.2",
                },
                "/usr/lib64/libnss_db.so": {
                    "type": "Symlink",
                    "target": "/lib64/libnss_db.so.2",
                },
                "/usr/lib64/libnss_hesiod.so": {
                    "type": "Symlink",
                    "target": "/lib64/libnss_hesiod.so.2",
                },
                "/usr/lib64/libpthread.a": {},
                "/usr/lib64/libresolv.a": {},
                "/usr/lib64/libresolv.so": {
                    "type": "Symlink",
                    "target": "/lib64/libresolv.so.2",
                },
                "/usr/lib64/librt.a": {},
                "/usr/lib64/libthread_db.so": {
                    "type": "Symlink",
                    "target": "/lib64/libthread_db.so.1",
                },
                "/usr/lib64/libutil.a": {},
                "/usr/lib64/locale/.keep_sys-libs_glibc-2.2": {},
                "/usr/lib64/misc/glibc/getconf/POSIX_V6_LP64_OFF64": {
                    "type": "Symlink",
                    "target": "/usr/lib64/misc/glibc/getconf/POSIX_V7_LP64_OFF64",
                },
                "/usr/lib64/misc/glibc/getconf/POSIX_V7_LP64_OFF64": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/lib64/misc/glibc/getconf/XBS5_LP64_OFF64": {
                    "type": "Symlink",
                    "target": "/usr/lib64/misc/glibc/getconf/POSIX_V7_LP64_OFF64",
                },
                "/usr/lib64/rcrt1.o": {},
                "/usr/sbin/iconvconfig": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                    },
                },
                "/usr/sbin/locale-gen": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.1.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.10.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.11.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.12.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.13.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.14.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.15.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.16.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.17.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.18.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.19.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.2.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.20.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.21.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.22.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.23.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.24.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.3.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.4.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.5.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.6.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.7.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.8.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.9.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.libidn.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.localedata.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.nptl.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.nptl_db.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-aarch64.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-aix.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-alpha.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-am33.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-arm.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-cris.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-hppa.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-ia64.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-linux-generic.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-m68k.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-microblaze.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-mips.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-powerpc.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports-tile.gz": {},
                "/usr/share/doc/glibc-2.35-r22/ChangeLog.old/ChangeLog.ports.gz": {},
                "/usr/share/doc/glibc-2.35-r22/NEWS.gz": {},
                "/usr/share/doc/glibc-2.35-r22/README.gz": {},
                "/usr/share/i18n/SUPPORTED": {},
                "/usr/share/i18n/charmaps/ANSI_X3.110-1983.gz": {},
                "/usr/share/i18n/charmaps/ANSI_X3.4-1968.gz": {},
                "/usr/share/i18n/charmaps/ARMSCII-8.gz": {},
                "/usr/share/i18n/charmaps/ASMO_449.gz": {},
                "/usr/share/i18n/charmaps/BIG5-HKSCS.gz": {},
                "/usr/share/i18n/charmaps/BIG5.gz": {},
                "/usr/share/i18n/charmaps/BRF.gz": {},
                "/usr/share/i18n/charmaps/BS_4730.gz": {},
                "/usr/share/i18n/charmaps/BS_VIEWDATA.gz": {},
                "/usr/share/i18n/charmaps/CP10007.gz": {},
                "/usr/share/i18n/charmaps/CP1125.gz": {},
                "/usr/share/i18n/charmaps/CP1250.gz": {},
                "/usr/share/i18n/charmaps/CP1251.gz": {},
                "/usr/share/i18n/charmaps/CP1252.gz": {},
                "/usr/share/i18n/charmaps/CP1253.gz": {},
                "/usr/share/i18n/charmaps/CP1254.gz": {},
                "/usr/share/i18n/charmaps/CP1255.gz": {},
                "/usr/share/i18n/charmaps/CP1256.gz": {},
                "/usr/share/i18n/charmaps/CP1257.gz": {},
                "/usr/share/i18n/charmaps/CP1258.gz": {},
                "/usr/share/i18n/charmaps/CP737.gz": {},
                "/usr/share/i18n/charmaps/CP770.gz": {},
                "/usr/share/i18n/charmaps/CP771.gz": {},
                "/usr/share/i18n/charmaps/CP772.gz": {},
                "/usr/share/i18n/charmaps/CP773.gz": {},
                "/usr/share/i18n/charmaps/CP774.gz": {},
                "/usr/share/i18n/charmaps/CP775.gz": {},
                "/usr/share/i18n/charmaps/CP949.gz": {},
                "/usr/share/i18n/charmaps/CSA_Z243.4-1985-1.gz": {},
                "/usr/share/i18n/charmaps/CSA_Z243.4-1985-2.gz": {},
                "/usr/share/i18n/charmaps/CSA_Z243.4-1985-GR.gz": {},
                "/usr/share/i18n/charmaps/CSN_369103.gz": {},
                "/usr/share/i18n/charmaps/CWI.gz": {},
                "/usr/share/i18n/charmaps/DEC-MCS.gz": {},
                "/usr/share/i18n/charmaps/DIN_66003.gz": {},
                "/usr/share/i18n/charmaps/DS_2089.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-AT-DE-A.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-AT-DE.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-CA-FR.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-DK-NO-A.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-DK-NO.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-ES-A.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-ES-S.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-ES.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-FI-SE-A.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-FI-SE.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-FR.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-IS-FRISS.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-IT.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-PT.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-UK.gz": {},
                "/usr/share/i18n/charmaps/EBCDIC-US.gz": {},
                "/usr/share/i18n/charmaps/ECMA-CYRILLIC.gz": {},
                "/usr/share/i18n/charmaps/ES.gz": {},
                "/usr/share/i18n/charmaps/ES2.gz": {},
                "/usr/share/i18n/charmaps/EUC-JISX0213.gz": {},
                "/usr/share/i18n/charmaps/EUC-JP-MS.gz": {},
                "/usr/share/i18n/charmaps/EUC-JP.gz": {},
                "/usr/share/i18n/charmaps/EUC-KR.gz": {},
                "/usr/share/i18n/charmaps/EUC-TW.gz": {},
                "/usr/share/i18n/charmaps/GB18030.gz": {},
                "/usr/share/i18n/charmaps/GB2312.gz": {},
                "/usr/share/i18n/charmaps/GBK.gz": {},
                "/usr/share/i18n/charmaps/GB_1988-80.gz": {},
                "/usr/share/i18n/charmaps/GEORGIAN-ACADEMY.gz": {},
                "/usr/share/i18n/charmaps/GEORGIAN-PS.gz": {},
                "/usr/share/i18n/charmaps/GOST_19768-74.gz": {},
                "/usr/share/i18n/charmaps/GREEK-CCITT.gz": {},
                "/usr/share/i18n/charmaps/GREEK7-OLD.gz": {},
                "/usr/share/i18n/charmaps/GREEK7.gz": {},
                "/usr/share/i18n/charmaps/HP-GREEK8.gz": {},
                "/usr/share/i18n/charmaps/HP-ROMAN8.gz": {},
                "/usr/share/i18n/charmaps/HP-ROMAN9.gz": {},
                "/usr/share/i18n/charmaps/HP-THAI8.gz": {},
                "/usr/share/i18n/charmaps/HP-TURKISH8.gz": {},
                "/usr/share/i18n/charmaps/IBM037.gz": {},
                "/usr/share/i18n/charmaps/IBM038.gz": {},
                "/usr/share/i18n/charmaps/IBM1004.gz": {},
                "/usr/share/i18n/charmaps/IBM1026.gz": {},
                "/usr/share/i18n/charmaps/IBM1047.gz": {},
                "/usr/share/i18n/charmaps/IBM1124.gz": {},
                "/usr/share/i18n/charmaps/IBM1129.gz": {},
                "/usr/share/i18n/charmaps/IBM1132.gz": {},
                "/usr/share/i18n/charmaps/IBM1133.gz": {},
                "/usr/share/i18n/charmaps/IBM1160.gz": {},
                "/usr/share/i18n/charmaps/IBM1161.gz": {},
                "/usr/share/i18n/charmaps/IBM1162.gz": {},
                "/usr/share/i18n/charmaps/IBM1163.gz": {},
                "/usr/share/i18n/charmaps/IBM1164.gz": {},
                "/usr/share/i18n/charmaps/IBM256.gz": {},
                "/usr/share/i18n/charmaps/IBM273.gz": {},
                "/usr/share/i18n/charmaps/IBM274.gz": {},
                "/usr/share/i18n/charmaps/IBM275.gz": {},
                "/usr/share/i18n/charmaps/IBM277.gz": {},
                "/usr/share/i18n/charmaps/IBM278.gz": {},
                "/usr/share/i18n/charmaps/IBM280.gz": {},
                "/usr/share/i18n/charmaps/IBM281.gz": {},
                "/usr/share/i18n/charmaps/IBM284.gz": {},
                "/usr/share/i18n/charmaps/IBM285.gz": {},
                "/usr/share/i18n/charmaps/IBM290.gz": {},
                "/usr/share/i18n/charmaps/IBM297.gz": {},
                "/usr/share/i18n/charmaps/IBM420.gz": {},
                "/usr/share/i18n/charmaps/IBM423.gz": {},
                "/usr/share/i18n/charmaps/IBM424.gz": {},
                "/usr/share/i18n/charmaps/IBM437.gz": {},
                "/usr/share/i18n/charmaps/IBM500.gz": {},
                "/usr/share/i18n/charmaps/IBM850.gz": {},
                "/usr/share/i18n/charmaps/IBM851.gz": {},
                "/usr/share/i18n/charmaps/IBM852.gz": {},
                "/usr/share/i18n/charmaps/IBM855.gz": {},
                "/usr/share/i18n/charmaps/IBM856.gz": {},
                "/usr/share/i18n/charmaps/IBM857.gz": {},
                "/usr/share/i18n/charmaps/IBM858.gz": {},
                "/usr/share/i18n/charmaps/IBM860.gz": {},
                "/usr/share/i18n/charmaps/IBM861.gz": {},
                "/usr/share/i18n/charmaps/IBM862.gz": {},
                "/usr/share/i18n/charmaps/IBM863.gz": {},
                "/usr/share/i18n/charmaps/IBM864.gz": {},
                "/usr/share/i18n/charmaps/IBM865.gz": {},
                "/usr/share/i18n/charmaps/IBM866.gz": {},
                "/usr/share/i18n/charmaps/IBM866NAV.gz": {},
                "/usr/share/i18n/charmaps/IBM868.gz": {},
                "/usr/share/i18n/charmaps/IBM869.gz": {},
                "/usr/share/i18n/charmaps/IBM870.gz": {},
                "/usr/share/i18n/charmaps/IBM871.gz": {},
                "/usr/share/i18n/charmaps/IBM874.gz": {},
                "/usr/share/i18n/charmaps/IBM875.gz": {},
                "/usr/share/i18n/charmaps/IBM880.gz": {},
                "/usr/share/i18n/charmaps/IBM891.gz": {},
                "/usr/share/i18n/charmaps/IBM903.gz": {},
                "/usr/share/i18n/charmaps/IBM904.gz": {},
                "/usr/share/i18n/charmaps/IBM905.gz": {},
                "/usr/share/i18n/charmaps/IBM918.gz": {},
                "/usr/share/i18n/charmaps/IBM922.gz": {},
                "/usr/share/i18n/charmaps/IEC_P27-1.gz": {},
                "/usr/share/i18n/charmaps/INIS-8.gz": {},
                "/usr/share/i18n/charmaps/INIS-CYRILLIC.gz": {},
                "/usr/share/i18n/charmaps/INIS.gz": {},
                "/usr/share/i18n/charmaps/INVARIANT.gz": {},
                "/usr/share/i18n/charmaps/ISIRI-3342.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-1.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-10.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-11.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-13.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-14.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-15.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-16.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-2.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-3.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-4.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-5.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-6.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-7.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-8.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-9.gz": {},
                "/usr/share/i18n/charmaps/ISO-8859-9E.gz": {},
                "/usr/share/i18n/charmaps/ISO-IR-197.gz": {},
                "/usr/share/i18n/charmaps/ISO-IR-209.gz": {},
                "/usr/share/i18n/charmaps/ISO-IR-90.gz": {},
                "/usr/share/i18n/charmaps/ISO_10367-BOX.gz": {},
                "/usr/share/i18n/charmaps/ISO_10646.gz": {},
                "/usr/share/i18n/charmaps/ISO_11548-1.gz": {},
                "/usr/share/i18n/charmaps/ISO_2033-1983.gz": {},
                "/usr/share/i18n/charmaps/ISO_5427-EXT.gz": {},
                "/usr/share/i18n/charmaps/ISO_5427.gz": {},
                "/usr/share/i18n/charmaps/ISO_5428.gz": {},
                "/usr/share/i18n/charmaps/ISO_646.BASIC.gz": {},
                "/usr/share/i18n/charmaps/ISO_646.IRV.gz": {},
                "/usr/share/i18n/charmaps/ISO_6937-2-25.gz": {},
                "/usr/share/i18n/charmaps/ISO_6937-2-ADD.gz": {},
                "/usr/share/i18n/charmaps/ISO_6937.gz": {},
                "/usr/share/i18n/charmaps/ISO_8859-1,GL.gz": {},
                "/usr/share/i18n/charmaps/ISO_8859-SUPP.gz": {},
                "/usr/share/i18n/charmaps/IT.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6220-1969-JP.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6220-1969-RO.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6229-1984-A.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6229-1984-B-ADD.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6229-1984-B.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6229-1984-HAND-ADD.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6229-1984-HAND.gz": {},
                "/usr/share/i18n/charmaps/JIS_C6229-1984-KANA.gz": {},
                "/usr/share/i18n/charmaps/JIS_X0201.gz": {},
                "/usr/share/i18n/charmaps/JOHAB.gz": {},
                "/usr/share/i18n/charmaps/JUS_I.B1.002.gz": {},
                "/usr/share/i18n/charmaps/JUS_I.B1.003-MAC.gz": {},
                "/usr/share/i18n/charmaps/JUS_I.B1.003-SERB.gz": {},
                "/usr/share/i18n/charmaps/KOI-8.gz": {},
                "/usr/share/i18n/charmaps/KOI8-R.gz": {},
                "/usr/share/i18n/charmaps/KOI8-RU.gz": {},
                "/usr/share/i18n/charmaps/KOI8-T.gz": {},
                "/usr/share/i18n/charmaps/KOI8-U.gz": {},
                "/usr/share/i18n/charmaps/KSC5636.gz": {},
                "/usr/share/i18n/charmaps/LATIN-GREEK-1.gz": {},
                "/usr/share/i18n/charmaps/LATIN-GREEK.gz": {},
                "/usr/share/i18n/charmaps/MAC-CENTRALEUROPE.gz": {},
                "/usr/share/i18n/charmaps/MAC-CYRILLIC.gz": {},
                "/usr/share/i18n/charmaps/MAC-IS.gz": {},
                "/usr/share/i18n/charmaps/MAC-SAMI.gz": {},
                "/usr/share/i18n/charmaps/MAC-UK.gz": {},
                "/usr/share/i18n/charmaps/MACINTOSH.gz": {},
                "/usr/share/i18n/charmaps/MIK.gz": {},
                "/usr/share/i18n/charmaps/MSZ_7795.3.gz": {},
                "/usr/share/i18n/charmaps/NATS-DANO-ADD.gz": {},
                "/usr/share/i18n/charmaps/NATS-DANO.gz": {},
                "/usr/share/i18n/charmaps/NATS-SEFI-ADD.gz": {},
                "/usr/share/i18n/charmaps/NATS-SEFI.gz": {},
                "/usr/share/i18n/charmaps/NC_NC00-10.gz": {},
                "/usr/share/i18n/charmaps/NEXTSTEP.gz": {},
                "/usr/share/i18n/charmaps/NF_Z_62-010.gz": {},
                "/usr/share/i18n/charmaps/NF_Z_62-010_1973.gz": {},
                "/usr/share/i18n/charmaps/NS_4551-1.gz": {},
                "/usr/share/i18n/charmaps/NS_4551-2.gz": {},
                "/usr/share/i18n/charmaps/PT.gz": {},
                "/usr/share/i18n/charmaps/PT154.gz": {},
                "/usr/share/i18n/charmaps/PT2.gz": {},
                "/usr/share/i18n/charmaps/RK1048.gz": {},
                "/usr/share/i18n/charmaps/SAMI-WS2.gz": {},
                "/usr/share/i18n/charmaps/SAMI.gz": {},
                "/usr/share/i18n/charmaps/SEN_850200_B.gz": {},
                "/usr/share/i18n/charmaps/SEN_850200_C.gz": {},
                "/usr/share/i18n/charmaps/SHIFT_JIS.gz": {},
                "/usr/share/i18n/charmaps/SHIFT_JISX0213.gz": {},
                "/usr/share/i18n/charmaps/T.101-G2.gz": {},
                "/usr/share/i18n/charmaps/T.61-7BIT.gz": {},
                "/usr/share/i18n/charmaps/T.61-8BIT.gz": {},
                "/usr/share/i18n/charmaps/TCVN5712-1.gz": {},
                "/usr/share/i18n/charmaps/TIS-620.gz": {},
                "/usr/share/i18n/charmaps/TSCII.gz": {},
                "/usr/share/i18n/charmaps/UTF-8.gz": {},
                "/usr/share/i18n/charmaps/VIDEOTEX-SUPPL.gz": {},
                "/usr/share/i18n/charmaps/VISCII.gz": {},
                "/usr/share/i18n/charmaps/WINDOWS-31J.gz": {},
                "/usr/share/i18n/locales/C": {},
                "/usr/share/i18n/locales/POSIX": {},
                "/usr/share/i18n/locales/aa_DJ": {},
                "/usr/share/i18n/locales/aa_ER": {},
                "/usr/share/i18n/locales/aa_ER@saaho": {},
                "/usr/share/i18n/locales/aa_ET": {},
                "/usr/share/i18n/locales/ab_GE": {},
                "/usr/share/i18n/locales/af_ZA": {},
                "/usr/share/i18n/locales/agr_PE": {},
                "/usr/share/i18n/locales/ak_GH": {},
                "/usr/share/i18n/locales/am_ET": {},
                "/usr/share/i18n/locales/an_ES": {},
                "/usr/share/i18n/locales/anp_IN": {},
                "/usr/share/i18n/locales/ar_AE": {},
                "/usr/share/i18n/locales/ar_BH": {},
                "/usr/share/i18n/locales/ar_DZ": {},
                "/usr/share/i18n/locales/ar_EG": {},
                "/usr/share/i18n/locales/ar_IN": {},
                "/usr/share/i18n/locales/ar_IQ": {},
                "/usr/share/i18n/locales/ar_JO": {},
                "/usr/share/i18n/locales/ar_KW": {},
                "/usr/share/i18n/locales/ar_LB": {},
                "/usr/share/i18n/locales/ar_LY": {},
                "/usr/share/i18n/locales/ar_MA": {},
                "/usr/share/i18n/locales/ar_OM": {},
                "/usr/share/i18n/locales/ar_QA": {},
                "/usr/share/i18n/locales/ar_SA": {},
                "/usr/share/i18n/locales/ar_SD": {},
                "/usr/share/i18n/locales/ar_SS": {},
                "/usr/share/i18n/locales/ar_SY": {},
                "/usr/share/i18n/locales/ar_TN": {},
                "/usr/share/i18n/locales/ar_YE": {},
                "/usr/share/i18n/locales/as_IN": {},
                "/usr/share/i18n/locales/ast_ES": {},
                "/usr/share/i18n/locales/ayc_PE": {},
                "/usr/share/i18n/locales/az_AZ": {},
                "/usr/share/i18n/locales/az_IR": {},
                "/usr/share/i18n/locales/be_BY": {},
                "/usr/share/i18n/locales/be_BY@latin": {},
                "/usr/share/i18n/locales/bem_ZM": {},
                "/usr/share/i18n/locales/ber_DZ": {},
                "/usr/share/i18n/locales/ber_MA": {},
                "/usr/share/i18n/locales/bg_BG": {},
                "/usr/share/i18n/locales/bhb_IN": {},
                "/usr/share/i18n/locales/bho_IN": {},
                "/usr/share/i18n/locales/bho_NP": {},
                "/usr/share/i18n/locales/bi_VU": {},
                "/usr/share/i18n/locales/bn_BD": {},
                "/usr/share/i18n/locales/bn_IN": {},
                "/usr/share/i18n/locales/bo_CN": {},
                "/usr/share/i18n/locales/bo_IN": {},
                "/usr/share/i18n/locales/br_FR": {},
                "/usr/share/i18n/locales/br_FR@euro": {},
                "/usr/share/i18n/locales/brx_IN": {},
                "/usr/share/i18n/locales/bs_BA": {},
                "/usr/share/i18n/locales/byn_ER": {},
                "/usr/share/i18n/locales/ca_AD": {},
                "/usr/share/i18n/locales/ca_ES": {},
                "/usr/share/i18n/locales/ca_ES@euro": {},
                "/usr/share/i18n/locales/ca_ES@valencia": {},
                "/usr/share/i18n/locales/ca_FR": {},
                "/usr/share/i18n/locales/ca_IT": {},
                "/usr/share/i18n/locales/ce_RU": {},
                "/usr/share/i18n/locales/chr_US": {},
                "/usr/share/i18n/locales/ckb_IQ": {},
                "/usr/share/i18n/locales/cmn_TW": {},
                "/usr/share/i18n/locales/cns11643_stroke": {},
                "/usr/share/i18n/locales/crh_UA": {},
                "/usr/share/i18n/locales/cs_CZ": {},
                "/usr/share/i18n/locales/csb_PL": {},
                "/usr/share/i18n/locales/cv_RU": {},
                "/usr/share/i18n/locales/cy_GB": {},
                "/usr/share/i18n/locales/da_DK": {},
                "/usr/share/i18n/locales/de_AT": {},
                "/usr/share/i18n/locales/de_AT@euro": {},
                "/usr/share/i18n/locales/de_BE": {},
                "/usr/share/i18n/locales/de_BE@euro": {},
                "/usr/share/i18n/locales/de_CH": {},
                "/usr/share/i18n/locales/de_DE": {},
                "/usr/share/i18n/locales/de_DE@euro": {},
                "/usr/share/i18n/locales/de_IT": {},
                "/usr/share/i18n/locales/de_LI": {},
                "/usr/share/i18n/locales/de_LU": {},
                "/usr/share/i18n/locales/de_LU@euro": {},
                "/usr/share/i18n/locales/doi_IN": {},
                "/usr/share/i18n/locales/dsb_DE": {},
                "/usr/share/i18n/locales/dv_MV": {},
                "/usr/share/i18n/locales/dz_BT": {},
                "/usr/share/i18n/locales/el_CY": {},
                "/usr/share/i18n/locales/el_GR": {},
                "/usr/share/i18n/locales/el_GR@euro": {},
                "/usr/share/i18n/locales/en_AG": {},
                "/usr/share/i18n/locales/en_AU": {},
                "/usr/share/i18n/locales/en_BW": {},
                "/usr/share/i18n/locales/en_CA": {},
                "/usr/share/i18n/locales/en_DK": {},
                "/usr/share/i18n/locales/en_GB": {},
                "/usr/share/i18n/locales/en_HK": {},
                "/usr/share/i18n/locales/en_IE": {},
                "/usr/share/i18n/locales/en_IE@euro": {},
                "/usr/share/i18n/locales/en_IL": {},
                "/usr/share/i18n/locales/en_IN": {},
                "/usr/share/i18n/locales/en_NG": {},
                "/usr/share/i18n/locales/en_NZ": {},
                "/usr/share/i18n/locales/en_PH": {},
                "/usr/share/i18n/locales/en_SC": {},
                "/usr/share/i18n/locales/en_SG": {},
                "/usr/share/i18n/locales/en_US": {},
                "/usr/share/i18n/locales/en_ZA": {},
                "/usr/share/i18n/locales/en_ZM": {},
                "/usr/share/i18n/locales/en_ZW": {},
                "/usr/share/i18n/locales/eo": {},
                "/usr/share/i18n/locales/es_AR": {},
                "/usr/share/i18n/locales/es_BO": {},
                "/usr/share/i18n/locales/es_CL": {},
                "/usr/share/i18n/locales/es_CO": {},
                "/usr/share/i18n/locales/es_CR": {},
                "/usr/share/i18n/locales/es_CU": {},
                "/usr/share/i18n/locales/es_DO": {},
                "/usr/share/i18n/locales/es_EC": {},
                "/usr/share/i18n/locales/es_ES": {},
                "/usr/share/i18n/locales/es_ES@euro": {},
                "/usr/share/i18n/locales/es_GT": {},
                "/usr/share/i18n/locales/es_HN": {},
                "/usr/share/i18n/locales/es_MX": {},
                "/usr/share/i18n/locales/es_NI": {},
                "/usr/share/i18n/locales/es_PA": {},
                "/usr/share/i18n/locales/es_PE": {},
                "/usr/share/i18n/locales/es_PR": {},
                "/usr/share/i18n/locales/es_PY": {},
                "/usr/share/i18n/locales/es_SV": {},
                "/usr/share/i18n/locales/es_US": {},
                "/usr/share/i18n/locales/es_UY": {},
                "/usr/share/i18n/locales/es_VE": {},
                "/usr/share/i18n/locales/et_EE": {},
                "/usr/share/i18n/locales/eu_ES": {},
                "/usr/share/i18n/locales/eu_ES@euro": {},
                "/usr/share/i18n/locales/fa_IR": {},
                "/usr/share/i18n/locales/ff_SN": {},
                "/usr/share/i18n/locales/fi_FI": {},
                "/usr/share/i18n/locales/fi_FI@euro": {},
                "/usr/share/i18n/locales/fil_PH": {},
                "/usr/share/i18n/locales/fo_FO": {},
                "/usr/share/i18n/locales/fr_BE": {},
                "/usr/share/i18n/locales/fr_BE@euro": {},
                "/usr/share/i18n/locales/fr_CA": {},
                "/usr/share/i18n/locales/fr_CH": {},
                "/usr/share/i18n/locales/fr_FR": {},
                "/usr/share/i18n/locales/fr_FR@euro": {},
                "/usr/share/i18n/locales/fr_LU": {},
                "/usr/share/i18n/locales/fr_LU@euro": {},
                "/usr/share/i18n/locales/fur_IT": {},
                "/usr/share/i18n/locales/fy_DE": {},
                "/usr/share/i18n/locales/fy_NL": {},
                "/usr/share/i18n/locales/ga_IE": {},
                "/usr/share/i18n/locales/ga_IE@euro": {},
                "/usr/share/i18n/locales/gd_GB": {},
                "/usr/share/i18n/locales/gez_ER": {},
                "/usr/share/i18n/locales/gez_ER@abegede": {},
                "/usr/share/i18n/locales/gez_ET": {},
                "/usr/share/i18n/locales/gez_ET@abegede": {},
                "/usr/share/i18n/locales/gl_ES": {},
                "/usr/share/i18n/locales/gl_ES@euro": {},
                "/usr/share/i18n/locales/gu_IN": {},
                "/usr/share/i18n/locales/gv_GB": {},
                "/usr/share/i18n/locales/ha_NG": {},
                "/usr/share/i18n/locales/hak_TW": {},
                "/usr/share/i18n/locales/he_IL": {},
                "/usr/share/i18n/locales/hi_IN": {},
                "/usr/share/i18n/locales/hif_FJ": {},
                "/usr/share/i18n/locales/hne_IN": {},
                "/usr/share/i18n/locales/hr_HR": {},
                "/usr/share/i18n/locales/hsb_DE": {},
                "/usr/share/i18n/locales/ht_HT": {},
                "/usr/share/i18n/locales/hu_HU": {},
                "/usr/share/i18n/locales/hy_AM": {},
                "/usr/share/i18n/locales/i18n": {},
                "/usr/share/i18n/locales/i18n_ctype": {},
                "/usr/share/i18n/locales/ia_FR": {},
                "/usr/share/i18n/locales/id_ID": {},
                "/usr/share/i18n/locales/ig_NG": {},
                "/usr/share/i18n/locales/ik_CA": {},
                "/usr/share/i18n/locales/is_IS": {},
                "/usr/share/i18n/locales/iso14651_t1": {},
                "/usr/share/i18n/locales/iso14651_t1_common": {},
                "/usr/share/i18n/locales/iso14651_t1_pinyin": {},
                "/usr/share/i18n/locales/it_CH": {},
                "/usr/share/i18n/locales/it_IT": {},
                "/usr/share/i18n/locales/it_IT@euro": {},
                "/usr/share/i18n/locales/iu_CA": {},
                "/usr/share/i18n/locales/ja_JP": {},
                "/usr/share/i18n/locales/ka_GE": {},
                "/usr/share/i18n/locales/kab_DZ": {},
                "/usr/share/i18n/locales/kk_KZ": {},
                "/usr/share/i18n/locales/kl_GL": {},
                "/usr/share/i18n/locales/km_KH": {},
                "/usr/share/i18n/locales/kn_IN": {},
                "/usr/share/i18n/locales/ko_KR": {},
                "/usr/share/i18n/locales/kok_IN": {},
                "/usr/share/i18n/locales/ks_IN": {},
                "/usr/share/i18n/locales/ks_IN@devanagari": {},
                "/usr/share/i18n/locales/ku_TR": {},
                "/usr/share/i18n/locales/kw_GB": {},
                "/usr/share/i18n/locales/ky_KG": {},
                "/usr/share/i18n/locales/lb_LU": {},
                "/usr/share/i18n/locales/lg_UG": {},
                "/usr/share/i18n/locales/li_BE": {},
                "/usr/share/i18n/locales/li_NL": {},
                "/usr/share/i18n/locales/lij_IT": {},
                "/usr/share/i18n/locales/ln_CD": {},
                "/usr/share/i18n/locales/lo_LA": {},
                "/usr/share/i18n/locales/lt_LT": {},
                "/usr/share/i18n/locales/lv_LV": {},
                "/usr/share/i18n/locales/lzh_TW": {},
                "/usr/share/i18n/locales/mag_IN": {},
                "/usr/share/i18n/locales/mai_IN": {},
                "/usr/share/i18n/locales/mai_NP": {},
                "/usr/share/i18n/locales/mfe_MU": {},
                "/usr/share/i18n/locales/mg_MG": {},
                "/usr/share/i18n/locales/mhr_RU": {},
                "/usr/share/i18n/locales/mi_NZ": {},
                "/usr/share/i18n/locales/miq_NI": {},
                "/usr/share/i18n/locales/mjw_IN": {},
                "/usr/share/i18n/locales/mk_MK": {},
                "/usr/share/i18n/locales/ml_IN": {},
                "/usr/share/i18n/locales/mn_MN": {},
                "/usr/share/i18n/locales/mni_IN": {},
                "/usr/share/i18n/locales/mnw_MM": {},
                "/usr/share/i18n/locales/mr_IN": {},
                "/usr/share/i18n/locales/ms_MY": {},
                "/usr/share/i18n/locales/mt_MT": {},
                "/usr/share/i18n/locales/my_MM": {},
                "/usr/share/i18n/locales/nan_TW": {},
                "/usr/share/i18n/locales/nan_TW@latin": {},
                "/usr/share/i18n/locales/nb_NO": {},
                "/usr/share/i18n/locales/nds_DE": {},
                "/usr/share/i18n/locales/nds_NL": {},
                "/usr/share/i18n/locales/ne_NP": {},
                "/usr/share/i18n/locales/nhn_MX": {},
                "/usr/share/i18n/locales/niu_NU": {},
                "/usr/share/i18n/locales/niu_NZ": {},
                "/usr/share/i18n/locales/nl_AW": {},
                "/usr/share/i18n/locales/nl_BE": {},
                "/usr/share/i18n/locales/nl_BE@euro": {},
                "/usr/share/i18n/locales/nl_NL": {},
                "/usr/share/i18n/locales/nl_NL@euro": {},
                "/usr/share/i18n/locales/nn_NO": {},
                "/usr/share/i18n/locales/nr_ZA": {},
                "/usr/share/i18n/locales/nso_ZA": {},
                "/usr/share/i18n/locales/oc_FR": {},
                "/usr/share/i18n/locales/om_ET": {},
                "/usr/share/i18n/locales/om_KE": {},
                "/usr/share/i18n/locales/or_IN": {},
                "/usr/share/i18n/locales/os_RU": {},
                "/usr/share/i18n/locales/pa_IN": {},
                "/usr/share/i18n/locales/pa_PK": {},
                "/usr/share/i18n/locales/pap_AW": {},
                "/usr/share/i18n/locales/pap_CW": {},
                "/usr/share/i18n/locales/pl_PL": {},
                "/usr/share/i18n/locales/ps_AF": {},
                "/usr/share/i18n/locales/pt_BR": {},
                "/usr/share/i18n/locales/pt_PT": {},
                "/usr/share/i18n/locales/pt_PT@euro": {},
                "/usr/share/i18n/locales/quz_PE": {},
                "/usr/share/i18n/locales/raj_IN": {},
                "/usr/share/i18n/locales/ro_RO": {},
                "/usr/share/i18n/locales/ru_RU": {},
                "/usr/share/i18n/locales/ru_UA": {},
                "/usr/share/i18n/locales/rw_RW": {},
                "/usr/share/i18n/locales/sa_IN": {},
                "/usr/share/i18n/locales/sah_RU": {},
                "/usr/share/i18n/locales/sat_IN": {},
                "/usr/share/i18n/locales/sc_IT": {},
                "/usr/share/i18n/locales/sd_IN": {},
                "/usr/share/i18n/locales/sd_IN@devanagari": {},
                "/usr/share/i18n/locales/se_NO": {},
                "/usr/share/i18n/locales/sgs_LT": {},
                "/usr/share/i18n/locales/shn_MM": {},
                "/usr/share/i18n/locales/shs_CA": {},
                "/usr/share/i18n/locales/si_LK": {},
                "/usr/share/i18n/locales/sid_ET": {},
                "/usr/share/i18n/locales/sk_SK": {},
                "/usr/share/i18n/locales/sl_SI": {},
                "/usr/share/i18n/locales/sm_WS": {},
                "/usr/share/i18n/locales/so_DJ": {},
                "/usr/share/i18n/locales/so_ET": {},
                "/usr/share/i18n/locales/so_KE": {},
                "/usr/share/i18n/locales/so_SO": {},
                "/usr/share/i18n/locales/sq_AL": {},
                "/usr/share/i18n/locales/sq_MK": {},
                "/usr/share/i18n/locales/sr_ME": {},
                "/usr/share/i18n/locales/sr_RS": {},
                "/usr/share/i18n/locales/sr_RS@latin": {},
                "/usr/share/i18n/locales/ss_ZA": {},
                "/usr/share/i18n/locales/st_ZA": {},
                "/usr/share/i18n/locales/sv_FI": {},
                "/usr/share/i18n/locales/sv_FI@euro": {},
                "/usr/share/i18n/locales/sv_SE": {},
                "/usr/share/i18n/locales/sw_KE": {},
                "/usr/share/i18n/locales/sw_TZ": {},
                "/usr/share/i18n/locales/szl_PL": {},
                "/usr/share/i18n/locales/ta_IN": {},
                "/usr/share/i18n/locales/ta_LK": {},
                "/usr/share/i18n/locales/tcy_IN": {},
                "/usr/share/i18n/locales/te_IN": {},
                "/usr/share/i18n/locales/tg_TJ": {},
                "/usr/share/i18n/locales/th_TH": {},
                "/usr/share/i18n/locales/the_NP": {},
                "/usr/share/i18n/locales/ti_ER": {},
                "/usr/share/i18n/locales/ti_ET": {},
                "/usr/share/i18n/locales/tig_ER": {},
                "/usr/share/i18n/locales/tk_TM": {},
                "/usr/share/i18n/locales/tl_PH": {},
                "/usr/share/i18n/locales/tn_ZA": {},
                "/usr/share/i18n/locales/to_TO": {},
                "/usr/share/i18n/locales/tpi_PG": {},
                "/usr/share/i18n/locales/tr_CY": {},
                "/usr/share/i18n/locales/tr_TR": {},
                "/usr/share/i18n/locales/translit_circle": {},
                "/usr/share/i18n/locales/translit_cjk_compat": {},
                "/usr/share/i18n/locales/translit_cjk_variants": {},
                "/usr/share/i18n/locales/translit_combining": {},
                "/usr/share/i18n/locales/translit_compat": {},
                "/usr/share/i18n/locales/translit_font": {},
                "/usr/share/i18n/locales/translit_fraction": {},
                "/usr/share/i18n/locales/translit_hangul": {},
                "/usr/share/i18n/locales/translit_narrow": {},
                "/usr/share/i18n/locales/translit_neutral": {},
                "/usr/share/i18n/locales/translit_small": {},
                "/usr/share/i18n/locales/translit_wide": {},
                "/usr/share/i18n/locales/ts_ZA": {},
                "/usr/share/i18n/locales/tt_RU": {},
                "/usr/share/i18n/locales/tt_RU@iqtelif": {},
                "/usr/share/i18n/locales/ug_CN": {},
                "/usr/share/i18n/locales/uk_UA": {},
                "/usr/share/i18n/locales/unm_US": {},
                "/usr/share/i18n/locales/ur_IN": {},
                "/usr/share/i18n/locales/ur_PK": {},
                "/usr/share/i18n/locales/uz_UZ": {},
                "/usr/share/i18n/locales/uz_UZ@cyrillic": {},
                "/usr/share/i18n/locales/ve_ZA": {},
                "/usr/share/i18n/locales/vi_VN": {},
                "/usr/share/i18n/locales/wa_BE": {},
                "/usr/share/i18n/locales/wa_BE@euro": {},
                "/usr/share/i18n/locales/wae_CH": {},
                "/usr/share/i18n/locales/wal_ET": {},
                "/usr/share/i18n/locales/wo_SN": {},
                "/usr/share/i18n/locales/xh_ZA": {},
                "/usr/share/i18n/locales/yi_US": {},
                "/usr/share/i18n/locales/yo_NG": {},
                "/usr/share/i18n/locales/yue_HK": {},
                "/usr/share/i18n/locales/yuw_PG": {},
                "/usr/share/i18n/locales/zh_CN": {},
                "/usr/share/i18n/locales/zh_HK": {},
                "/usr/share/i18n/locales/zh_SG": {},
                "/usr/share/i18n/locales/zh_TW": {},
                "/usr/share/i18n/locales/zu_ZA": {},
                "/usr/share/locale/be/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/bg/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/ca/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/cs/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/da/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/de/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/el/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/en_GB/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/eo/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/es/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/fi/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/fr/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/gl/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/hr/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/hu/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/ia/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/id/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/it/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/ja/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/ko/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/locale.alias": {},
                "/usr/share/locale/lt/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/nb/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/nl/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/pl/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/pt/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/pt_BR/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/ru/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/rw/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/sk/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/sl/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/sr/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/sv/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/tr/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/uk/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/vi/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/zh_CN/LC_MESSAGES/libc.mo": {},
                "/usr/share/locale/zh_TW/LC_MESSAGES/libc.mo": {},
                "/usr/share/man/man5/locale.gen.5.gz": {},
                "/usr/share/man/man8/locale-gen.8.gz": {},
                "/var/db/Makefile": {},
            },
        },
        {
            "name": "sys-libs/ncurses",
            "slot": "0",
            "content": {
                "/etc/env.d/50ncurses": {},
                "/etc/terminfo/a/ansi": {},
                "/etc/terminfo/d/dumb": {},
                "/etc/terminfo/h/hterm": {},
                "/etc/terminfo/h/hterm-256color": {},
                "/etc/terminfo/l/linux": {},
                "/etc/terminfo/r/rxvt": {},
                "/etc/terminfo/r/rxvt-256color": {},
                "/etc/terminfo/r/rxvt-unicode": {},
                "/etc/terminfo/r/rxvt-unicode-256color": {},
                "/etc/terminfo/s/screen": {},
                "/etc/terminfo/s/screen-256color": {},
                "/etc/terminfo/s/screen.xterm-256color": {},
                "/etc/terminfo/v/vt100": {},
                "/etc/terminfo/v/vt102": {},
                "/etc/terminfo/v/vt200": {
                    "type": "Symlink",
                    "target": "/etc/terminfo/v/vt220",
                },
                "/etc/terminfo/v/vt220": {},
                "/etc/terminfo/v/vt52": {},
                "/etc/terminfo/x/xterm": {},
                "/etc/terminfo/x/xterm-256color": {},
                "/etc/terminfo/x/xterm-color": {},
                "/usr/bin/captoinfo": {
                    "type": "Symlink",
                    "target": "/usr/bin/tic",
                },
                "/usr/bin/clear": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libtinfotw.so.6": "/usr/lib64/libtinfotw.so.6",
                    },
                },
                "/usr/bin/infocmp": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libtinfotw.so.6": "/usr/lib64/libtinfotw.so.6",
                    },
                },
                "/usr/bin/infotocap": {
                    "type": "Symlink",
                    "target": "/usr/bin/tic",
                },
                "/usr/bin/ncurses6-config": {},
                "/usr/bin/ncursest6-config": {},
                "/usr/bin/ncursestw6-config": {},
                "/usr/bin/ncursesw6-config": {},
                "/usr/bin/reset": {
                    "type": "Symlink",
                    "target": "/usr/bin/tset",
                },
                "/usr/bin/tabs": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libtinfotw.so.6": "/usr/lib64/libtinfotw.so.6",
                    },
                },
                "/usr/bin/tic": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libtinfotw.so.6": "/usr/lib64/libtinfotw.so.6",
                    },
                },
                "/usr/bin/toe": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libtinfotw.so.6": "/usr/lib64/libtinfotw.so.6",
                    },
                },
                "/usr/bin/tput": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libtinfotw.so.6": "/usr/lib64/libtinfotw.so.6",
                    },
                },
                "/usr/bin/tset": {
                    "type": "ElfBinary",
                    "libs": {
                        "libc.so.6": "/lib64/libc.so.6",
                        "libtinfotw.so.6": "/usr/lib64/libtinfotw.so.6",
                    },
                },
                "/usr/include/curses.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/cursesapp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/cursesf.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/cursesm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/cursesp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/cursesw.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/cursslk.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/eti.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/etip.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/form.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/menu.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/nc_tparm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncurses.h": {
                    "type": "Symlink",
                    "target": "/usr/include/curses.h",
                },
                "/usr/include/ncurses_dll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/curses.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/cursesapp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/cursesf.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/cursesm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/cursesp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/cursesw.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/cursslk.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/eti.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/etip.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/form.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/menu.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/nc_tparm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/ncurses.h": {
                    "type": "Symlink",
                    "target": "/usr/include/ncursest/curses.h",
                },
                "/usr/include/ncursest/ncurses_dll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/panel.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/term.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/term_entry.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/termcap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/tic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursest/unctrl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/curses.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/cursesapp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/cursesf.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/cursesm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/cursesp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/cursesw.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/cursslk.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/eti.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/etip.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/form.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/menu.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/nc_tparm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/ncurses.h": {
                    "type": "Symlink",
                    "target": "/usr/include/ncursestw/curses.h",
                },
                "/usr/include/ncursestw/ncurses_dll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/panel.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/term.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/term_entry.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/termcap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/tic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursestw/unctrl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/curses.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/cursesapp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/cursesf.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/cursesm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/cursesp.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/cursesw.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/cursslk.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/eti.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/etip.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/form.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/menu.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/nc_tparm.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/ncurses.h": {
                    "type": "Symlink",
                    "target": "/usr/include/ncursesw/curses.h",
                },
                "/usr/include/ncursesw/ncurses_dll.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/panel.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/term.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/term_entry.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/termcap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/tic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/ncursesw/unctrl.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/panel.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/term.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/term_entry.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/termcap.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/tic.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/unctrl.h": {
                    "type": "HeaderFile",
                },
                "/usr/lib64/libcurses.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses.so.6.3",
                },
                "/usr/lib64/libform.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libform.so.6.3",
                },
                "/usr/lib64/libform.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libform.so.6.3",
                },
                "/usr/lib64/libform.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libformt.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libformt.so.6.3",
                },
                "/usr/lib64/libformt.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libformt.so.6.3",
                },
                "/usr/lib64/libformt.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libformtw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libformtw.so.6.3",
                },
                "/usr/lib64/libformtw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libformtw.so.6.3",
                },
                "/usr/lib64/libformtw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libformw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libformw.so.6.3",
                },
                "/usr/lib64/libformw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libformw.so.6.3",
                },
                "/usr/lib64/libformw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libmenu.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenu.so.6.3",
                },
                "/usr/lib64/libmenu.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenu.so.6.3",
                },
                "/usr/lib64/libmenu.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libmenut.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenut.so.6.3",
                },
                "/usr/lib64/libmenut.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenut.so.6.3",
                },
                "/usr/lib64/libmenut.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libmenutw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenutw.so.6.3",
                },
                "/usr/lib64/libmenutw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenutw.so.6.3",
                },
                "/usr/lib64/libmenutw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libmenuw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenuw.so.6.3",
                },
                "/usr/lib64/libmenuw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libmenuw.so.6.3",
                },
                "/usr/lib64/libmenuw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncurses++.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++.so.6.3",
                },
                "/usr/lib64/libncurses++.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++.so.6.3",
                },
                "/usr/lib64/libncurses++.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncurses++t.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++t.so.6.3",
                },
                "/usr/lib64/libncurses++t.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++t.so.6.3",
                },
                "/usr/lib64/libncurses++t.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncurses++tw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++tw.so.6.3",
                },
                "/usr/lib64/libncurses++tw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++tw.so.6.3",
                },
                "/usr/lib64/libncurses++tw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncurses++w.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++w.so.6.3",
                },
                "/usr/lib64/libncurses++w.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses++w.so.6.3",
                },
                "/usr/lib64/libncurses++w.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncurses.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses.so.6.3",
                },
                "/usr/lib64/libncurses.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncurses.so.6.3",
                },
                "/usr/lib64/libncurses.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncursest.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncursest.so.6.3",
                },
                "/usr/lib64/libncursest.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncursest.so.6.3",
                },
                "/usr/lib64/libncursest.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncursestw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncursestw.so.6.3",
                },
                "/usr/lib64/libncursestw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncursestw.so.6.3",
                },
                "/usr/lib64/libncursestw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libncursesw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncursesw.so.6.3",
                },
                "/usr/lib64/libncursesw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libncursesw.so.6.3",
                },
                "/usr/lib64/libncursesw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libpanel.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpanel.so.6.3",
                },
                "/usr/lib64/libpanel.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpanel.so.6.3",
                },
                "/usr/lib64/libpanel.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libpanelt.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpanelt.so.6.3",
                },
                "/usr/lib64/libpanelt.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpanelt.so.6.3",
                },
                "/usr/lib64/libpanelt.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libpaneltw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpaneltw.so.6.3",
                },
                "/usr/lib64/libpaneltw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpaneltw.so.6.3",
                },
                "/usr/lib64/libpaneltw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libpanelw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpanelw.so.6.3",
                },
                "/usr/lib64/libpanelw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libpanelw.so.6.3",
                },
                "/usr/lib64/libpanelw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libtinfo.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfo.so.6.3",
                },
                "/usr/lib64/libtinfo.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfo.so.6.3",
                },
                "/usr/lib64/libtinfo.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libtinfot.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfot.so.6.3",
                },
                "/usr/lib64/libtinfot.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfot.so.6.3",
                },
                "/usr/lib64/libtinfot.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libtinfotw.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfotw.so.6.3",
                },
                "/usr/lib64/libtinfotw.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfotw.so.6.3",
                },
                "/usr/lib64/libtinfotw.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/libtinfow.so": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfow.so.6.3",
                },
                "/usr/lib64/libtinfow.so.6": {
                    "type": "Symlink",
                    "target": "/usr/lib64/libtinfow.so.6.3",
                },
                "/usr/lib64/libtinfow.so.6.3": {
                    "type": "SharedLibrary",
                },
                "/usr/lib64/pkgconfig/form.pc": {},
                "/usr/lib64/pkgconfig/formt.pc": {},
                "/usr/lib64/pkgconfig/formtw.pc": {},
                "/usr/lib64/pkgconfig/formw.pc": {},
                "/usr/lib64/pkgconfig/menu.pc": {},
                "/usr/lib64/pkgconfig/menut.pc": {},
                "/usr/lib64/pkgconfig/menutw.pc": {},
                "/usr/lib64/pkgconfig/menuw.pc": {},
                "/usr/lib64/pkgconfig/ncurses++.pc": {},
                "/usr/lib64/pkgconfig/ncurses++t.pc": {},
                "/usr/lib64/pkgconfig/ncurses++tw.pc": {},
                "/usr/lib64/pkgconfig/ncurses++w.pc": {},
                "/usr/lib64/pkgconfig/ncurses.pc": {},
                "/usr/lib64/pkgconfig/ncursest.pc": {},
                "/usr/lib64/pkgconfig/ncursestw.pc": {},
                "/usr/lib64/pkgconfig/ncursesw.pc": {},
                "/usr/lib64/pkgconfig/panel.pc": {},
                "/usr/lib64/pkgconfig/panelt.pc": {},
                "/usr/lib64/pkgconfig/paneltw.pc": {},
                "/usr/lib64/pkgconfig/panelw.pc": {},
                "/usr/lib64/pkgconfig/tinfo.pc": {},
                "/usr/lib64/pkgconfig/tinfot.pc": {},
                "/usr/lib64/pkgconfig/tinfotw.pc": {},
                "/usr/lib64/pkgconfig/tinfow.pc": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/ANNOUNCE.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/MANIFEST.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/NEWS.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/README.MinGW.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/README.emx.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/README.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/TO-DO.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/hackguide.doc.gz": {},
                "/usr/share/doc/ncurses-6.3_p20220423-r1/ncurses-intro.doc.gz": {},
                "/usr/share/man/man1/captoinfo.1m.gz": {},
                "/usr/share/man/man1/clear.1.gz": {},
                "/usr/share/man/man1/infocmp.1m.gz": {},
                "/usr/share/man/man1/infotocap.1m.gz": {},
                "/usr/share/man/man1/ncurses6-config.1.gz": {},
                "/usr/share/man/man1/ncursest6-config.1.gz": {},
                "/usr/share/man/man1/ncursestw6-config.1.gz": {},
                "/usr/share/man/man1/ncursesw6-config.1.gz": {},
                "/usr/share/man/man1/reset.1.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man1/tset.1.gz",
                },
                "/usr/share/man/man1/tabs.1.gz": {},
                "/usr/share/man/man1/tic.1m.gz": {},
                "/usr/share/man/man1/toe.1m.gz": {},
                "/usr/share/man/man1/tput.1.gz": {},
                "/usr/share/man/man1/tset.1.gz": {},
                "/usr/share/man/man3/BC.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/COLORS.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/COLOR_PAIR.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/COLOR_PAIRS.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/COLS.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/ESCDELAY.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/LINES.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/PAIR_NUMBER.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/PC.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/SP.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/TABSIZE.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/TYPE_ALNUM.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_variables.3x.gz",
                },
                "/usr/share/man/man3/TYPE_ALPHA.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_variables.3x.gz",
                },
                "/usr/share/man/man3/TYPE_ENUM.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_variables.3x.gz",
                },
                "/usr/share/man/man3/TYPE_INTEGER.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_variables.3x.gz",
                },
                "/usr/share/man/man3/TYPE_IPV4.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_variables.3x.gz",
                },
                "/usr/share/man/man3/TYPE_NUMERIC.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_variables.3x.gz",
                },
                "/usr/share/man/man3/TYPE_REGEXP.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_variables.3x.gz",
                },
                "/usr/share/man/man3/UP.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/_nc_free_and_exit.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_memleaks.3x.gz",
                },
                "/usr/share/man/man3/_nc_free_tinfo.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_memleaks.3x.gz",
                },
                "/usr/share/man/man3/_nc_freeall.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_memleaks.3x.gz",
                },
                "/usr/share/man/man3/_nc_tracebits.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_traceattr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_traceattr2.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracecchar_t.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracecchar_t2.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracechar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracechtype.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracechtype2.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracedump.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracef.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/_tracemouse.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/acs_map.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/add_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wch.3x.gz",
                },
                "/usr/share/man/man3/add_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/add_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/addch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addch.3x.gz",
                },
                "/usr/share/man/man3/addchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/addchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/addnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/addnwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/addstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/addwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/alloc_pair.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/new_pair.3x.gz",
                },
                "/usr/share/man/man3/alloc_pair_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/assume_default_colors.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/default_colors.3x.gz",
                },
                "/usr/share/man/man3/assume_default_colors_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/attr_get.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/attr_off.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/attr_on.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/attr_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/attroff.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/attron.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/attrset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/baudrate.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/baudrate_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/beep.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_beep.3x.gz",
                },
                "/usr/share/man/man3/beep_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/bkgd.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgd.3x.gz",
                },
                "/usr/share/man/man3/bkgdset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgd.3x.gz",
                },
                "/usr/share/man/man3/bkgrnd.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgrnd.3x.gz",
                },
                "/usr/share/man/man3/bkgrndset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgrnd.3x.gz",
                },
                "/usr/share/man/man3/boolcodes.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/boolfnames.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/boolnames.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/border.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/border_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/bottom_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/box.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/box_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/can_change_color.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/can_change_color_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/cbreak.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/cbreak_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/ceiling_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/chgat.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/clear.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/clearok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/clrtobot.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/clrtoeol.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/color_content.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/color_content_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/color_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/copywin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_overlay.3x.gz",
                },
                "/usr/share/man/man3/cur_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/current_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_page.3x.gz",
                },
                "/usr/share/man/man3/current_item.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_current.3x.gz",
                },
                "/usr/share/man/man3/curs_add_wch.3x.gz": {},
                "/usr/share/man/man3/curs_add_wchstr.3x.gz": {},
                "/usr/share/man/man3/curs_addch.3x.gz": {},
                "/usr/share/man/man3/curs_addchstr.3x.gz": {},
                "/usr/share/man/man3/curs_addstr.3x.gz": {},
                "/usr/share/man/man3/curs_addwstr.3x.gz": {},
                "/usr/share/man/man3/curs_attr.3x.gz": {},
                "/usr/share/man/man3/curs_beep.3x.gz": {},
                "/usr/share/man/man3/curs_bkgd.3x.gz": {},
                "/usr/share/man/man3/curs_bkgrnd.3x.gz": {},
                "/usr/share/man/man3/curs_border.3x.gz": {},
                "/usr/share/man/man3/curs_border_set.3x.gz": {},
                "/usr/share/man/man3/curs_clear.3x.gz": {},
                "/usr/share/man/man3/curs_color.3x.gz": {},
                "/usr/share/man/man3/curs_delch.3x.gz": {},
                "/usr/share/man/man3/curs_deleteln.3x.gz": {},
                "/usr/share/man/man3/curs_extend.3x.gz": {},
                "/usr/share/man/man3/curs_get_wch.3x.gz": {},
                "/usr/share/man/man3/curs_get_wstr.3x.gz": {},
                "/usr/share/man/man3/curs_getcchar.3x.gz": {},
                "/usr/share/man/man3/curs_getch.3x.gz": {},
                "/usr/share/man/man3/curs_getstr.3x.gz": {},
                "/usr/share/man/man3/curs_getyx.3x.gz": {},
                "/usr/share/man/man3/curs_in_wch.3x.gz": {},
                "/usr/share/man/man3/curs_in_wchstr.3x.gz": {},
                "/usr/share/man/man3/curs_inch.3x.gz": {},
                "/usr/share/man/man3/curs_inchstr.3x.gz": {},
                "/usr/share/man/man3/curs_initscr.3x.gz": {},
                "/usr/share/man/man3/curs_inopts.3x.gz": {},
                "/usr/share/man/man3/curs_ins_wch.3x.gz": {},
                "/usr/share/man/man3/curs_ins_wstr.3x.gz": {},
                "/usr/share/man/man3/curs_insch.3x.gz": {},
                "/usr/share/man/man3/curs_insstr.3x.gz": {},
                "/usr/share/man/man3/curs_instr.3x.gz": {},
                "/usr/share/man/man3/curs_inwstr.3x.gz": {},
                "/usr/share/man/man3/curs_kernel.3x.gz": {},
                "/usr/share/man/man3/curs_legacy.3x.gz": {},
                "/usr/share/man/man3/curs_memleaks.3x.gz": {},
                "/usr/share/man/man3/curs_mouse.3x.gz": {},
                "/usr/share/man/man3/curs_move.3x.gz": {},
                "/usr/share/man/man3/curs_opaque.3x.gz": {},
                "/usr/share/man/man3/curs_outopts.3x.gz": {},
                "/usr/share/man/man3/curs_overlay.3x.gz": {},
                "/usr/share/man/man3/curs_pad.3x.gz": {},
                "/usr/share/man/man3/curs_print.3x.gz": {},
                "/usr/share/man/man3/curs_printw.3x.gz": {},
                "/usr/share/man/man3/curs_refresh.3x.gz": {},
                "/usr/share/man/man3/curs_scanw.3x.gz": {},
                "/usr/share/man/man3/curs_scr_dump.3x.gz": {},
                "/usr/share/man/man3/curs_scroll.3x.gz": {},
                "/usr/share/man/man3/curs_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/curs_set_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/curs_slk.3x.gz": {},
                "/usr/share/man/man3/curs_sp_funcs.3x.gz": {},
                "/usr/share/man/man3/curs_termattrs.3x.gz": {},
                "/usr/share/man/man3/curs_termcap.3x.gz": {},
                "/usr/share/man/man3/curs_terminfo.3x.gz": {},
                "/usr/share/man/man3/curs_threads.3x.gz": {},
                "/usr/share/man/man3/curs_touch.3x.gz": {},
                "/usr/share/man/man3/curs_trace.3x.gz": {},
                "/usr/share/man/man3/curs_util.3x.gz": {},
                "/usr/share/man/man3/curs_variables.3x.gz": {},
                "/usr/share/man/man3/curs_window.3x.gz": {},
                "/usr/share/man/man3/curscr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/curses_trace.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/curses_version.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_extend.3x.gz",
                },
                "/usr/share/man/man3/data_ahead.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_data.3x.gz",
                },
                "/usr/share/man/man3/data_behind.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_data.3x.gz",
                },
                "/usr/share/man/man3/def_prog_mode.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/def_prog_mode_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/def_shell_mode.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/def_shell_mode_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/default_colors.3x.gz": {},
                "/usr/share/man/man3/define_key.3x.gz": {},
                "/usr/share/man/man3/define_key_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/del_curterm.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/del_curterm_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/del_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/delay_output.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/delay_output_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/delch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_delch.3x.gz",
                },
                "/usr/share/man/man3/deleteln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_deleteln.3x.gz",
                },
                "/usr/share/man/man3/delscreen.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_initscr.3x.gz",
                },
                "/usr/share/man/man3/delwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/derwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/doupdate.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_refresh.3x.gz",
                },
                "/usr/share/man/man3/doupdate_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/dup_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_new.3x.gz",
                },
                "/usr/share/man/man3/dupwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/dynamic_field_info.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_info.3x.gz",
                },
                "/usr/share/man/man3/echo.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/echo_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/echo_wchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wch.3x.gz",
                },
                "/usr/share/man/man3/echochar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addch.3x.gz",
                },
                "/usr/share/man/man3/endwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_initscr.3x.gz",
                },
                "/usr/share/man/man3/endwin_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/erase.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/erasechar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/erasechar_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/erasewchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/erasewchar_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/exit_curses.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_memleaks.3x.gz",
                },
                "/usr/share/man/man3/exit_terminfo.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_memleaks.3x.gz",
                },
                "/usr/share/man/man3/extended_color_content.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/extended_color_content_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/extended_pair_content.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/extended_pair_content_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/extended_slk_color.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/extended_slk_color_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/field_arg.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_validation.3x.gz",
                },
                "/usr/share/man/man3/field_back.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_attributes.3x.gz",
                },
                "/usr/share/man/man3/field_buffer.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_buffer.3x.gz",
                },
                "/usr/share/man/man3/field_count.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field.3x.gz",
                },
                "/usr/share/man/man3/field_fore.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_attributes.3x.gz",
                },
                "/usr/share/man/man3/field_index.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_page.3x.gz",
                },
                "/usr/share/man/man3/field_info.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_info.3x.gz",
                },
                "/usr/share/man/man3/field_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/field_just.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_just.3x.gz",
                },
                "/usr/share/man/man3/field_opts.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_opts.3x.gz",
                },
                "/usr/share/man/man3/field_opts_off.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_opts.3x.gz",
                },
                "/usr/share/man/man3/field_opts_on.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_opts.3x.gz",
                },
                "/usr/share/man/man3/field_pad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_attributes.3x.gz",
                },
                "/usr/share/man/man3/field_status.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_buffer.3x.gz",
                },
                "/usr/share/man/man3/field_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/field_type.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_validation.3x.gz",
                },
                "/usr/share/man/man3/field_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_userptr.3x.gz",
                },
                "/usr/share/man/man3/filter.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/filter_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/find_pair.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/new_pair.3x.gz",
                },
                "/usr/share/man/man3/find_pair_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/flash.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_beep.3x.gz",
                },
                "/usr/share/man/man3/flash_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/flushinp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/flushinp_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/form.3x.gz": {},
                "/usr/share/man/man3/form_cursor.3x.gz": {},
                "/usr/share/man/man3/form_data.3x.gz": {},
                "/usr/share/man/man3/form_driver.3x.gz": {},
                "/usr/share/man/man3/form_driver_w.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_driver.3x.gz",
                },
                "/usr/share/man/man3/form_field.3x.gz": {},
                "/usr/share/man/man3/form_field_attributes.3x.gz": {},
                "/usr/share/man/man3/form_field_buffer.3x.gz": {},
                "/usr/share/man/man3/form_field_info.3x.gz": {},
                "/usr/share/man/man3/form_field_just.3x.gz": {},
                "/usr/share/man/man3/form_field_new.3x.gz": {},
                "/usr/share/man/man3/form_field_opts.3x.gz": {},
                "/usr/share/man/man3/form_field_userptr.3x.gz": {},
                "/usr/share/man/man3/form_field_validation.3x.gz": {},
                "/usr/share/man/man3/form_fields.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field.3x.gz",
                },
                "/usr/share/man/man3/form_fieldtype.3x.gz": {},
                "/usr/share/man/man3/form_hook.3x.gz": {},
                "/usr/share/man/man3/form_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/form_new.3x.gz": {},
                "/usr/share/man/man3/form_new_page.3x.gz": {},
                "/usr/share/man/man3/form_opts.3x.gz": {},
                "/usr/share/man/man3/form_opts_off.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_opts.3x.gz",
                },
                "/usr/share/man/man3/form_opts_on.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_opts.3x.gz",
                },
                "/usr/share/man/man3/form_page.3x.gz": {},
                "/usr/share/man/man3/form_post.3x.gz": {},
                "/usr/share/man/man3/form_request_by_name.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_requestname.3x.gz",
                },
                "/usr/share/man/man3/form_request_name.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_requestname.3x.gz",
                },
                "/usr/share/man/man3/form_requestname.3x.gz": {},
                "/usr/share/man/man3/form_sub.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_win.3x.gz",
                },
                "/usr/share/man/man3/form_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/form_userptr.3x.gz": {},
                "/usr/share/man/man3/form_variables.3x.gz": {},
                "/usr/share/man/man3/form_win.3x.gz": {},
                "/usr/share/man/man3/free_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_new.3x.gz",
                },
                "/usr/share/man/man3/free_fieldtype.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_fieldtype.3x.gz",
                },
                "/usr/share/man/man3/free_form.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_new.3x.gz",
                },
                "/usr/share/man/man3/free_item.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_new.3x.gz",
                },
                "/usr/share/man/man3/free_menu.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_new.3x.gz",
                },
                "/usr/share/man/man3/free_pair.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/new_pair.3x.gz",
                },
                "/usr/share/man/man3/free_pair_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/get_escdelay.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_threads.3x.gz",
                },
                "/usr/share/man/man3/get_escdelay_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/get_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wch.3x.gz",
                },
                "/usr/share/man/man3/get_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/getattrs.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getbegx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getbegy.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getbegyx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getyx.3x.gz",
                },
                "/usr/share/man/man3/getbkgd.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgd.3x.gz",
                },
                "/usr/share/man/man3/getbkgrnd.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgrnd.3x.gz",
                },
                "/usr/share/man/man3/getcchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getcchar.3x.gz",
                },
                "/usr/share/man/man3/getch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getch.3x.gz",
                },
                "/usr/share/man/man3/getcurx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getcury.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getmaxx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getmaxy.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getmaxyx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getyx.3x.gz",
                },
                "/usr/share/man/man3/getmouse.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/getmouse_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/getn_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/getnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/getparx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getpary.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_legacy.3x.gz",
                },
                "/usr/share/man/man3/getparyx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getyx.3x.gz",
                },
                "/usr/share/man/man3/getstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/getsyx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/getwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/getwin_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/getyx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getyx.3x.gz",
                },
                "/usr/share/man/man3/ground_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/halfdelay.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/halfdelay_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/has_colors.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/has_colors_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/has_ic.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/has_ic_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/has_il.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/has_il_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/has_key.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getch.3x.gz",
                },
                "/usr/share/man/man3/has_key_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/has_mouse.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/has_mouse_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/hide_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/hline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/hline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/idcok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/idlok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/immedok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/in_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wch.3x.gz",
                },
                "/usr/share/man/man3/in_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/in_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/inch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inch.3x.gz",
                },
                "/usr/share/man/man3/inchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/inchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/init_color.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/init_color_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/init_extended_color.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/init_extended_color_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/init_extended_pair.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/init_extended_pair_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/init_pair.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/init_pair_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/initscr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_initscr.3x.gz",
                },
                "/usr/share/man/man3/innstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/innwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/ins_nwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/ins_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wch.3x.gz",
                },
                "/usr/share/man/man3/ins_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/insch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insch.3x.gz",
                },
                "/usr/share/man/man3/insdelln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_deleteln.3x.gz",
                },
                "/usr/share/man/man3/insertln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_deleteln.3x.gz",
                },
                "/usr/share/man/man3/insnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/insstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/instr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/intrflush.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/intrflush_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/inwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/is_cleared.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_idcok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_idlok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_immedok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_keypad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_leaveok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_linetouched.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_touch.3x.gz",
                },
                "/usr/share/man/man3/is_nodelay.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_notimeout.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_pad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_scrollok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_subwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_syncok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/is_term_resized.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/resizeterm.3x.gz",
                },
                "/usr/share/man/man3/is_term_resized_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/is_wintouched.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_touch.3x.gz",
                },
                "/usr/share/man/man3/isendwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_initscr.3x.gz",
                },
                "/usr/share/man/man3/isendwin_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/item_count.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_items.3x.gz",
                },
                "/usr/share/man/man3/item_description.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_name.3x.gz",
                },
                "/usr/share/man/man3/item_index.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_current.3x.gz",
                },
                "/usr/share/man/man3/item_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/item_name.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_name.3x.gz",
                },
                "/usr/share/man/man3/item_opts.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_opts.3x.gz",
                },
                "/usr/share/man/man3/item_opts_off.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_opts.3x.gz",
                },
                "/usr/share/man/man3/item_opts_on.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_opts.3x.gz",
                },
                "/usr/share/man/man3/item_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/item_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_userptr.3x.gz",
                },
                "/usr/share/man/man3/item_value.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_value.3x.gz",
                },
                "/usr/share/man/man3/item_visible.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_visible.3x.gz",
                },
                "/usr/share/man/man3/key_defined.3x.gz": {},
                "/usr/share/man/man3/key_defined_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/key_name.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/keybound.3x.gz": {},
                "/usr/share/man/man3/keybound_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/keyname.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/keyname_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/keyok.3x.gz": {},
                "/usr/share/man/man3/keyok_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/keypad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/killchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/killchar_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/killwchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/killwchar_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/leaveok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/legacy_coding.3x.gz": {},
                "/usr/share/man/man3/link_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_new.3x.gz",
                },
                "/usr/share/man/man3/link_fieldtype.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_fieldtype.3x.gz",
                },
                "/usr/share/man/man3/longname.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/longname_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/mcprint.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_print.3x.gz",
                },
                "/usr/share/man/man3/mcprint_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/menu.3x.gz": {},
                "/usr/share/man/man3/menu_attributes.3x.gz": {},
                "/usr/share/man/man3/menu_back.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/menu_cursor.3x.gz": {},
                "/usr/share/man/man3/menu_driver.3x.gz": {},
                "/usr/share/man/man3/menu_fore.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/menu_format.3x.gz": {},
                "/usr/share/man/man3/menu_grey.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/menu_hook.3x.gz": {},
                "/usr/share/man/man3/menu_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/menu_items.3x.gz": {},
                "/usr/share/man/man3/menu_mark.3x.gz": {},
                "/usr/share/man/man3/menu_new.3x.gz": {},
                "/usr/share/man/man3/menu_opts.3x.gz": {},
                "/usr/share/man/man3/menu_opts_off.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_opts.3x.gz",
                },
                "/usr/share/man/man3/menu_opts_on.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_opts.3x.gz",
                },
                "/usr/share/man/man3/menu_pad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/menu_pattern.3x.gz": {},
                "/usr/share/man/man3/menu_post.3x.gz": {},
                "/usr/share/man/man3/menu_request_by_name.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_requestname.3x.gz",
                },
                "/usr/share/man/man3/menu_request_name.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_requestname.3x.gz",
                },
                "/usr/share/man/man3/menu_requestname.3x.gz": {},
                "/usr/share/man/man3/menu_spacing.3x.gz": {},
                "/usr/share/man/man3/menu_sub.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_win.3x.gz",
                },
                "/usr/share/man/man3/menu_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/menu_userptr.3x.gz": {},
                "/usr/share/man/man3/menu_win.3x.gz": {},
                "/usr/share/man/man3/meta.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/mitem_current.3x.gz": {},
                "/usr/share/man/man3/mitem_name.3x.gz": {},
                "/usr/share/man/man3/mitem_new.3x.gz": {},
                "/usr/share/man/man3/mitem_opts.3x.gz": {},
                "/usr/share/man/man3/mitem_userptr.3x.gz": {},
                "/usr/share/man/man3/mitem_value.3x.gz": {},
                "/usr/share/man/man3/mitem_visible.3x.gz": {},
                "/usr/share/man/man3/mouse_trafo.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/mouseinterval.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/mouseinterval_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/mousemask.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/mousemask_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/move.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_move.3x.gz",
                },
                "/usr/share/man/man3/move_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field.3x.gz",
                },
                "/usr/share/man/man3/move_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/mvadd_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wch.3x.gz",
                },
                "/usr/share/man/man3/mvadd_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvadd_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvaddch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addch.3x.gz",
                },
                "/usr/share/man/man3/mvaddchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/mvaddchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/mvaddnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/mvaddnwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/mvaddstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/mvaddwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/mvchgat.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/mvcur.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/mvcur_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/mvdelch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_delch.3x.gz",
                },
                "/usr/share/man/man3/mvderwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/mvget_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wch.3x.gz",
                },
                "/usr/share/man/man3/mvget_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvgetch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getch.3x.gz",
                },
                "/usr/share/man/man3/mvgetn_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvgetnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/mvgetstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/mvhline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/mvhline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/mvin_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wch.3x.gz",
                },
                "/usr/share/man/man3/mvin_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvin_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvinch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inch.3x.gz",
                },
                "/usr/share/man/man3/mvinchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/mvinchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/mvinnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/mvinnwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/mvins_nwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvins_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wch.3x.gz",
                },
                "/usr/share/man/man3/mvins_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvinsch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insch.3x.gz",
                },
                "/usr/share/man/man3/mvinsnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/mvinsstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/mvinstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/mvinwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/mvprintw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_printw.3x.gz",
                },
                "/usr/share/man/man3/mvscanw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scanw.3x.gz",
                },
                "/usr/share/man/man3/mvvline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/mvvline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/mvwadd_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wch.3x.gz",
                },
                "/usr/share/man/man3/mvwadd_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwadd_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwaddch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addch.3x.gz",
                },
                "/usr/share/man/man3/mvwaddchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwaddchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwaddnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/mvwaddnwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/mvwaddstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/mvwaddwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/mvwchgat.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/mvwdelch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_delch.3x.gz",
                },
                "/usr/share/man/man3/mvwget_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wch.3x.gz",
                },
                "/usr/share/man/man3/mvwget_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvwgetch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getch.3x.gz",
                },
                "/usr/share/man/man3/mvwgetn_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvwgetnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/mvwgetstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/mvwhline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/mvwhline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/mvwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/mvwin_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wch.3x.gz",
                },
                "/usr/share/man/man3/mvwin_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwin_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwinch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inch.3x.gz",
                },
                "/usr/share/man/man3/mvwinchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwinchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/mvwinnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/mvwinnwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/mvwins_nwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvwins_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wch.3x.gz",
                },
                "/usr/share/man/man3/mvwins_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/mvwinsch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insch.3x.gz",
                },
                "/usr/share/man/man3/mvwinsnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/mvwinsstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/mvwinstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/mvwinwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/mvwprintw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_printw.3x.gz",
                },
                "/usr/share/man/man3/mvwscanw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scanw.3x.gz",
                },
                "/usr/share/man/man3/mvwvline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/mvwvline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/napms.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/napms_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/ncurses.3x.gz": {},
                "/usr/share/man/man3/new_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_new.3x.gz",
                },
                "/usr/share/man/man3/new_fieldtype.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_fieldtype.3x.gz",
                },
                "/usr/share/man/man3/new_form.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_new.3x.gz",
                },
                "/usr/share/man/man3/new_form_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/new_item.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_new.3x.gz",
                },
                "/usr/share/man/man3/new_menu.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_new.3x.gz",
                },
                "/usr/share/man/man3/new_menu_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/new_page.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_new_page.3x.gz",
                },
                "/usr/share/man/man3/new_pair.3x.gz": {},
                "/usr/share/man/man3/new_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/new_prescr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/newpad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_pad.3x.gz",
                },
                "/usr/share/man/man3/newpad_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/newscr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/newterm.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_initscr.3x.gz",
                },
                "/usr/share/man/man3/newterm_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/newwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/newwin_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/nl.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/nl_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/nocbreak.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/nocbreak_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/nodelay.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/noecho.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/noecho_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/nofilter.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/nofilter_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/nonl.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/nonl_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/noqiflush.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/noqiflush_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/noraw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/noraw_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/notimeout.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/numcodes.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/numfnames.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/numnames.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/ospeed.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/overlay.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_overlay.3x.gz",
                },
                "/usr/share/man/man3/overwrite.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_overlay.3x.gz",
                },
                "/usr/share/man/man3/pair_content.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/pair_content_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/panel.3x.gz": {},
                "/usr/share/man/man3/panel_above.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/panel_below.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/panel_hidden.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/panel_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/panel_window.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/pecho_wchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_pad.3x.gz",
                },
                "/usr/share/man/man3/pechochar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_pad.3x.gz",
                },
                "/usr/share/man/man3/pnoutrefresh.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_pad.3x.gz",
                },
                "/usr/share/man/man3/pos_form_cursor.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_cursor.3x.gz",
                },
                "/usr/share/man/man3/pos_menu_cursor.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_cursor.3x.gz",
                },
                "/usr/share/man/man3/post_form.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_post.3x.gz",
                },
                "/usr/share/man/man3/post_menu.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_post.3x.gz",
                },
                "/usr/share/man/man3/prefresh.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_pad.3x.gz",
                },
                "/usr/share/man/man3/printw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_printw.3x.gz",
                },
                "/usr/share/man/man3/putp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/putp_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/putwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/qiflush.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/qiflush_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/raw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/raw_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/redrawwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_refresh.3x.gz",
                },
                "/usr/share/man/man3/refresh.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_refresh.3x.gz",
                },
                "/usr/share/man/man3/replace_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/reset_color_pairs.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/reset_color_pairs_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/reset_prog_mode.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/reset_prog_mode_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/reset_shell_mode.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/reset_shell_mode_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/resetty.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/resetty_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/resize_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/resizeterm.3x.gz",
                },
                "/usr/share/man/man3/resize_term_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/resizeterm.3x.gz": {},
                "/usr/share/man/man3/resizeterm_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/restartterm.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/restartterm_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/ripoffline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/ripoffline_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/savetty.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/savetty_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/scale_form.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_win.3x.gz",
                },
                "/usr/share/man/man3/scale_menu.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_win.3x.gz",
                },
                "/usr/share/man/man3/scanw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scanw.3x.gz",
                },
                "/usr/share/man/man3/scr_dump.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scr_dump.3x.gz",
                },
                "/usr/share/man/man3/scr_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scr_dump.3x.gz",
                },
                "/usr/share/man/man3/scr_init_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/scr_restore.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scr_dump.3x.gz",
                },
                "/usr/share/man/man3/scr_restore_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/scr_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scr_dump.3x.gz",
                },
                "/usr/share/man/man3/scr_set_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/scrl.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scroll.3x.gz",
                },
                "/usr/share/man/man3/scroll.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scroll.3x.gz",
                },
                "/usr/share/man/man3/scrollok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/set_current_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_page.3x.gz",
                },
                "/usr/share/man/man3/set_current_item.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_current.3x.gz",
                },
                "/usr/share/man/man3/set_curterm.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/set_curterm_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/set_escdelay.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_threads.3x.gz",
                },
                "/usr/share/man/man3/set_escdelay_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/set_field_back.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_attributes.3x.gz",
                },
                "/usr/share/man/man3/set_field_buffer.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_buffer.3x.gz",
                },
                "/usr/share/man/man3/set_field_fore.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_attributes.3x.gz",
                },
                "/usr/share/man/man3/set_field_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/set_field_just.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_just.3x.gz",
                },
                "/usr/share/man/man3/set_field_opts.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_opts.3x.gz",
                },
                "/usr/share/man/man3/set_field_pad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_attributes.3x.gz",
                },
                "/usr/share/man/man3/set_field_status.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_buffer.3x.gz",
                },
                "/usr/share/man/man3/set_field_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/set_field_type.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_validation.3x.gz",
                },
                "/usr/share/man/man3/set_field_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_userptr.3x.gz",
                },
                "/usr/share/man/man3/set_fieldtype_arg.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_fieldtype.3x.gz",
                },
                "/usr/share/man/man3/set_fieldtype_choice.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_fieldtype.3x.gz",
                },
                "/usr/share/man/man3/set_form_fields.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field.3x.gz",
                },
                "/usr/share/man/man3/set_form_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/set_form_opts.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_opts.3x.gz",
                },
                "/usr/share/man/man3/set_form_page.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_page.3x.gz",
                },
                "/usr/share/man/man3/set_form_sub.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_win.3x.gz",
                },
                "/usr/share/man/man3/set_form_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_hook.3x.gz",
                },
                "/usr/share/man/man3/set_form_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_userptr.3x.gz",
                },
                "/usr/share/man/man3/set_form_win.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_win.3x.gz",
                },
                "/usr/share/man/man3/set_item_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/set_item_opts.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_opts.3x.gz",
                },
                "/usr/share/man/man3/set_item_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/set_item_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_userptr.3x.gz",
                },
                "/usr/share/man/man3/set_item_value.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_value.3x.gz",
                },
                "/usr/share/man/man3/set_max_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_field_buffer.3x.gz",
                },
                "/usr/share/man/man3/set_menu_back.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/set_menu_fore.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/set_menu_format.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_format.3x.gz",
                },
                "/usr/share/man/man3/set_menu_grey.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/set_menu_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/set_menu_items.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_items.3x.gz",
                },
                "/usr/share/man/man3/set_menu_mark.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_mark.3x.gz",
                },
                "/usr/share/man/man3/set_menu_opts.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_opts.3x.gz",
                },
                "/usr/share/man/man3/set_menu_pad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_attributes.3x.gz",
                },
                "/usr/share/man/man3/set_menu_pattern.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_pattern.3x.gz",
                },
                "/usr/share/man/man3/set_menu_spacing.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_spacing.3x.gz",
                },
                "/usr/share/man/man3/set_menu_sub.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_win.3x.gz",
                },
                "/usr/share/man/man3/set_menu_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_hook.3x.gz",
                },
                "/usr/share/man/man3/set_menu_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_userptr.3x.gz",
                },
                "/usr/share/man/man3/set_menu_win.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_win.3x.gz",
                },
                "/usr/share/man/man3/set_new_page.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_new_page.3x.gz",
                },
                "/usr/share/man/man3/set_panel_userptr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/set_tabsize.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_threads.3x.gz",
                },
                "/usr/share/man/man3/set_tabsize_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/set_term.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_initscr.3x.gz",
                },
                "/usr/share/man/man3/set_top_row.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_current.3x.gz",
                },
                "/usr/share/man/man3/setcchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getcchar.3x.gz",
                },
                "/usr/share/man/man3/setscrreg.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/setsyx.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_kernel.3x.gz",
                },
                "/usr/share/man/man3/setupterm.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/show_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/slk_attr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_attr_off.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_attr_on.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_attr_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_attr_set_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_attr_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_attroff.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_attroff_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_attron.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_attron_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_attrset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_attrset_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_clear.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_clear_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_color.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_color_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_init.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_init_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_label.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_label_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_noutrefresh.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_noutrefresh_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_refresh.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_refresh_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_restore.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_restore_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_set_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_touch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/slk_touch_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/slk_wset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_slk.3x.gz",
                },
                "/usr/share/man/man3/standend.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/standout.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/start_color.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_color.3x.gz",
                },
                "/usr/share/man/man3/start_color_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/stdscr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_variables.3x.gz",
                },
                "/usr/share/man/man3/strcodes.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/strfnames.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/strnames.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/subpad.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_pad.3x.gz",
                },
                "/usr/share/man/man3/subwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/syncok.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/term_attrs.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/term_attrs_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/term_variables.3x.gz": {},
                "/usr/share/man/man3/termattrs.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/termattrs_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/termname.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termattrs.3x.gz",
                },
                "/usr/share/man/man3/termname_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tgetent.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/tgetent_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tgetflag.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/tgetflag_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tgetnum.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/tgetnum_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tgetstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/tgetstr_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tgoto.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_termcap.3x.gz",
                },
                "/usr/share/man/man3/tgoto_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tigetflag.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/tigetflag_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tigetnum.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/tigetnum_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tigetstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/tigetstr_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/timeout.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/tiparm.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/top_panel.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/top_row.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/mitem_current.3x.gz",
                },
                "/usr/share/man/man3/touchline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_touch.3x.gz",
                },
                "/usr/share/man/man3/touchwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_touch.3x.gz",
                },
                "/usr/share/man/man3/tparm.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/tparm_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/tputs.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/tputs_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/trace.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_trace.3x.gz",
                },
                "/usr/share/man/man3/ttytype.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/term_variables.3x.gz",
                },
                "/usr/share/man/man3/typeahead.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/typeahead_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/unctrl.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/unctrl_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/unfocus_current_field.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_page.3x.gz",
                },
                "/usr/share/man/man3/unget_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wch.3x.gz",
                },
                "/usr/share/man/man3/unget_wch_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/ungetch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getch.3x.gz",
                },
                "/usr/share/man/man3/ungetch_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/ungetmouse.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/ungetmouse_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/unpost_form.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/form_post.3x.gz",
                },
                "/usr/share/man/man3/unpost_menu.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/menu_post.3x.gz",
                },
                "/usr/share/man/man3/untouchwin.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_touch.3x.gz",
                },
                "/usr/share/man/man3/update_panels.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/panel.3x.gz",
                },
                "/usr/share/man/man3/update_panels_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/use_default_colors.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/default_colors.3x.gz",
                },
                "/usr/share/man/man3/use_default_colors_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/use_env.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/use_env_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/use_extended_names.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_extend.3x.gz",
                },
                "/usr/share/man/man3/use_legacy_coding.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/legacy_coding.3x.gz",
                },
                "/usr/share/man/man3/use_legacy_coding_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/use_screen.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_threads.3x.gz",
                },
                "/usr/share/man/man3/use_tioctl.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/use_tioctl_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/use_window.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_threads.3x.gz",
                },
                "/usr/share/man/man3/vid_attr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/vid_attr_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/vid_puts.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/vid_puts_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/vidattr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/vidattr_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/vidputs.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_terminfo.3x.gz",
                },
                "/usr/share/man/man3/vidputs_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/vline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/vline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/vw_printw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_printw.3x.gz",
                },
                "/usr/share/man/man3/vw_scanw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scanw.3x.gz",
                },
                "/usr/share/man/man3/vwprintw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_printw.3x.gz",
                },
                "/usr/share/man/man3/vwscanw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scanw.3x.gz",
                },
                "/usr/share/man/man3/wadd_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wch.3x.gz",
                },
                "/usr/share/man/man3/wadd_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/wadd_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wchstr.3x.gz",
                },
                "/usr/share/man/man3/waddch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addch.3x.gz",
                },
                "/usr/share/man/man3/waddchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/waddchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addchstr.3x.gz",
                },
                "/usr/share/man/man3/waddnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/waddnwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/waddstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addstr.3x.gz",
                },
                "/usr/share/man/man3/waddwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addwstr.3x.gz",
                },
                "/usr/share/man/man3/wattr_get.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wattr_off.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wattr_on.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wattr_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wattroff.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wattron.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wattrset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wbkgd.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgd.3x.gz",
                },
                "/usr/share/man/man3/wbkgdset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgd.3x.gz",
                },
                "/usr/share/man/man3/wbkgrnd.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgrnd.3x.gz",
                },
                "/usr/share/man/man3/wbkgrndset.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgrnd.3x.gz",
                },
                "/usr/share/man/man3/wborder.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/wborder_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/wchgat.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wclear.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/wclrtobot.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/wclrtoeol.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/wcolor_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wcursyncup.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/wdelch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_delch.3x.gz",
                },
                "/usr/share/man/man3/wdeleteln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_deleteln.3x.gz",
                },
                "/usr/share/man/man3/wecho_wchar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_add_wch.3x.gz",
                },
                "/usr/share/man/man3/wechochar.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_addch.3x.gz",
                },
                "/usr/share/man/man3/wenclose.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/werase.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_clear.3x.gz",
                },
                "/usr/share/man/man3/wget_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wch.3x.gz",
                },
                "/usr/share/man/man3/wget_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/wgetbkgrnd.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_bkgrnd.3x.gz",
                },
                "/usr/share/man/man3/wgetch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getch.3x.gz",
                },
                "/usr/share/man/man3/wgetdelay.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/wgetn_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_get_wstr.3x.gz",
                },
                "/usr/share/man/man3/wgetnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/wgetparent.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/wgetscrreg.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_opaque.3x.gz",
                },
                "/usr/share/man/man3/wgetstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_getstr.3x.gz",
                },
                "/usr/share/man/man3/whline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/whline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man3/win_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wch.3x.gz",
                },
                "/usr/share/man/man3/win_wchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/win_wchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_in_wchstr.3x.gz",
                },
                "/usr/share/man/man3/winch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inch.3x.gz",
                },
                "/usr/share/man/man3/winchnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/winchstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inchstr.3x.gz",
                },
                "/usr/share/man/man3/winnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/winnwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/wins_nwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/wins_wch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wch.3x.gz",
                },
                "/usr/share/man/man3/wins_wstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_ins_wstr.3x.gz",
                },
                "/usr/share/man/man3/winsch.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insch.3x.gz",
                },
                "/usr/share/man/man3/winsdelln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_deleteln.3x.gz",
                },
                "/usr/share/man/man3/winsertln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_deleteln.3x.gz",
                },
                "/usr/share/man/man3/winsnstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/winsstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_insstr.3x.gz",
                },
                "/usr/share/man/man3/winstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_instr.3x.gz",
                },
                "/usr/share/man/man3/winwstr.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inwstr.3x.gz",
                },
                "/usr/share/man/man3/wmouse_trafo.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_mouse.3x.gz",
                },
                "/usr/share/man/man3/wmove.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_move.3x.gz",
                },
                "/usr/share/man/man3/wnoutrefresh.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_refresh.3x.gz",
                },
                "/usr/share/man/man3/wprintw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_printw.3x.gz",
                },
                "/usr/share/man/man3/wredrawln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_refresh.3x.gz",
                },
                "/usr/share/man/man3/wrefresh.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_refresh.3x.gz",
                },
                "/usr/share/man/man3/wresize.3x.gz": {},
                "/usr/share/man/man3/wscanw.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scanw.3x.gz",
                },
                "/usr/share/man/man3/wscrl.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_scroll.3x.gz",
                },
                "/usr/share/man/man3/wsetscrreg.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_outopts.3x.gz",
                },
                "/usr/share/man/man3/wstandend.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wstandout.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_attr.3x.gz",
                },
                "/usr/share/man/man3/wsyncdown.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/wsyncup.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_window.3x.gz",
                },
                "/usr/share/man/man3/wtimeout.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_inopts.3x.gz",
                },
                "/usr/share/man/man3/wtouchln.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_touch.3x.gz",
                },
                "/usr/share/man/man3/wunctrl.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_util.3x.gz",
                },
                "/usr/share/man/man3/wunctrl_sp.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_sp_funcs.3x.gz",
                },
                "/usr/share/man/man3/wvline.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border.3x.gz",
                },
                "/usr/share/man/man3/wvline_set.3x.gz": {
                    "type": "Symlink",
                    "target": "/usr/share/man/man3/curs_border_set.3x.gz",
                },
                "/usr/share/man/man5/scr_dump.5.gz": {},
                "/usr/share/man/man5/term.5.gz": {},
                "/usr/share/man/man5/terminfo.5.gz": {},
                "/usr/share/man/man5/user_caps.5.gz": {},
                "/usr/share/man/man7/term.7.gz": {},
                "/usr/share/tabset/std": {},
                "/usr/share/tabset/stdcrt": {},
                "/usr/share/tabset/vt100": {},
                "/usr/share/tabset/vt300": {},
            },
        },
    ],
    "header_file_dirs": [
        "/usr/include",
    ],
    "header_file_dir_regexes": [
        "/usr/include",
    ],
    "ld_library_path": [
        "/lib64",
        "/lib32",
        "/usr/lib64",
    ],
}
