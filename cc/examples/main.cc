// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include <iostream>

#include "bazel/cc/examples/lib.h"

int main() {
  std::cout << "The answer is " << get_answer() << std::endl;
  return 0;
}
