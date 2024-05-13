// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[cfg(test)]
mod tests {
    use super::*;

    use prost::Message;

    use example_proto::Example;

    fn simple_proto() -> Example {
        Example {
            foo: "blah".to_string(),
        }
    }

    #[test]
    fn sets_fields() {
        assert_eq!(simple_proto().foo, "blah");
    }

    #[test]
    fn serde() {
        let orig = simple_proto();
        let serialized = orig.encode_to_vec();

        let deserialized = Example::decode(serialized.as_slice()).unwrap();
        assert_eq!(deserialized.foo, "blah");
    }
}
