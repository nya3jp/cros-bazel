// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use binarypackage::{BinaryPackage, OutputFileSpec, XpakSpec};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[arg(long, required = true)]
    binpkg: PathBuf,

    #[arg(
    long,
    help = "<inside path>=<outside path>: Extracts a file from the binpkg and writes it to the outside path"
    )]
    output_file: Vec<OutputFileSpec>,

    #[arg(
    long,
    help = "<XPAK key>=[?]<output file>: Write the XPAK key from the binpkg to the \
    specified file. If =? is used then an empty file is created if XPAK key doesn't exist."
    )]
    xpak: Vec<XpakSpec>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    BinaryPackage::extract_files(args.binpkg, &args.xpak, &args.output_file)
}
