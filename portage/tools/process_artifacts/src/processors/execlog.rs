// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::collections::{BTreeMap, BTreeSet};

use crate::proto::spawn::{exec_log_entry, ExecLogEntry};
use anyhow::{bail, Result};

type EntryType = exec_log_entry::Type;

struct ExecLogIndex<'e> {
    entries: Vec<&'e ExecLogEntry>,
    index: BTreeMap<i32, &'e EntryType>,
}

impl<'e, T> From<T> for ExecLogIndex<'e>
where
    T: IntoIterator<Item = &'e ExecLogEntry>,
{
    /// Constructs [`ExecLogIndex`] from an iterator of [`ExecLogEntry`].
    fn from(iter: T) -> Self {
        let entries: Vec<_> = iter.into_iter().collect();
        let index = entries
            .iter()
            .copied()
            .filter_map(|entry| entry.r#type.as_ref().map(|t| (entry.id, t)))
            .collect();
        Self { entries, index }
    }
}

impl<'e> ExecLogIndex<'e> {
    /// Returns raw entries.
    pub fn entries(&self) -> impl Iterator<Item = &ExecLogEntry> {
        self.entries.iter().copied()
    }

    /// Looks up [`ExecLogEntry`] by ID.
    pub fn get(&self, id: i32) -> Option<&EntryType> {
        self.index.get(&id).copied()
    }
}

pub struct ExecLogProcessor<'e> {
    index: ExecLogIndex<'e>,
}

impl<'e, T> From<T> for ExecLogProcessor<'e>
where
    T: IntoIterator<Item = &'e ExecLogEntry>,
{
    /// Constructs [`ExecLogIndex`] from an iterator of [`ExecLogEntry`].
    fn from(iter: T) -> Self {
        Self { index: iter.into() }
    }
}

impl ExecLogProcessor<'_> {
    /// Returns raw entries.
    pub fn entries(&self) -> impl Iterator<Item = &ExecLogEntry> {
        self.index.entries()
    }

    /// Finds all input sets that contain any one of the specified files, and returns their IDs.
    pub fn intersecting_input_sets(
        &self,
        files: impl IntoIterator<Item = i32>,
    ) -> Result<Vec<i32>> {
        let files: BTreeSet<i32> = files.into_iter().collect();
        let mut cache: BTreeMap<i32, bool> = BTreeMap::new();
        let mut intersecting_input_sets: Vec<i32> = Vec::new();
        for entry in self.entries() {
            if let Some(EntryType::InputSet(_)) = &entry.r#type {
                if self.intersects_memoized(entry.id, &files, &mut cache)? {
                    intersecting_input_sets.push(entry.id);
                }
            }
        }
        Ok(intersecting_input_sets)
    }

    fn intersects_memoized(
        &self,
        input_set: i32,
        files: &BTreeSet<i32>,
        cache: &mut BTreeMap<i32, bool>,
    ) -> Result<bool> {
        if let Some(intersects) = cache.get(&input_set) {
            return Ok(*intersects);
        }
        let intersects = (|| -> Result<bool> {
            let Some(EntryType::InputSet(input_set)) = self.index.get(input_set) else {
                bail!("Input set {input_set} not found");
            };
            if input_set
                .file_ids
                .iter()
                .chain(input_set.directory_ids.iter())
                .chain(input_set.unresolved_symlink_ids.iter())
                .any(|file_id| files.contains(file_id))
            {
                return Ok(true);
            }
            for transitive_set_id in &input_set.transitive_set_ids {
                if self.intersects_memoized(*transitive_set_id, files, cache)? {
                    return Ok(true);
                }
            }
            Ok(false)
        })()?;
        cache.insert(input_set, intersects);
        Ok(intersects)
    }
}

#[cfg(test)]
mod tests {
    use exec_log_entry::{File, InputSet};

    use super::*;

    #[test]
    fn intersecting_input_sets() -> Result<()> {
        let entries = vec![
            ExecLogEntry {
                id: 1,
                r#type: Some(EntryType::File(File {
                    path: "x".to_string(),
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 2,
                r#type: Some(EntryType::File(File {
                    path: "y".to_string(),
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 11,
                r#type: Some(EntryType::InputSet(InputSet {
                    file_ids: vec![1],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 12,
                r#type: Some(EntryType::InputSet(InputSet {
                    file_ids: vec![2],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 13,
                r#type: Some(EntryType::InputSet(InputSet {
                    file_ids: vec![1, 2],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 101,
                r#type: Some(EntryType::InputSet(InputSet {
                    transitive_set_ids: vec![11],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 102,
                r#type: Some(EntryType::InputSet(InputSet {
                    transitive_set_ids: vec![12],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 103,
                r#type: Some(EntryType::InputSet(InputSet {
                    transitive_set_ids: vec![11, 12],
                    ..Default::default()
                })),
            },
        ];
        let processor = ExecLogProcessor::from(&entries);

        assert_eq!(processor.intersecting_input_sets([])?, Vec::<i32>::new());
        assert_eq!(
            processor.intersecting_input_sets([1])?,
            vec![11, 13, 101, 103]
        );
        assert_eq!(
            processor.intersecting_input_sets([2])?,
            vec![12, 13, 102, 103]
        );
        assert_eq!(
            processor.intersecting_input_sets([1, 2])?,
            vec![11, 12, 13, 101, 102, 103]
        );

        Ok(())
    }

    #[test]
    fn intersecting_input_sets_deeply_nested() -> Result<()> {
        let mut entries = vec![
            ExecLogEntry {
                id: 1,
                r#type: Some(EntryType::File(File {
                    path: "x".to_string(),
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 2,
                r#type: Some(EntryType::File(File {
                    path: "y".to_string(),
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 3,
                r#type: Some(EntryType::InputSet(InputSet {
                    file_ids: vec![1, 2],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 4,
                r#type: Some(EntryType::InputSet(InputSet {
                    file_ids: vec![2],
                    transitive_set_ids: vec![3],
                    ..Default::default()
                })),
            },
        ];
        for id in 5..1000 {
            entries.push(ExecLogEntry {
                id,
                r#type: Some(EntryType::InputSet(InputSet {
                    transitive_set_ids: vec![id - 2, id - 1],
                    ..Default::default()
                })),
            });
        }
        let processor = ExecLogProcessor::from(&entries);

        assert_eq!(processor.intersecting_input_sets([])?, Vec::<i32>::new());
        assert_eq!(
            processor.intersecting_input_sets([1])?,
            Vec::from_iter(3..1000)
        );
        assert_eq!(
            processor.intersecting_input_sets([2])?,
            Vec::from_iter(3..1000)
        );
        assert_eq!(
            processor.intersecting_input_sets([1, 2])?,
            Vec::from_iter(3..1000)
        );

        Ok(())
    }
}
