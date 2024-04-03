// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
extern crate example_rust_proto;

fn main() {
    let mut proto = example_rust_proto::Example::new();
    proto.set_foo("blah".to_string());
    println!("Proto: {:?}", proto);
}
