// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/*
 Clang_selector decides whether to invoke clang or clang++.
 This resolves https://github.com/bazelbuild/bazel/issues/11122.
 Other users have suggested using clang++ to build C code, but that doesn't
 appear to work.
*/

#include <unistd.h>

#include <algorithm>
#include <iostream>
#include <string>
#include <vector>

constexpr const char *kEnvVar = "FORCE_C_COMPILER";
constexpr const char *kCliArg = "--force-c-compiler";

constexpr const char *kCCompiler = "clang";
constexpr const char *kCppCompiler = "clang++";

int main(int argc, char **argv) {
  std::string path = argv[0];
  // Get the directory name.
  path.erase(path.rfind("/") + 1);

  const char *compiler = kCppCompiler;
  std::vector<const char *> args(argv, argv + argc);

  if (const auto iter = std::find_if(
          args.begin(), args.end(),
          [](const char *arg) { return strcmp(arg, kCliArg) == 0; });
      iter != args.end()) {
    args.erase(iter);
    compiler = kCCompiler;
  }

  if (const char *env = std::getenv(kEnvVar);
      env != nullptr && strcmp(env, "") != 0 && strcmp(env, "0") != 0) {
    compiler = kCCompiler;
  }

  path.append(compiler);
  args[0] = path.c_str();

  // Ensure that args.data() is null-terminated.
  args.push_back(nullptr);
  execv(args[0], (char *const *)args.data());

  std::cerr << "Got error " << strerror(errno) << " while executing " << args[0]
            << std::endl;
  return errno;
}
