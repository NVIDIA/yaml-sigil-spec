// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Developer-only tasks for the `rebuild-rs` workspace.
//!
//! Invoked via the `cargo xtask ...` alias defined in
//! `.cargo/config.toml`. Currently provides:
//!
//! ```text
//! cargo xtask update-acvp [--commit <hash>]
//! ```
//!
//! which refreshes the vendored NIST ACVP-Server ECDSA SigGen test
//! vectors. The xtask is intentionally dependency-free (`std` only);
//! HTTP fetching is delegated to `curl`, which is universally
//! available on the dev environments this targets.

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// Pinned upstream commit of `usnistgov/ACVP-Server`. Bump this
/// constant (or pass `--commit <hash>`) when refreshing the vendored
/// vectors; the xtask rewrites both the JSON file and the vendor
/// README so the pin is always self-describing.
const DEFAULT_COMMIT: &str = "15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0";

const UPSTREAM_REPO: &str = "usnistgov/ACVP-Server";
const UPSTREAM_PATH: &str = "gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json";
const VENDORED_FILE_NAME: &str = "ECDSA-SigGen-FIPS186-5.json";

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "update-acvp" => match parse_update_args(args) {
            Ok(commit) => match update_acvp(&commit) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("update-acvp failed: {e}");
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("{e}");
                print_usage();
                ExitCode::FAILURE
            }
        },
        "" | "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown subcommand: {other}");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage:\n  \
         cargo xtask update-acvp [--commit <hash>]\n\n\
         Refreshes vendor/acvp/{VENDORED_FILE_NAME} and the matching\n\
         vendor/acvp/README.md to track the requested commit of\n\
         https://github.com/{UPSTREAM_REPO}."
    );
}

fn parse_update_args<I: Iterator<Item = String>>(mut args: I) -> Result<String, String> {
    let mut commit = DEFAULT_COMMIT.to_string();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--commit" => {
                commit = args
                    .next()
                    .ok_or_else(|| "--commit needs a hash argument".to_string())?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    if commit.is_empty() {
        return Err("commit hash is empty".into());
    }
    Ok(commit)
}

fn update_acvp(commit: &str) -> io::Result<()> {
    let root = workspace_root();
    let vendor_dir = root.join("vendor").join("acvp");
    fs::create_dir_all(&vendor_dir)?;

    let url = format!("https://raw.githubusercontent.com/{UPSTREAM_REPO}/{commit}/{UPSTREAM_PATH}");
    let dest = vendor_dir.join(VENDORED_FILE_NAME);

    println!("fetching: {url}");
    println!("dest:     {}", dest.display());

    let status = Command::new("curl")
        .arg("--fail")
        .arg("--location")
        .arg("--silent")
        .arg("--show-error")
        .arg("--output")
        .arg(&dest)
        .arg(&url)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "curl exited with status {status}"
        )));
    }

    let size = fs::metadata(&dest)?.len();

    let readme = render_readme(commit, size);
    let readme_path = vendor_dir.join("README.md");
    fs::write(&readme_path, readme)?;

    println!("wrote: {} ({size} bytes)", dest.display());
    println!("wrote: {}", readme_path.display());
    Ok(())
}

/// Compute the workspace root by walking up from the xtask crate's
/// `CARGO_MANIFEST_DIR`. Falls back to the current working directory
/// if the env var is unset (which shouldn't happen under `cargo run`).
fn workspace_root() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest = PathBuf::from(dir);
        if let Some(parent) = manifest.parent() {
            return parent.to_path_buf();
        }
    }
    env::current_dir().expect("cwd available")
}

fn render_readme(commit: &str, size: u64) -> String {
    let raw_url =
        format!("https://raw.githubusercontent.com/{UPSTREAM_REPO}/{commit}/{UPSTREAM_PATH}");
    let tree_url = format!("https://github.com/{UPSTREAM_REPO}/blob/{commit}/{UPSTREAM_PATH}");
    format!(
        "# Vendored ACVP test vectors\n\
         \n\
         This directory tracks a single file from the NIST\n\
         [Automated Cryptographic Validation Protocol (ACVP)](https://pages.nist.gov/ACVP/)\n\
         server's reference test-vector tree.\n\
         \n\
         | Field | Value |\n\
         | --- | --- |\n\
         | Upstream repo | <https://github.com/{UPSTREAM_REPO}> |\n\
         | Commit | `{commit}` |\n\
         | Upstream path | `{UPSTREAM_PATH}` |\n\
         | Browse on GitHub | <{tree_url}> |\n\
         | Vendored as | `vendor/acvp/{VENDORED_FILE_NAME}` ({size} bytes) |\n\
         | Pinned by | `xtask/src/main.rs` `DEFAULT_COMMIT` |\n\
         \n\
         ## What this is\n\
         \n\
         A NIST ACVP \"AFT\" (Algorithm Functional Test) vector set\n\
         for ECDSA signature generation under FIPS 186-5. Each test\n\
         group pins `(d, Q, k, message, r, s)` — i.e. the private key,\n\
         public key, ephemeral nonce, message, and expected signature\n\
         components — so the rebuilder can replay the sign and assert\n\
         byte-equality against the published `(r, s)`. The file\n\
         covers multiple curve / hash combinations; the rebuilder\n\
         filters for `curve = P-256` and `hashAlg = SHA2-256`.\n\
         \n\
         NIST-published works are in the public domain in the United\n\
         States (17 U.S.C. § 105). No attribution is required, but the\n\
         origin URL above is the canonical reference.\n\
         \n\
         ## Manual verification\n\
         \n\
         To confirm the vendored bytes by hand, fetch the same file at\n\
         the pinned commit and compare SHA-256 hashes:\n\
         \n\
         ```sh\n\
         # Compute the hash of the upstream file at the pinned commit.\n\
         curl -sL '{raw_url}' | sha256sum\n\
         \n\
         # Compare against the hash of the vendored copy.\n\
         sha256sum vendor/acvp/{VENDORED_FILE_NAME}\n\
         ```\n\
         \n\
         The two outputs MUST match. If they don't, the vendored file\n\
         has drifted from its upstream pin and that diff is itself a\n\
         finding to surface.\n\
         \n\
         ## Refreshing\n\
         \n\
         To bump the pin to a newer commit, edit `DEFAULT_COMMIT` in\n\
         `xtask/src/main.rs` and run:\n\
         \n\
         ```sh\n\
         cargo xtask update-acvp\n\
         ```\n\
         \n\
         Or pass an explicit commit hash (without bumping the default):\n\
         \n\
         ```sh\n\
         cargo xtask update-acvp --commit <hash>\n\
         ```\n\
         \n\
         The xtask rewrites both the JSON file and this README. This\n\
         file is regenerated on every run; do not edit it by hand.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_update_uses_default_commit_when_no_args() {
        let it: std::vec::IntoIter<String> = Vec::<String>::new().into_iter();
        let got = parse_update_args(it).expect("no args is valid");
        assert_eq!(got, DEFAULT_COMMIT);
    }

    #[test]
    fn parse_update_accepts_explicit_commit() {
        let it = vec!["--commit".into(), "deadbeef".into()].into_iter();
        let got = parse_update_args(it).expect("explicit commit is valid");
        assert_eq!(got, "deadbeef");
    }

    #[test]
    fn parse_update_rejects_unknown_arg() {
        let it = vec!["--bogus".into()].into_iter();
        assert!(parse_update_args(it).is_err());
    }

    #[test]
    fn parse_update_rejects_missing_commit_value() {
        let it = vec!["--commit".into()].into_iter();
        assert!(parse_update_args(it).is_err());
    }

    /// README MUST embed the commit hash so the pin is auditable
    /// from the vendor README alone.
    #[test]
    fn render_readme_embeds_commit() {
        let out = render_readme("0123456789abcdef", 42);
        assert!(out.contains("0123456789abcdef"));
        assert!(out.contains("42 bytes"));
        assert!(out.contains(UPSTREAM_REPO));
    }
}
