// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod alchemist;
mod digest_repo;
mod dump_deps;
mod dump_package;
mod generate_repo;
mod ver_test;

use crate::alchemist::alchemist_main;
use anyhow::Result;
use clap::{Parser, Subcommand};
use ver_test::ver_test_main;

#[derive(Parser, Debug)]
#[command(multicall(true))]
struct Cli {
    #[clap(subcommand)]
    executables: Executables,
}

#[derive(Subcommand, Debug)]
enum Executables {
    Alchemist(alchemist::Args),

    #[command(name = "ver_test")] // Otherwise we get ver-test
    VerTest(ver_test::Args),
}

fn main() -> Result<()> {
    match Cli::parse().executables {
        Executables::Alchemist(args) => alchemist_main(args),
        Executables::VerTest(args) => ver_test_main(args),
    }
}
