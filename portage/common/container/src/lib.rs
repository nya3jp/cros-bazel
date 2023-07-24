// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod clean_layer;
mod container;
mod control;
mod install_group;
mod mounts;
mod namespace;

pub use clean_layer::*;
pub use container::*;
pub use install_group::*;
pub use namespace::*;

// Run unit tests in a mount namespace.
#[cfg(test)]
#[used]
#[link_section = ".init_array"]
static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;
