// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#define _GNU_SOURCE
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

// Calls fstatat with AT_EMPTY_PATH.
int fstatat_empty_path(const char *path) {
  int dirfd = open(path, O_RDONLY | O_PATH);
  if (dirfd < 0) {
    perror("open");
    return EXIT_FAILURE;
  }

  struct stat st;
  if (fstatat(dirfd, "", &st, AT_EMPTY_PATH) < 0) {
    perror("fstatat");
    return EXIT_FAILURE;
  }

  close(dirfd);
  printf("%d:%d\n", st.st_uid, st.st_gid);
  return EXIT_SUCCESS;
}

// Calls stat via /proc/self/fd.
int stat_proc_self_fd(const char *path) {
  int fd = open(path, O_RDONLY | O_PATH);
  if (fd < 0) {
    perror("open");
    return EXIT_FAILURE;
  }

  char fdpath[64];
  sprintf(fdpath, "/proc/self/fd/%d", fd);

  struct stat st;
  if (stat(fdpath, &st) < 0) {
    perror("stat");
    return EXIT_FAILURE;
  }

  close(fd);
  printf("%d:%d\n", st.st_uid, st.st_gid);
  return EXIT_SUCCESS;
}

// Calls fchown with the current UID/GID.
int fchown_self(const char *path) {
  int fd = open(path, O_RDONLY | O_PATH);
  if (fd < 0) {
    perror("open");
    return EXIT_FAILURE;
  }

  if (fchown(fd, getuid(), getgid()) < 0) {
    perror("fchown");
    return EXIT_FAILURE;
  }

  close(fd);
  return EXIT_SUCCESS;
}

int main(int argc, char **argv) {
  if (argc < 2) {
    fprintf(stderr, "testhelper: needs arguments\n");
    return EXIT_FAILURE;
  }
  if (strcmp(argv[1], "fstatat-empty-path") == 0) {
    if (argc != 3) {
      fprintf(stderr, "testhelper: fstatat-empty-path: needs exactly 1 path\n");
      return EXIT_FAILURE;
    }
    return fstatat_empty_path(argv[2]);
  }
  if (strcmp(argv[1], "stat-proc-self-fd") == 0) {
    if (argc != 3) {
      fprintf(stderr, "testhelper: stat-proc-self-fd: needs exactly 1 path\n");
      return EXIT_FAILURE;
    }
    return stat_proc_self_fd(argv[2]);
  }
  if (strcmp(argv[1], "fchown-self") == 0) {
    if (argc != 3) {
      fprintf(stderr, "testhelper: fchown-self: needs exactly 1 path\n");
      return EXIT_FAILURE;
    }
    return fchown_self(argv[2]);
  }
  fprintf(stderr, "testhelper: unknown subcommand %s\n", argv[1]);
  return EXIT_FAILURE;
}
