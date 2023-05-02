// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use log::log_enabled;
use log::Level::Info;

fn main() {
    env_logger::init();
    if log_enabled!(Info) {
        println!("Info enabled");
    } else {
        println!("Info disabled");
    }

    println!(
        "RUNFILES DIR IS {:?}",
        runfiles::find_runfiles_dir().unwrap()
    )
}
