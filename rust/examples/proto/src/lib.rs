// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
extern crate example_rust_proto;

#[cfg(test)]
mod tests {
    use super::*;

    use protobuf::{parse_from_bytes, Message};

    fn simple_proto() -> example_rust_proto::Example {
        let mut proto = example_rust_proto::Example::new();
        proto.set_foo("blah".to_string());
        proto
    }

    #[test]
    fn sets_fields() {
        assert_eq!(simple_proto().get_foo(), "blah");
    }

    #[test]
    fn serde() {
        let orig = simple_proto();
        let serialized = orig.write_to_bytes().unwrap();

        let deserialized = parse_from_bytes::<example_rust_proto::Example>(&serialized).unwrap();
        assert_eq!(deserialized.get_foo(), "blah");
    }
}
