// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Defines types corresponding to Build Event Protocol protobuf messages.
//!
//! See: https://github.com/bazelbuild/bazel/blob/HEAD/src/main/java/com/google/devtools/build/lib/buildeventstream/proto/build_event_stream.proto
//! See: https://github.com/bazelbuild/bazel/blob/HEAD/src/main/protobuf/command_line.proto

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
    StructuredCommandLine(StructuredCommandLineId),
    #[serde(untagged)]
    Other(PhantomValue),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildEventPayload {
    NamedSetOfFiles(NamedSetOfFiles),
    Completed(TargetComplete),
    StructuredCommandLine(StructuredCommandLine),
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
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub output_group: Vec<OutputGroup>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredCommandLineId {
    pub command_line_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredCommandLine {
    #[serde(default)]
    pub command_line_label: String,
    pub sections: Vec<CommandLineSection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandLineSection {
    #[serde(default)]
    pub section_label: String,
    #[serde(flatten)]
    pub section_type: CommandLineSectionType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandLineSectionType {
    ChunkList(ChunkList),
    OptionList(OptionList),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkList {
    pub chunk: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionList {
    pub option: Vec<OptionItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionItem {
    pub combined_form: String,
    pub option_name: String,
    pub option_value: String,
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
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
        // NamedSetOfFiles.
        assert_eq!(
            serde_json::from_value::<BuildEvent>(json!({
                "id": {
                    "namedSet": {"id": "5"}
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

        // TargetCompleted.
        assert_eq!(
            serde_json::from_value::<BuildEvent>(json!({
                "id": {
                    "targetCompleted": {
                        "label": "@portage/some/package:target",
                        "aspect": "//bazel/portage/build_defs:some.bzl%some_aspect"
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
                        }],
                        incomplete: false,
                    }],
                })
            }
        );

        // TargetCompleted for a failed case.
        assert_eq!(
            serde_json::from_value::<BuildEvent>(json!({
                "id": {
                    "targetCompleted": {
                        "label": "@portage/some/package:target",
                        "aspect": "//bazel/portage/build_defs:some.bzl%some_aspect"
                    },
                },
                "completed": {
                    "outputGroup": [
                        {
                            "name": "transitive_logs",
                            "fileSets": [{"id": "0"}],
                            "incomplete": true
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
                    success: false,
                    output_group: vec![OutputGroup {
                        name: "transitive_logs".to_string(),
                        file_sets: vec![NamedSetOfFilesId {
                            id: "0".to_string(),
                        }],
                        incomplete: true,
                    }],
                })
            }
        );

        // TargetCompleted for a failed case.
        assert_eq!(
            serde_json::from_value::<BuildEvent>(json!({
              "id": {
                "structuredCommandLine": {
                  "commandLineLabel": "canonical"
                }
              },
              "structuredCommandLine": {
                "commandLineLabel": "canonical",
                "sections": [
                  {
                    "sectionLabel": "executable",
                    "chunkList": {
                      "chunk": [
                        "bazel"
                      ]
                    }
                  },
                  {
                    "sectionLabel": "startup options",
                    "optionList": {
                      "option": [
                        {
                          "combinedForm": "--max_idle_secs=10800",
                          "optionName": "max_idle_secs",
                          "optionValue": "10800",
                          "effectTags": [
                            "EAGERNESS_TO_EXIT",
                            "LOSES_INCREMENTAL_STATE"
                          ]
                        },
                      ]
                    }
                  },
                  {
                    "sectionLabel": "command",
                    "chunkList": {
                      "chunk": [
                        "build"
                      ]
                    }
                  },
                  {
                    "sectionLabel": "command options",
                    "optionList": {
                      "option": [
                        {
                          "combinedForm": "--remote_instance_name=projects/chromeos-bot/instances/cros-rbe-nonrelease",
                          "optionName": "remote_instance_name",
                          "optionValue": "projects/chromeos-bot/instances/cros-rbe-nonrelease",
                          "effectTags": [
                            "UNKNOWN"
                          ]
                        }
                      ]
                    }
                  }
                ]
              }
            }))
            .expect("failed to deserialize BuildEvent"),
            BuildEvent {
                id: BuildEventId::StructuredCommandLine(StructuredCommandLineId {
                    command_line_label: "canonical".to_string(),
                }),
                payload: BuildEventPayload::StructuredCommandLine(StructuredCommandLine {
                    command_line_label: "canonical".to_string(),
                    sections: vec![
                        CommandLineSection {
                            section_label: "executable".to_string(),
                            section_type: CommandLineSectionType::ChunkList(ChunkList {
                                chunk: vec!["bazel".to_string()]
                            })
                        },
                        CommandLineSection {
                            section_label: "startup options".to_string(),
                            section_type: CommandLineSectionType::OptionList(OptionList {
                                option: vec![
                                    OptionItem {
                                        combined_form: "--max_idle_secs=10800".to_string(),
                                        option_name: "max_idle_secs".to_string(),
                                        option_value: "10800".to_string()
                                    }
                                ]
                            })
                        },
                        CommandLineSection {
                            section_label: "command".to_string(),
                            section_type: CommandLineSectionType::ChunkList(ChunkList {
                                chunk: vec!["build".to_string()]
                            })
                        },

                        CommandLineSection {
                            section_label: "command options".to_string(),
                            section_type: CommandLineSectionType::OptionList(OptionList {
                                option: vec![
                                    OptionItem {
                                        combined_form: "--remote_instance_name=projects/chromeos-bot/instances/cros-rbe-nonrelease".to_string(),
                                        option_name: "remote_instance_name".to_string(),
                                        option_value: "projects/chromeos-bot/instances/cros-rbe-nonrelease".to_string()
                                    }
                                ]
                            })
                        },
                    ]
                })
            }
        );
    }
}
