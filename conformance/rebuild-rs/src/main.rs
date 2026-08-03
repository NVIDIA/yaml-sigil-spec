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
use std::path::PathBuf;
use std::process::ExitCode;
use yamlsigil_pinned_dir::PinnedDir;

mod acvp;
mod alg_ecdsa;
mod alg_ed25519;
mod b64;
mod base64_gen;
mod key_id;
mod p256;
mod protobuf_conformance;
mod schema_alignment;
mod transcoding;
mod util;
mod wire;
mod yaml_decomposition;
mod yaml_signature_conformance;

type Generator = fn(&PinnedDir) -> std::io::Result<()>;

const SUBDIRS: &[(&str, Generator)] = &[
    ("yaml-decomposition", yaml_decomposition::generate),
    ("protobuf-conformance", protobuf_conformance::generate),
    (
        "yaml-signature-conformance",
        yaml_signature_conformance::generate,
    ),
    ("schema-alignment", schema_alignment::generate),
    ("transcoding", transcoding::generate),
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
    let root_dir = match PinnedDir::open(&root) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("cannot pin work root {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    for (name, generator) in SUBDIRS {
        let subdir = match root_dir.open_child(name) {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("cannot pin subdirectory {}: {e}", root.join(name).display());
                return ExitCode::FAILURE;
            }
        };
        println!("=== {name} ===");
        if let Err(e) = generator(&subdir) {
            eprintln!("{name}: {e}");
            return ExitCode::FAILURE;
        }
    }
    println!("\ndone — rebuild complete.");
    ExitCode::SUCCESS
}
