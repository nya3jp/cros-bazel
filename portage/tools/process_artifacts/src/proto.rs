// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Defines types corresponding to Build Event Protocol protobuf messages.
//!
//! See: https://github.com/bazelbuild/bazel/blob/HEAD/src/main/java/com/google/devtools/build/lib/buildeventstream/proto/build_event_stream.proto

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PhantomValue;

impl<'de> Deserialize<'de> for PhantomValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = serde_json::Value::deserialize(deserializer)?;
        Ok(Self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildEvent {
    pub id: BuildEventId,
    #[serde(flatten)]
    pub payload: BuildEventPayload,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildEventId {
    NamedSet(NamedSetOfFilesId),
    TargetCompleted(TargetCompletedId),
    #[serde(untagged)]
    Other(PhantomValue),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildEventPayload {
    NamedSetOfFiles(NamedSetOfFiles),
    Completed(TargetComplete),
    #[serde(untagged)]
    Other(PhantomValue),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedSetOfFiles {
    #[serde(default)]
    pub files: Vec<File>,
    #[serde(default)]
    pub file_sets: Vec<NamedSetOfFilesId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetComplete {
    pub success: bool,
    #[serde(default)]
    pub output_group: Vec<OutputGroup>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedSetOfFilesId {
    pub id: String,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetCompletedId {
    pub label: String,
    #[serde(default)]
    pub aspect: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub name: String,
    pub path_prefix: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputGroup {
    pub name: String,
    #[serde(default)]
    pub file_sets: Vec<NamedSetOfFilesId>,
    #[serde(default)]
    pub incomplete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_value::<BuildEvent>(json!({
                "id": {
                    "namedSet": {"id":"5"}
                },
                "namedSetOfFiles": {
                    "files": [
                        {
                            "name": "path/to/package.log",
                            "pathPrefix": ["bazel-out","k8-fastbuild","bin"]
                        }
                    ]
                }
            }))
            .expect("failed to deserialize BuildEvent"),
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "5".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: "path/to/package.log".to_string(),
                        path_prefix: vec![
                            "bazel-out".to_string(),
                            "k8-fastbuild".to_string(),
                            "bin".to_string(),
                        ],
                    }],
                    file_sets: vec![],
                })
            }
        );

        assert_eq!(
            serde_json::from_value::<BuildEvent>(json!({
                "id": {
                    "targetCompleted": {
                        "label": "@portage/some/package:target",
                        "aspect": "//bazel/portage/build_defs:some.bzl%some_aspect",
                    }
                },
                "completed": {
                    "success": true,
                    "outputGroup": [
                        {
                            "name": "all_logs",
                            "fileSets": [{"id": "0"}]
                        }
                    ]
                }
            }))
            .expect("failed to deserialize BuildEvent"),
            BuildEvent {
                id: BuildEventId::TargetCompleted(TargetCompletedId {
                    label: "@portage/some/package:target".to_string(),
                    aspect: Some("//bazel/portage/build_defs:some.bzl%some_aspect".to_string()),
                }),
                payload: BuildEventPayload::Completed(TargetComplete {
                    success: true,
                    output_group: vec![OutputGroup {
                        name: "all_logs".to_string(),
                        file_sets: vec![NamedSetOfFilesId {
                            id: "0".to_string(),
                        },],
                        incomplete: false,
                    },],
                })
            }
        );
    }
}
