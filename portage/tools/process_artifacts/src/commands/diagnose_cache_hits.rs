// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{collections::BTreeSet, io::Write, path::Path};

use crate::{
    processors::execlog::ExecLogProcessor,
    proto::spawn::exec_log_entry::{self, Spawn},
};
use anyhow::{Context, Result};
use itertools::Itertools;

type EntryType = exec_log_entry::Type;
type OutputType = exec_log_entry::output::Type;

pub fn diagnose_cache_hits(output_path: &Path, processor: &ExecLogProcessor) -> Result<()> {
    // Extract all spawn entries.
    let all_spawns: Vec<&Spawn> = processor
        .entries()
        .filter_map(|entry| {
            if let Some(EntryType::Spawn(spawn)) = &entry.r#type {
                Some(spawn)
            } else {
                None
            }
        })
        .collect();

    // Filter irrelevant spawn entries.
    let relevant_spawns: Vec<&Spawn> = all_spawns
        .iter()
        .copied()
        .filter(|spawn| {
            // Filter hash tracer spawns.
            if spawn.mnemonic == "HashTracer" {
                return false;
            }
            // Older execlogs have hash tracer spawns with right mnemonic, so filter them with
            // a hack.
            if let Some(last_arg) = spawn.args.last() {
                if last_arg.ends_with(".hash") {
                    return false;
                }
            }
            // PackageTar spawn is set to no-remote.
            if spawn.mnemonic == "PackageTar" {
                return false;
            }
            true
        })
        .collect();

    // Compute cache-miss spawns.
    let cache_miss_spawns: Vec<&Spawn> = relevant_spawns
        .iter()
        .copied()
        .filter(|spawn| !spawn.cache_hit)
        .collect();

    // Compute the union of all output files from cache-miss spawns.
    let cache_miss_spawn_outputs: BTreeSet<i32> = cache_miss_spawns
        .iter()
        .flat_map(|spawn| {
            spawn
                .outputs
                .iter()
                .filter_map(|output| match output.r#type {
                    Some(OutputType::FileId(id)) => Some(id),
                    Some(OutputType::DirectoryId(id)) => Some(id),
                    Some(OutputType::UnresolvedSymlinkId(id)) => Some(id),
                    _ => None,
                })
        })
        .collect();

    // Find all input sets containing any of cache-miss spawn outputs.
    let non_leaf_input_sets: BTreeSet<i32> = processor
        .intersecting_input_sets(cache_miss_spawn_outputs)?
        .into_iter()
        .collect();

    // Compute "leaf" cache-miss spawns whose input set doesn't contain outputs from other
    // cache-miss spawns.
    let (leaf_cache_miss_spawns, non_leaf_cache_miss_spawns): (Vec<&Spawn>, Vec<&Spawn>) =
        cache_miss_spawns
            .iter()
            .copied()
            .sorted_by_cached_key(|spawn| (spawn.target_label.clone(), spawn.mnemonic.clone()))
            .partition(|spawn| {
                !non_leaf_input_sets.contains(&spawn.input_set_id)
                    && !non_leaf_input_sets.contains(&spawn.tool_set_id)
            });

    // Finally, print reports.
    let mut out = std::fs::File::create(output_path)
        .with_context(|| format!("Failed to create {}", output_path.display()))?;
    writeln!(&mut out, "======= cache hit diagnosis =======")?;
    writeln!(&mut out, "All actions: {}", all_spawns.len())?;
    writeln!(&mut out, "Non-trivial actions: {}", relevant_spawns.len())?;
    writeln!(&mut out, "Cache-miss actions: {}", cache_miss_spawns.len())?;
    writeln!(
        &mut out,
        "Leaf cache-miss actions: {}",
        leaf_cache_miss_spawns.len(),
    )?;
    for s in leaf_cache_miss_spawns {
        writeln!(&mut out, "        {} [{}]", s.target_label, s.mnemonic)?;
    }
    writeln!(
        &mut out,
        "Non-leaf cache-miss actions: {}",
        non_leaf_cache_miss_spawns.len(),
    )?;
    for s in non_leaf_cache_miss_spawns {
        writeln!(&mut out, "        {} [{}]", s.target_label, s.mnemonic)?;
    }
    writeln!(&mut out, "======= end cache hit diagnosis =======")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::proto::spawn::{
        exec_log_entry::{File, InputSet},
        ExecLogEntry,
    };

    use super::*;

    #[test]
    fn smoke() -> Result<()> {
        let entries = vec![
            ExecLogEntry {
                id: 0,
                r#type: Some(EntryType::InputSet(InputSet {
                    file_ids: vec![],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 1,
                r#type: Some(EntryType::InputSet(InputSet {
                    file_ids: vec![10, 11],
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 10,
                r#type: Some(EntryType::File(File {
                    path: "x".to_string(),
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 11,
                r#type: Some(EntryType::File(File {
                    path: "y".to_string(),
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 100,
                r#type: Some(EntryType::Spawn(Spawn {
                    target_label: "//a".to_string(),
                    mnemonic: "A".to_string(),
                    cache_hit: true,
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 101,
                r#type: Some(EntryType::Spawn(Spawn {
                    target_label: "//b".to_string(),
                    mnemonic: "B".to_string(),
                    cache_hit: false,
                    ..Default::default()
                })),
            },
            ExecLogEntry {
                id: 102,
                r#type: Some(EntryType::Spawn(Spawn {
                    target_label: "//c".to_string(),
                    mnemonic: "C".to_string(),
                    cache_hit: false,
                    input_set_id: 1,
                    ..Default::default()
                })),
            },
        ];
        let processor = ExecLogProcessor::from(&entries);

        let output_file = tempfile::NamedTempFile::new()?;
        let output_path = output_file.path();

        diagnose_cache_hits(output_path, &processor)?;

        assert_eq!(
            std::fs::read_to_string(output_path)?,
            r#"======= cache hit diagnosis =======
All actions: 3
Non-trivial actions: 3
Cache-miss actions: 2
Leaf cache-miss actions: 2
        //b [B]
        //c [C]
Non-leaf cache-miss actions: 0
======= end cache hit diagnosis =======
"#
        );

        Ok(())
    }
}
