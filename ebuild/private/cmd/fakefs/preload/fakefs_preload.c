// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/xattr.h>
#include <unistd.h>

// Maximize glibc compatibility.
// TODO: Compile this code with CrOS SDK.
__asm__(".symver __errno_location,__errno_location@GLIBC_2.2.5");
__asm__(".symver dlsym,dlsym@GLIBC_2.2.5");
__asm__(".symver fgetxattr,fgetxattr@GLIBC_2.3");
__asm__(".symver fwrite,fwrite@GLIBC_2.2.5");
__asm__(".symver getenv,getenv@GLIBC_2.2.5");
__asm__(".symver getxattr,getxattr@GLIBC_2.3");
__asm__(".symver lgetxattr,lgetxattr@GLIBC_2.3");
__asm__(".symver pthread_once,pthread_once@GLIBC_2.2.5");
__asm__(".symver stderr,stderr@GLIBC_2.2.5");
__asm__(".symver syscall,syscall@GLIBC_2.2.5");

static const char OVERRIDE_XATTR_NAME[] = "user.fakefs.override";
static const int FAKEFS_BACKDOOR_KEY = 0x20221107;

static pthread_once_t g_init_flag = PTHREAD_ONCE_INIT;

static bool g_verbose;
static int (*g_libc_fstatat)(int dirfd, const char* pathname,
                             struct stat* statbuf, int flags);
static int (*g_libc_statx)(int dirfd, const char* pathname, int flags,
                           unsigned int mask, struct statx* statxbuf);

static void do_init(void) {
  g_verbose = getenv("FAKEFS_VERBOSE") != NULL;
  g_libc_fstatat = dlsym(RTLD_NEXT, "fstatat");
  g_libc_statx = dlsym(RTLD_NEXT, "statx");
}

static void ensure_init(void) { pthread_once(&g_init_flag, do_init); }

static bool errno_has_no_override() {
  return errno == ENODATA || errno == ENOTSUP || errno == ENOENT ||
         errno == ENOTDIR;
}

static bool path_has_no_override(const char* pathname, bool follow_symlink) {
  int saved_errno = errno;
  errno = 0;
  (follow_symlink ? getxattr : lgetxattr)(pathname, OVERRIDE_XATTR_NAME, NULL,
                                          0);
  bool result = errno_has_no_override();
  errno = saved_errno;
  return result;
}

static bool fd_has_no_override(int fd) {
  int saved_errno = errno;
  errno = 0;
  fgetxattr(fd, OVERRIDE_XATTR_NAME, NULL, 0);
  bool result = errno_has_no_override();
  errno = saved_errno;
  return result;
}

static int backdoor_fstatat(int dirfd, const char* pathname,
                            struct stat* statbuf, int flags) {
  if (g_verbose) {
    fprintf(stderr, "[fakefs %d] fast: fstatat(%d, \"%s\", 0x%x)\n", gettid(),
            dirfd, pathname, flags);
  }

  int ret = syscall(SYS_newfstatat, dirfd, pathname, statbuf, flags, 0,
                    FAKEFS_BACKDOOR_KEY);
  // Clobber %r9 so that FAKEFS_PASS_KEY is not preserved.
  asm volatile("mov $0, %%r9" ::: "r9");
  return ret;
}

static int backdoor_statx(int dirfd, const char* pathname, int flags,
                          unsigned int mask, struct statx* statxbuf) {
  if (g_verbose) {
    fprintf(stderr, "[fakefs %d] fast: statx(%d, \"%s\", 0x%x, 0x%x)\n",
            gettid(), dirfd, pathname, flags, mask);
  }

  int ret = syscall(SYS_statx, dirfd, pathname, flags, mask, statxbuf,
                    FAKEFS_BACKDOOR_KEY);
  // Clobber %r9 so that FAKEFS_PASS_KEY is not preserved.
  asm volatile("mov $0, %%r9" ::: "r9");
  return ret;
}

static int wrap_fstatat(int dirfd, const char* pathname, struct stat* statbuf,
                        int flags) {
  if (pathname == NULL || statbuf == NULL) {
    errno = EFAULT;
    return -1;
  }

  if (dirfd == AT_FDCWD || pathname[0] == '/') {
    if (path_has_no_override(pathname, (flags & AT_SYMLINK_NOFOLLOW) == 0)) {
      return backdoor_fstatat(dirfd, pathname, statbuf, flags);
    }
  }

  if (pathname[0] == '\0' && (flags & AT_EMPTY_PATH) != 0) {
    if (fd_has_no_override(dirfd)) {
      return backdoor_fstatat(dirfd, pathname, statbuf, flags);
    }
  }

  return g_libc_fstatat(dirfd, pathname, statbuf, flags);
}

static int wrap_statx(int dirfd, const char* pathname, int flags,
                      unsigned int mask, struct statx* statxbuf) {
  if (pathname == NULL || statxbuf == NULL) {
    errno = EFAULT;
    return -1;
  }

  if (dirfd == AT_FDCWD || pathname[0] == '/') {
    if (path_has_no_override(pathname, (flags & AT_SYMLINK_NOFOLLOW) == 0)) {
      return backdoor_statx(dirfd, pathname, flags, mask, statxbuf);
    }
  }

  if (pathname[0] == '\0' && (flags & AT_EMPTY_PATH) != 0) {
    if (fd_has_no_override(dirfd)) {
      return backdoor_statx(dirfd, pathname, flags, mask, statxbuf);
    }
  }

  return g_libc_statx(dirfd, pathname, flags, mask, statxbuf);
}

int stat(const char* pathname, struct stat* statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, statbuf, 0);
}

int stat64(const char* pathname, struct stat64* statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, (struct stat*)statbuf, 0);
}

int lstat(const char* pathname, struct stat* statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, statbuf, AT_SYMLINK_NOFOLLOW);
}

int lstat64(const char* pathname, struct stat64* statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, (struct stat*)statbuf,
                      AT_SYMLINK_NOFOLLOW);
}

int fstat(int fd, struct stat* statbuf) {
  ensure_init();
  return wrap_fstatat(fd, "", statbuf, AT_EMPTY_PATH);
}

int fstat64(int fd, struct stat64* statbuf) {
  ensure_init();
  return wrap_fstatat(fd, "", (struct stat*)statbuf, AT_EMPTY_PATH);
}

int fstatat(int dirfd, const char* pathname, struct stat* statbuf, int flags) {
  ensure_init();
  return wrap_fstatat(dirfd, pathname, statbuf, flags);
}

int fstatat64(int dirfd, const char* pathname, struct stat64* statbuf,
              int flags) {
  ensure_init();
  return wrap_fstatat(dirfd, pathname, (struct stat*)statbuf, flags);
}

int statx(int dirfd, const char* pathname, int flags, unsigned int mask,
          struct statx* statxbuf) {
  ensure_init();
  return wrap_statx(dirfd, pathname, flags, mask, statxbuf);
}
