// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString, strum_macros::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum LoginMode {
    #[strum(serialize = "")]
    Never,
    Before,
    After,
    AfterFail,
}
