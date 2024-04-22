// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Defines the ebuild metadata types.
//!
//! See: ///bazel/portage/bin/metadata/metadata.proto

use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EbuildMetadata {
    pub label: String,
    pub sha256: String,
    // Yes String, see https://github.com/protocolbuffers/protobuf/issues/2954.
    pub size: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize() {
        // NamedSetOfFiles.
        assert_eq!(
            serde_json::from_value::<EbuildMetadata>(json!({
              "label": "@@_main~portage~portage//internal/packages/stage1/target/host/chromiumos/sys-kernel/linux-headers:4.14-r92",
              "sha256": "c38045633a8cec7411aa0618e63896b52b0234ddd7136a34a69ef2af9134b74d",
              "size": "1322061"
            }))
            .expect("failed to deserialize EbuildMetadata"),
            EbuildMetadata {
                label: "@@_main~portage~portage//internal/packages/stage1/target/host/chromiumos/sys-kernel/linux-headers:4.14-r92".into(),
                sha256: "c38045633a8cec7411aa0618e63896b52b0234ddd7136a34a69ef2af9134b74d".into(),
                size: "1322061".into(),
            }
        );
    }
}
