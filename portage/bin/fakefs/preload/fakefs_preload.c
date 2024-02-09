// Copyright 2022 The ChromiumOS Authors
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
// TODO: Compile this code with hermetic toolchains and get rid of this hack.
__asm__(".symver __errno_location,__errno_location@GLIBC_2.2.5");
__asm__(".symver abort,abort@GLIBC_2.2.5");
__asm__(".symver close,close@GLIBC_2.2.5");
__asm__(".symver dlsym,dlsym@GLIBC_2.2.5");
__asm__(".symver fprintf,fprintf@GLIBC_2.2.5");
__asm__(".symver fwrite,fwrite@GLIBC_2.2.5");
__asm__(".symver getenv,getenv@GLIBC_2.2.5");
__asm__(".symver gettid,gettid@GLIBC_2.30");
__asm__(".symver getxattr,getxattr@GLIBC_2.3");
__asm__(".symver lgetxattr,lgetxattr@GLIBC_2.3");
__asm__(".symver lremovexattr,removexattr@GLIBC_2.3");
__asm__(".symver openat,openat@GLIBC_2.4");
__asm__(".symver pthread_once,pthread_once@GLIBC_2.2.5");
__asm__(".symver removexattr,removexattr@GLIBC_2.3");
__asm__(".symver sprintf,sprintf@GLIBC_2.2.5");
__asm__(".symver stderr,stderr@GLIBC_2.2.5");
__asm__(".symver syscall,syscall@GLIBC_2.2.5");

static const char OVERRIDE_XATTR_NAME[] = "user.fakefs.override";
static const int FAKEFS_BACKDOOR_KEY = 0x20221107;

static pthread_once_t g_init_flag = PTHREAD_ONCE_INIT;

static bool g_verbose;
static bool g_abort_on_slow;
static int (*g_libc_fstatat)(int dirfd, const char *pathname,
                             struct stat *statbuf, int flags);
static int (*g_libc_statx)(int dirfd, const char *pathname, int flags,
                           unsigned int mask, struct statx *statxbuf);
static int (*g_libc_fchownat)(int dirfd, const char *pathname, uid_t owner,
                              gid_t group, int flags);
static int (*g_libc_fchmodat)(int dirfd, const char *pathname, mode_t mode,
                              int flags);

static void do_init(void) {
  g_verbose = getenv("FAKEFS_VERBOSE") != NULL;
  g_abort_on_slow = getenv("FAKEFS_ABORT_ON_SLOW") != NULL;
  g_libc_fstatat = dlsym(RTLD_NEXT, "fstatat");
  g_libc_statx = dlsym(RTLD_NEXT, "statx");
  g_libc_fchownat = dlsym(RTLD_NEXT, "fchownat");
  g_libc_fchmodat = dlsym(RTLD_NEXT, "fchmodat");
}

static void ensure_init(void) { pthread_once(&g_init_flag, do_init); }

static bool path_has_no_override(const char *pathname, bool follow_symlink) {
  int ret = (follow_symlink ? getxattr : lgetxattr)(
      pathname, OVERRIDE_XATTR_NAME, NULL, 0);
  return ret < 0 && (errno == ENODATA || errno == ENOTSUP || errno == ENOENT ||
                     errno == ENOTDIR);
}

static bool fd_has_no_override(int fd) {
  // fgetxattr may not work with O_PATH file descriptors, so use /proc/self/fd
  // instead.
  char fdpath[64];
  sprintf(fdpath, "/proc/self/fd/%d", fd);
  return path_has_no_override(fdpath, true);
}

// Returns true if the specified file has no ownership override.
// Even if this function returns false, it does not necessarily mean that the
// file has ownership override, e.g. it might be because the function failed to
// determine it due to errors.
// This function preserves `errno`.
static bool has_no_override(int dirfd, const char *pathname, int flags) {
  int saved_errno = errno;

  bool no_override;
  if ((flags & AT_EMPTY_PATH) != 0 && pathname[0] == '\0') {
    no_override = fd_has_no_override(dirfd);
  } else {
    if (dirfd == AT_FDCWD || pathname[0] == '/') {
      no_override =
          path_has_no_override(pathname, (flags & AT_SYMLINK_NOFOLLOW) == 0);
    } else {
      int tmpfd =
          openat(dirfd, pathname,
                 O_RDONLY | O_CLOEXEC | O_PATH |
                     ((flags & AT_SYMLINK_NOFOLLOW) != 0 ? O_NOFOLLOW : 0));
      if (tmpfd >= 0) {
        no_override = fd_has_no_override(tmpfd);
        close(tmpfd);
      } else {
        no_override = false;
      }
    }
  }

  errno = saved_errno;
  return no_override;
}

static bool path_clear_override(const char *pathname, bool follow_symlink) {
  int ret = (follow_symlink ? removexattr : lremovexattr)(pathname,
                                                          OVERRIDE_XATTR_NAME);
  return ret == 0 || errno == ENODATA || errno == ENOTSUP || errno == EPERM;
}

static bool fd_clear_override(int fd) {
  // fremovexattr may not support O_PATH file descriptors, so use /proc/self/fd
  // instead.
  char fdpath[40];
  sprintf(fdpath, "/proc/self/fd/%d", fd);
  return path_clear_override(fdpath, true);
}

// Clears ownership override of the specified file. It returns true on success.
// This function preserves `errno`.
static bool clear_override(int dirfd, const char *pathname, int flags) {
  int saved_errno = errno;

  bool no_override;
  if ((flags & AT_EMPTY_PATH) != 0 && pathname[0] == '\0') {
    no_override = fd_clear_override(dirfd);
  } else {
    if (dirfd == AT_FDCWD || pathname[0] == '/') {
      no_override =
          path_clear_override(pathname, (flags & AT_SYMLINK_NOFOLLOW) == 0);
    } else {
      // We use O_RDONLY instead of O_WRONLY because the latter updates mtime.
      // Fortunately O_RDONLY is enough to manipulate xattrs.
      int tmpfd =
          openat(dirfd, pathname,
                 O_RDONLY | O_CLOEXEC | O_PATH |
                     ((flags & AT_SYMLINK_NOFOLLOW) != 0 ? O_NOFOLLOW : 0));
      if (tmpfd >= 0) {
        no_override = fd_clear_override(tmpfd);
        close(tmpfd);
      } else {
        no_override = false;
      }
    }
  }

  errno = saved_errno;
  return no_override;
}

static int backdoor_fstatat(int dirfd, const char *pathname, void *statbuf,
                            int flags) {
  int ret = syscall(SYS_newfstatat, dirfd, pathname, statbuf, flags, 0,
                    FAKEFS_BACKDOOR_KEY);
  // Clobber %r9 so that FAKEFS_PASS_KEY is not preserved.
  asm volatile("mov $0, %%r9" ::: "r9");
  return ret;
}

static int backdoor_statx(int dirfd, const char *pathname, int flags,
                          unsigned int mask, struct statx *statxbuf) {
  int ret = syscall(SYS_statx, dirfd, pathname, flags, mask, statxbuf,
                    FAKEFS_BACKDOOR_KEY);
  // Clobber %r9 so that FAKEFS_PASS_KEY is not preserved.
  asm volatile("mov $0, %%r9" ::: "r9");
  return ret;
}

static int backdoor_fchownat(int dirfd, const char *pathname, uid_t owner,
                             gid_t group, int flags) {
  int ret = syscall(SYS_fchownat, dirfd, pathname, owner, group, flags,
                    FAKEFS_BACKDOOR_KEY);
  // Clobber %r9 so that FAKEFS_PASS_KEY is not preserved.
  asm volatile("mov $0, %%r9" ::: "r9");
  return ret;
}

// Returns true if the specified file's original ownership (i.e. ignoring
// fakefs ownership override) matches the given UID/GID.
// This function returns false if the function failed to read the original
// ownership due to errors.
// This function preserves `errno`.
static bool matches_original_ownership(int dirfd, const char *pathname,
                                       int flags, uid_t owner, gid_t group) {
  int saved_errno = errno;
  struct stat statbuf;
  bool matched = backdoor_fstatat(dirfd, pathname, &statbuf, flags) == 0 &&
                 statbuf.st_uid == owner && statbuf.st_gid == group;
  errno = saved_errno;
  return matched;
}

static int wrap_fstatat(int dirfd, const char *pathname, void *statbuf,
                        int flags) {
  if (pathname == NULL || statbuf == NULL) {
    errno = EFAULT;
    return -1;
  }

  if (has_no_override(dirfd, pathname, flags)) {
    if (g_verbose) {
      fprintf(stderr, "[fakefs %d] fast: fstatat(%d, \"%s\", 0x%x)\n", gettid(),
              dirfd, pathname, flags);
    }
    return backdoor_fstatat(dirfd, pathname, statbuf, flags);
  }

  if (g_abort_on_slow) {
    fprintf(stderr, "[fakefs %d] ABORT-ON-SLOW: fstatat(%d, \"%s\", 0x%x)\n",
            gettid(), dirfd, pathname, flags);
    abort();
  }
  return g_libc_fstatat(dirfd, pathname, statbuf, flags);
}

static int wrap_statx(int dirfd, const char *pathname, int flags,
                      unsigned int mask, void *statxbuf) {
  if (pathname == NULL || statxbuf == NULL) {
    errno = EFAULT;
    return -1;
  }

  if (has_no_override(dirfd, pathname, flags)) {
    if (g_verbose) {
      fprintf(stderr, "[fakefs %d] fast: statx(%d, \"%s\", 0x%x, 0x%x)\n",
              gettid(), dirfd, pathname, flags, mask);
    }
    return backdoor_statx(dirfd, pathname, flags, mask, statxbuf);
  }

  if (g_abort_on_slow) {
    fprintf(stderr,
            "[fakefs %d] ABORT-ON-SLOW: statx(%d, \"%s\", 0x%x, 0x%x)\n",
            gettid(), dirfd, pathname, flags, mask);
    abort();
  }
  return g_libc_statx(dirfd, pathname, flags, mask, statxbuf);
}

static int wrap_fchownat(int dirfd, const char *pathname, uid_t owner,
                         gid_t group, int flags) {
  if (pathname == NULL) {
    errno = EFAULT;
    return -1;
  }

  if (matches_original_ownership(dirfd, pathname, flags, owner, group)) {
    if (clear_override(dirfd, pathname, flags)) {
      if (g_verbose) {
        fprintf(stderr,
                "[fakefs %d] fast: fchownat(%d, \"%s\", %d, %d, 0x%x)\n",
                gettid(), dirfd, pathname, owner, group, flags);
      }
      // Still call fchownat to update ctime.
      return backdoor_fchownat(dirfd, pathname, owner, group, flags);
    }
  }

  if (g_abort_on_slow) {
    fprintf(stderr,
            "[fakefs %d] ABORT-ON-SLOW: fchownat(%d, \"%s\", %d, %d, 0x%x)\n",
            gettid(), dirfd, pathname, owner, group, flags);
    abort();
  }
  return g_libc_fchownat(dirfd, pathname, owner, group, flags);
}

static int wrap_fchmodat(int dirfd, const char *pathname, mode_t mode,
                         int flags) {
  // Typical filesystems don't support changing permissions of symlinks, so
  // Linux kernel's fchmodat(2) interface was designed not to take the flags
  // argument, and glibc's fchmodat(3) initially returned ENOTSUP when
  // AT_SYMLINK_NOFOLLOW was passed. However, permissions of some special
  // symlinks, such as those on /proc, can be updated by calling chmod on
  // /proc/self/fd/N via O_PATH file descriptors, so glibc added support of
  // AT_SYMLINK_NOFOLLOW in 2.32. Unfortunately the glibc implementation calls
  // into fstatat(2) that is ptrace'd by fakefs and thus slow, while meaningful
  // use cases of fchmodat(3) with AT_SYMLINK_NOFOLLOW are extremely limited.
  // To avoid this unnecessary slowness, we forcibly simulate glibc's old
  // behavior to always return ENOTSUP. Even if this happens, callers (e.g.
  // GNU tar) should fall back to calling fchmodat(3) without
  // AT_SYMLINK_NOFOLLOW for compatibility with older glibc.
  if ((flags & AT_SYMLINK_NOFOLLOW) != 0) {
    errno = ENOTSUP;
    return -1;
  }
  return g_libc_fchmodat(dirfd, pathname, mode, flags);
}

int __fakefs_stat(const char *pathname, struct stat *statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, statbuf, 0);
}

int __fakefs_stat64(const char *pathname, struct stat64 *statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, (struct stat *)statbuf, 0);
}

int __fakefs_lstat(const char *pathname, struct stat *statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, statbuf, AT_SYMLINK_NOFOLLOW);
}

int __fakefs_lstat64(const char *pathname, struct stat64 *statbuf) {
  ensure_init();
  return wrap_fstatat(AT_FDCWD, pathname, (struct stat *)statbuf,
                      AT_SYMLINK_NOFOLLOW);
}

int __fakefs_fstat(int fd, struct stat *statbuf) {
  ensure_init();
  return wrap_fstatat(fd, "", statbuf, AT_EMPTY_PATH);
}

int __fakefs_fstat64(int fd, struct stat64 *statbuf) {
  ensure_init();
  return wrap_fstatat(fd, "", (struct stat *)statbuf, AT_EMPTY_PATH);
}

int __fakefs_fstatat(int dirfd, const char *pathname, struct stat *statbuf,
                     int flags) {
  ensure_init();
  return wrap_fstatat(dirfd, pathname, statbuf, flags);
}

int __fakefs_fstatat64(int dirfd, const char *pathname, struct stat64 *statbuf,
                       int flags) {
  ensure_init();
  return wrap_fstatat(dirfd, pathname, (struct stat *)statbuf, flags);
}

int __fakefs_statx(int dirfd, const char *pathname, int flags,
                   unsigned int mask, struct statx *statxbuf) {
  ensure_init();
  return wrap_statx(dirfd, pathname, flags, mask, statxbuf);
}

int __fakefs_chown(const char *pathname, uid_t owner, gid_t group) {
  ensure_init();
  return wrap_fchownat(AT_FDCWD, pathname, owner, group, 0);
}

int __fakefs_fchown(int fd, uid_t owner, gid_t group) {
  ensure_init();
  return wrap_fchownat(fd, "", owner, group, AT_EMPTY_PATH);
}

int __fakefs_lchown(const char *pathname, uid_t owner, gid_t group) {
  ensure_init();
  return wrap_fchownat(AT_FDCWD, pathname, owner, group, AT_SYMLINK_NOFOLLOW);
}

int __fakefs_fchownat(int dirfd, const char *pathname, uid_t owner, gid_t group,
                      int flags) {
  ensure_init();
  return wrap_fchownat(dirfd, pathname, owner, group, flags);
}

int __fakefs_fchmodat(int dirfd, const char *pathname, mode_t mode, int flags) {
  ensure_init();
  return wrap_fchmodat(dirfd, pathname, mode, flags);
}

// Define libc intercepting symbols as aliases.
// Implementing them directly can lead to incorrect compiler optimizations
// because prototype declarations of these functions in the standard library
// headers may be annotated with extra information (e.g. nonnull) that can cause
// our functions to be optimized in unexpected ways.
// See b/285262832 for the background.
int stat(const char *pathname, struct stat *statbuf)
    __attribute__((alias("__fakefs_stat")));
int stat64(const char *pathname, struct stat64 *statbuf)
    __attribute__((alias("__fakefs_stat64")));
int lstat(const char *pathname, struct stat *statbuf)
    __attribute__((alias("__fakefs_lstat")));
int lstat64(const char *pathname, struct stat64 *statbuf)
    __attribute__((alias("__fakefs_lstat64")));
int fstat(int fd, struct stat *statbuf)
    __attribute__((alias("__fakefs_fstat")));
int fstat64(int fd, struct stat64 *statbuf)
    __attribute__((alias("__fakefs_fstat64")));
int fstatat(int dirfd, const char *pathname, struct stat *statbuf, int flags)
    __attribute__((alias("__fakefs_fstatat")));
int fstatat64(int dirfd, const char *pathname, struct stat64 *statbuf,
              int flags) __attribute__((alias("__fakefs_fstatat64")));
int statx(int dirfd, const char *pathname, int flags, unsigned int mask,
          struct statx *statxbuf) __attribute__((alias("__fakefs_statx")));
int chown(const char *pathname, uid_t owner, gid_t group)
    __attribute__((alias("__fakefs_chown")));
int fchown(int fd, uid_t owner, gid_t group)
    __attribute__((alias("__fakefs_fchown")));
int lchown(const char *pathname, uid_t owner, gid_t group)
    __attribute__((alias("__fakefs_lchown")));
int fchownat(int dirfd, const char *pathname, uid_t owner, gid_t group,
             int flags) __attribute__((alias("__fakefs_fchownat")));
int fchmodat(int dirfd, const char *pathname, mode_t mode, int flags)
    __attribute__((alias("__fakefs_fchmodat")));
