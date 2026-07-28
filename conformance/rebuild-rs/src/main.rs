// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Rebuild every fixture in every conformance subdirectory.
//!
//! Default behaviour: writes fixtures under `$CONFORMANCE_ROOT`
//! (defaults to `/work`, matching the Dockerfile's WORKDIR + volume
//! mount layout).
//!
//! Local usage outside the container:
//!
//! ```text
//! CONFORMANCE_ROOT=$(realpath ..) cargo run --release --locked
//! ```
//!
//! (run from inside `conformance/rebuild-rs/`).

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

mod acvp;
mod alg_ecdsa;
mod alg_ed25519;
mod b64;
mod base64_gen;
mod key_id;
mod p256;
mod protobuf_conformance;
mod schema_alignment;
mod util;
mod wire;
mod yaml_decomposition;
mod yaml_signature_conformance;

type Generator = fn(&std::path::Path) -> std::io::Result<()>;

const SUBDIRS: &[(&str, Generator)] = &[
    ("yaml-decomposition", yaml_decomposition::generate),
    ("protobuf-conformance", protobuf_conformance::generate),
    (
        "yaml-signature-conformance",
        yaml_signature_conformance::generate,
    ),
    ("schema-alignment", schema_alignment::generate),
    ("key-id", key_id::generate),
    ("base64", base64_gen::generate),
    ("alg-ed25519", alg_ed25519::generate),
    ("alg-ecdsa", alg_ecdsa::generate),
];

fn main() -> ExitCode {
    let root_str = env::var("CONFORMANCE_ROOT").unwrap_or_else(|_| "/work".to_string());
    let root = match PathBuf::from(&root_str).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("work root does not exist: {root_str} ({e})");
            return ExitCode::FAILURE;
        }
    };
    for (name, generator) in SUBDIRS {
        let subdir = root.join(name);
        match fs::symlink_metadata(&subdir) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => {
                eprintln!(
                    "subdirectory is not a non-symlink directory: {}",
                    subdir.display()
                );
                return ExitCode::FAILURE;
            }
            Err(e) => {
                eprintln!("cannot inspect subdirectory {}: {e}", subdir.display());
                return ExitCode::FAILURE;
            }
        }
        println!("=== {name} ===");
        if let Err(e) = generator(&subdir) {
            eprintln!("{name}: {e}");
            return ExitCode::FAILURE;
        }
    }
    println!("\ndone — rebuild complete.");
    ExitCode::SUCCESS
}
