// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod alchemist;
mod digest_repo;
mod dump_package;
mod generate_repo;
mod ver_test;

use std::process::ExitCode;

use crate::alchemist::alchemist_main;
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

fn main() -> ExitCode {
    let result = match Cli::parse().executables {
        Executables::Alchemist(args) => alchemist_main(args),
        Executables::VerTest(args) => ver_test_main(args),
    };
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ERROR: {:?}", err);
            if std::env::var("RUST_BACKTRACE").is_err() {
                eprintln!("Hint: Set RUST_BACKTRACE=1 to print stack traces");
            }
            ExitCode::FAILURE
        }
    }
}
