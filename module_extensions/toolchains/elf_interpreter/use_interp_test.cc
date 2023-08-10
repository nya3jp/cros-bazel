// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include <sys/auxv.h>

#include <array>
#include <cstring>
#include <iostream>

int main(int argc, char **argv) {
  if (argc != 2) {
    std::cerr << "Incorrect argc: got " << argc << ", want 2\n";
    return 1;
  }
  if (strcmp(argv[1], "foo") != 0) {
    std::cerr << "Incorrect argv[1]: got " << argv[1] << ", want foo\n";
    return 1;
  }

  std::cout << "Hello, World!\n";

  // We screw with the env and aux array in the interpreter, so verify that it's
  // working as intended.
  const char *user = std::getenv("USER");
  if (user == nullptr) {
    std::cerr << "USER is unset\n";
    return 1;
  }

  if (auto page_size = getauxval(AT_PAGESZ); page_size != 4096) {
    std::cerr << "Unexpected page size: got " << page_size << ", want 4096\n";
    return 1;
  }

  return 0;
}
