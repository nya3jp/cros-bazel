// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{bail, Result};
use itertools::Itertools;

use crate::proto::build_event_stream::{
    BuildEvent, BuildEventId, BuildEventPayload, File, NamedSetOfFiles, NamedSetOfFilesId,
};

/// Provides fast merge operations over [`NamedSetOfFiles`].
///
/// [`NamedSetOfFiles`] can be deeply nested with duplications (like diamond inheritance).
/// Therefore, on keeping track of files, it is important to also keep track of named set IDs that
/// are already merged to the set, in order to avoid repeatedly merging the same named sets.
#[derive(Clone, Default)]
struct FastFileSet {
    // Invariant: named sets are closed over subsets, i.e. all subsets of the named sets are also in
    // the named sets.
    children: BTreeMap<NamedSetOfFilesId, NamedSetOfFiles>,
}

impl FastFileSet {
    /// Creates a new empty set.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns an iterator of files included in this set.
    pub fn files(&self) -> impl Iterator<Item = &File> {
        self.children
            .values()
            .flat_map(|named_set| named_set.files.iter())
            .sorted()
            .dedup()
    }
}

/// An index of [`BuildEvent`] that provides fast file set operations.
struct FastFileSetIndex {
    index: BTreeMap<NamedSetOfFilesId, NamedSetOfFiles>,
}

impl<'a, T> From<T> for FastFileSetIndex
where
    T: IntoIterator<Item = &'a BuildEvent>,
{
    /// Constructs a new [`FastFileSetIndex`] from an iterator of [`BuildEvent`].
    fn from(into_iter: T) -> Self {
        let index = into_iter
            .into_iter()
            .filter_map(|event| {
                let BuildEventId::NamedSet(id) = &event.id else {
                    return None;
                };
                let BuildEventPayload::NamedSetOfFiles(named_set) = &event.payload else {
                    return None;
                };
                Some((id.clone(), named_set.clone()))
            })
            .collect();
        Self { index }
    }
}

impl FastFileSetIndex {
    /// Merges a named set of files to [`FastFileSet`].
    pub fn merge(&self, mut fs: FastFileSet, id: &NamedSetOfFilesId) -> Result<FastFileSet> {
        if fs.children.contains_key(id) {
            return Ok(fs);
        }
        let Some(entry) = self.index.get(id) else {
            bail!("NamedSetOfFiles {} not found", id.id);
        };
        fs.children.insert(id.clone(), entry.clone());
        for subset in &entry.file_sets {
            fs = self.merge(fs, subset)?;
        }
        Ok(fs)
    }
}

pub struct BuildEventProcessor<'a> {
    events: &'a [BuildEvent],
    index: FastFileSetIndex,
}

impl<'a> From<&'a Vec<BuildEvent>> for BuildEventProcessor<'a> {
    /// Constructs a new [`BuildEventProcessor`] from an iterator of [`BuildEvent`].
    fn from(events: &'a Vec<BuildEvent>) -> Self {
        Self {
            events: events.as_slice(),
            index: events.into(),
        }
    }
}

fn path_for_file(file: &File) -> PathBuf {
    let mut path = PathBuf::new();
    for prefix in &file.path_prefix {
        path.push(prefix);
    }
    path.push(&file.name);
    path
}

impl BuildEventProcessor<'_> {
    /// Returns the workspace relative path to all files in the specified output group.
    pub fn output_group_files(&self, output_group_name: &str) -> Result<Vec<PathBuf>> {
        let mut fileset = FastFileSet::new();

        for event in self.events {
            let BuildEventId::TargetCompleted(_) = &event.id else {
                continue;
            };
            let BuildEventPayload::Completed(complete) = &event.payload else {
                // This can happen when the target was incomplete due to build errors.
                continue;
            };
            for output_group in &complete.output_group {
                if output_group.name != output_group_name {
                    continue;
                }
                for file_set_id in &output_group.file_sets {
                    fileset = self.index.merge(fileset, file_set_id)?;
                }
            }
        }

        Ok(fileset.files().map(path_for_file).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::build_event_stream::{OutputGroup, TargetComplete, TargetCompletedId};

    use super::*;

    #[test]
    fn output_group() -> Result<()> {
        let events = vec![
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "a".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: "a.txt".to_string(),
                        path_prefix: vec![],
                    }],
                    file_sets: vec![],
                }),
            },
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "b".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: "b.txt".to_string(),
                        path_prefix: vec![],
                    }],
                    file_sets: vec![],
                }),
            },
            BuildEvent {
                id: BuildEventId::TargetCompleted(TargetCompletedId {
                    label: "//foo".to_string(),
                    aspect: None,
                }),
                payload: BuildEventPayload::Completed(TargetComplete {
                    success: true,
                    output_group: vec![
                        OutputGroup {
                            name: "transitive_logs".to_string(),
                            file_sets: vec![NamedSetOfFilesId {
                                id: "a".to_string(),
                            }],
                            incomplete: false,
                        },
                        OutputGroup {
                            name: "other".to_string(),
                            file_sets: vec![NamedSetOfFilesId {
                                id: "b".to_string(),
                            }],
                            incomplete: false,
                        },
                    ],
                }),
            },
        ];

        let files = BuildEventProcessor::from(&events).output_group_files("transitive_logs")?;

        assert_eq!(files, vec![PathBuf::from("a.txt")]);

        Ok(())
    }

    #[test]
    fn deeply_nested_named_sets() -> Result<()> {
        // Create a fibonacci-like structure of named sets.
        let mut events: Vec<BuildEvent> = vec![
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "0".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: "0.txt".to_string(),
                        path_prefix: vec![],
                    }],
                    file_sets: vec![],
                }),
            },
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "1".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: "1.txt".to_string(),
                        path_prefix: vec![],
                    }],
                    file_sets: vec![],
                }),
            },
        ];
        for i in 2..=100 {
            events.push(BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId { id: i.to_string() }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: format!("{i}.txt"),
                        path_prefix: vec![],
                    }],
                    file_sets: vec![
                        NamedSetOfFilesId {
                            id: (i - 2).to_string(),
                        },
                        NamedSetOfFilesId {
                            id: (i - 1).to_string(),
                        },
                    ],
                }),
            })
        }
        events.push(BuildEvent {
            id: BuildEventId::TargetCompleted(TargetCompletedId {
                label: "//foo".to_string(),
                aspect: None,
            }),
            payload: BuildEventPayload::Completed(TargetComplete {
                output_group: vec![OutputGroup {
                    name: "transitive_logs".to_string(),
                    file_sets: vec![NamedSetOfFilesId {
                        id: "100".to_string(),
                    }],
                    incomplete: false,
                }],
                success: true,
            }),
        });

        let files = BuildEventProcessor::from(&events).output_group_files("transitive_logs")?;

        assert_eq!(
            files.into_iter().sorted().collect_vec(),
            (0..=100)
                .map(|i| PathBuf::from(format!("{i}.txt")))
                .sorted()
                .collect_vec(),
        );

        Ok(())
    }
}
