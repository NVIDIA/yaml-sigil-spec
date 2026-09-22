// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.len() == 1 {
        args.push("--help".into());
    } else if args.len() == 3 && args[2] == "help" {
        args[2] = "--help".into();
    }
    match xtask::run(xtask::Cli::parse_from(args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask failed: {error}");
            ExitCode::FAILURE
        }
    }
}
