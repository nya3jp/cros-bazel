// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Simple versioned shared object used for testing.
int hello_world_v1(char *name) { return 1; } asm(".symver hello_world_v1, hello_world@v1, remove");
int hello_world(char *name) { return 2; }
