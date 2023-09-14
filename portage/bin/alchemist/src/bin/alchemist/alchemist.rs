// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::os::unix::fs;
use std::path::Path;
use std::sync::Arc;

use std::{env::current_dir, path::PathBuf};

use crate::digest_repo::digest_repo_main;
use crate::dump_package::dump_package_main;
use crate::generate_repo::generate_repo_main;

use alchemist::common::is_inside_chroot;
use alchemist::fakechroot;
use alchemist::toolchain::ToolchainConfig;
use alchemist::{
    config::{
        bundle::ConfigBundle, profile::Profile, site::SiteSettings, ConfigNode, ConfigNodeValue,
        ConfigSource, PackageMaskKind, PackageMaskUpdate, SimpleConfigSource, UseUpdate,
        UseUpdateFilter, UseUpdateKind,
    },
    ebuild::{metadata::CachedEBuildEvaluator, CachedPackageLoader, PackageLoader},
    fakechroot::{enter_fake_chroot, PathTranslator},
    repository::RepositorySet,
    resolver::PackageResolver,
    toolchain::load_toolchains,
};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(name = "alchemist")]
#[command(author = "ChromiumOS Authors")]
#[command(about = "Analyzes Portage trees", long_about = None)]
pub struct Args {
    /// Board name to build packages for.
    #[arg(short = 'b', long, value_name = "NAME")]
    board: Option<String>,

    /// Build packages for the host.
    #[arg(long)]
    host: bool,

    /// Profile of the board.
    #[arg(short = 'p', long, value_name = "PROFILE", default_value = "base")]
    profile: String,

    /// Name of the host repository.
    #[arg(long, value_name = "NAME", default_value = "amd64-host")]
    host_board: String,

    /// Profile name of the host target.
    #[arg(long, value_name = "PROFILE", default_value = "sdk/bootstrap")]
    host_profile: String,

    // For use in Alchemist's unit tests.
    #[arg(long, default_value_t = false, hide = true)]
    skip_unshare_for_testing: bool,

    /// Path to the ChromiumOS source directory root.
    /// If unset, it is inferred from the current directory.
    #[arg(short = 's', long, value_name = "DIR")]
    source_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Dumps information of packages.
    DumpPackage {
        #[command(flatten)]
        args: crate::dump_package::Args,
    },
    /// Generates a Bazel repository containing overlays and packages.
    GenerateRepo {
        /// Output directory path.
        #[arg(short = 'o', long, value_name = "PATH")]
        output_dir: PathBuf,

        #[arg(long)]
        /// An output path for a json-encoded Vec<deps::Repository>.
        output_repos_json: PathBuf,
    },
    /// Generates a digest of the repository that can be used to indicate if
    /// any of the overlays, ebuilds, eclasses, etc have changed.
    DigestRepo {
        /// Directory used to store a (file_name, mtime) => digest cache.
        #[command(flatten)]
        args: crate::digest_repo::Args,
    },
}

fn default_source_dir() -> Result<PathBuf> {
    for dir in current_dir()?.ancestors() {
        if dir.join(".repo").exists() {
            return Ok(dir.to_owned());
        }
    }
    bail!(
        "Cannot locate the CrOS source checkout directory from the current directory; \
         consider passing --source-dir option"
    );
}

fn build_override_config_source() -> SimpleConfigSource {
    let mut masked = vec![
        // HACK: Mask chromeos-base/chromeos-lacros-9999 as it's not functional.
        PackageMaskUpdate {
            kind: PackageMaskKind::Mask,
            atom: "=chromeos-base/chromeos-lacros-9999".parse().unwrap(),
        },
        // We don't want to build 9999 llvm-project ebuilds as they currently require
        // the whole .git directory. This causes problems because everything will
        // cache bust between hosts and syncs.
        PackageMaskUpdate {
            kind: PackageMaskKind::Mask,
            atom: "=sys-libs/scudo-9999".parse().unwrap(),
        },
        PackageMaskUpdate {
            kind: PackageMaskKind::Mask,
            atom: "=sys-devel/llvm-9999".parse().unwrap(),
        },
    ];
    let toolchain_categories = [
        "sys-libs",
        "cross-aarch64-cros-linux-gnu",
        "cross-x86_64-cros-linux-gnux32",
        "cross-i686-cros-linux-gnu",
        "cross-x86_64-cros-linux-gnu",
        "cross-armv7m-cros-eabi",
        "cross-armv7a-cros-linux-gnueabihf",
    ];

    let toolchain_packages = ["libcxx", "compiler-rt", "llvm-libunwind"];
    for category in toolchain_categories {
        for package_name in toolchain_packages {
            masked.push(PackageMaskUpdate {
                kind: PackageMaskKind::Mask,
                atom: format!("={category}/{package_name}-9999").parse().unwrap(),
            });
        }
    }

    let nodes = vec![
        ConfigNode {
            sources: vec![],
            value: ConfigNodeValue::PackageMasks(masked),
        },
        // We run the hooks in the chrome repository rule, so prevent them
        // from running in the ebuild action. We also have this USE flag hard
        // coded in build_package.sh.
        ConfigNode {
            sources: vec![],
            value: ConfigNodeValue::Uses(vec![
                UseUpdate {
                    kind: UseUpdateKind::Set,
                    filter: UseUpdateFilter {
                        atom: Some("chromeos-base/chrome-icu".parse().unwrap()),
                        stable_only: false,
                    },
                    use_tokens: "-runhooks".to_string(),
                },
                UseUpdate {
                    kind: UseUpdateKind::Set,
                    filter: UseUpdateFilter {
                        atom: Some("chromeos-base/chromeos-chrome".parse().unwrap()),
                        stable_only: false,
                    },
                    use_tokens: "-runhooks".to_string(),
                },
            ]),
        },
    ];
    SimpleConfigSource::new(nodes)
}

fn setup_tools() -> Result<TempDir> {
    let current_exec = std::env::current_exe()?;

    let tools_dir = tempfile::tempdir()?;

    fs::symlink(&current_exec, tools_dir.path().join("ver_test"))?;
    fs::symlink(&current_exec, tools_dir.path().join("ver_rs"))?;

    Ok(tools_dir)
}

/// Container that contains all the data structures for a specific board.
pub struct TargetData {
    pub board: String,
    pub profile: String,
    pub repos: Arc<RepositorySet>,
    pub config: Arc<ConfigBundle>,
    pub loader: Arc<CachedPackageLoader>,
    pub resolver: PackageResolver,
    pub toolchains: ToolchainConfig,
}

fn load_board(
    repos: RepositorySet,
    evaluator: &Arc<CachedEBuildEvaluator>,
    board: &str,
    profile_name: &str,
    root_dir: &Path,
) -> Result<TargetData> {
    let repos = Arc::new(repos);

    // Load configurations.
    let config = Arc::new({
        let profile = Profile::load_default(root_dir, &repos)?;
        let site_settings = SiteSettings::load(root_dir)?;
        let override_source = build_override_config_source();

        ConfigBundle::from_sources(vec![
            // The order matters.
            Box::new(profile) as Box<dyn ConfigSource>,
            Box::new(site_settings) as Box<dyn ConfigSource>,
            Box::new(override_source) as Box<dyn ConfigSource>,
        ])
    });

    // Force accept 9999 ebuilds when running outside a cros chroot.
    let force_accept_9999_ebuilds = !is_inside_chroot()?;

    let loader = Arc::new(CachedPackageLoader::new(PackageLoader::new(
        Arc::clone(evaluator),
        Arc::clone(&config),
        force_accept_9999_ebuilds,
    )));

    let resolver =
        PackageResolver::new(Arc::clone(&repos), Arc::clone(&config), Arc::clone(&loader));

    let toolchains = load_toolchains(&repos)?;

    Ok(TargetData {
        board: board.to_string(),
        profile: profile_name.to_string(),
        repos,
        config,
        loader,
        resolver,
        toolchains,
    })
}

pub fn alchemist_main(args: Args) -> Result<()> {
    if args.board.is_none() && !args.host {
        bail!("Either --board or --host should be specified.")
    }
    if args.board.is_some() && args.host {
        bail!("--board and --host shouldn't be specified together.");
    }

    let source_dir = match args.source_dir {
        Some(s) => PathBuf::from(s),
        None => default_source_dir()?,
    };
    let src_dir = source_dir.join("src");

    let host_target = fakechroot::BoardTarget {
        board: &args.host_board,
        profile: &args.host_profile,
    };

    let board_target = if let Some(board) = args.board.as_ref() {
        let profile = &args.profile;

        // We don't support a board ROOT with two different profiles.
        if board == host_target.board && profile != host_target.profile {
            bail!(
                "--profile ({}) must match --host-profile ({})",
                profile,
                host_target.profile
            );
        }

        Some(fakechroot::BoardTarget { board, profile })
    } else {
        None
    };

    // Enter a fake chroot when running outside a cros chroot.
    let translator = if is_inside_chroot()? {
        // TODO: What do we do here?
        PathTranslator::noop()
    } else {
        let targets = if let Some(board_target) = board_target.as_ref() {
            if board_target.board == host_target.board {
                vec![&host_target]
            } else {
                vec![board_target, &host_target]
            }
        } else {
            vec![&host_target]
        };
        enter_fake_chroot(&targets, &source_dir, args.skip_unshare_for_testing)?
    };

    let tools_dir = setup_tools()?;

    let target_data = if let Some(board_target) = board_target {
        let root_dir = Path::new("/build").join(board_target.board);
        let repos = RepositorySet::load(&root_dir)?;

        Some((root_dir, repos, board_target))
    } else {
        None
    };

    let host_data = {
        let root_dir = Path::new("/build").join(host_target.board);
        match RepositorySet::load(&root_dir) {
            Ok(repos) => Some((root_dir, repos, host_target)),
            Err(e) => {
                // TODO: We need to eventually make this fatal.
                eprintln!(
                    "Failed to load {} repos, skipping host tools: {}",
                    host_target.board, e
                );
                None
            }
        }
    };

    // We share an evaluator between both config ROOTS so we only have to parse
    // the ebuilds once.
    let evaluator = Arc::new(CachedEBuildEvaluator::new(
        [&target_data, &host_data]
            .into_iter()
            .filter_map(|x| x.as_ref())
            .flat_map(|x| x.1.get_repos())
            .cloned()
            .collect(),
        tools_dir.path(),
    ));

    let target = if let Some((root_dir, repos, board_target)) = target_data {
        Some(load_board(
            repos,
            &evaluator,
            board_target.board,
            board_target.profile,
            &root_dir,
        )?)
    } else {
        None
    };

    let host = host_data.and_then(|(root_dir, repos, host_target)| {
        match load_board(
            repos,
            &evaluator,
            host_target.board,
            host_target.profile,
            &root_dir,
        ) {
            Ok(data) => Some(data),
            Err(e) => {
                // TODO: We need to eventually make this fatal.
                eprintln!("Failed to load {} config: {}", host_target.board, e);
                None
            }
        }
    });

    match args.command {
        Commands::DumpPackage { args: local_args } => {
            dump_package_main(host.as_ref(), target.as_ref(), local_args)?;
        }
        Commands::GenerateRepo {
            output_dir,
            output_repos_json,
        } => {
            generate_repo_main(
                host.as_ref(),
                target.as_ref(),
                &translator,
                &src_dir,
                &output_dir,
                &output_repos_json,
            )?;
        }
        Commands::DigestRepo { args: local_args } => {
            digest_repo_main(host.as_ref(), target.as_ref(), local_args)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::remove_file;

    use super::*;
    use tempfile::tempdir;
    use testutil::compare_with_golden_data;

    #[used]
    #[link_section = ".init_array"]
    static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;

    const TESTDATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/alchemist/testdata");

    #[test]
    fn main_test() -> Result<()> {
        let temp_dir = tempdir()?;
        let temp = temp_dir.path();
        let output_dir = temp.join("portage-repo");
        let deps_file = temp.join("output_repos.json");

        let testdata_dir = Path::new(TESTDATA_DIR);
        let input_dir = testdata_dir.join("input").to_str().unwrap().to_owned();

        let args = Args {
            board: Some("amd64-generic".into()),
            host: false,
            profile: "base".into(),
            host_board: "amd64-host".into(),
            host_profile: "sdk/bootstrap".into(),
            source_dir: Some(input_dir),
            skip_unshare_for_testing: true,
            command: Commands::GenerateRepo {
                output_dir: output_dir.clone(),
                output_repos_json: deps_file,
            },
        };

        alchemist_main(args)?;

        // trace.json changes every time we run.
        remove_file(output_dir.join("trace.json"))?;

        let golden_dir = testdata_dir.join("golden");
        compare_with_golden_data(&output_dir, &golden_dir)?;

        Ok(())
    }
}
