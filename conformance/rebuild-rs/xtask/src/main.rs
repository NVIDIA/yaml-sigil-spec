// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Developer-only tasks for the `rebuild-rs` workspace.
//!
//! Invoked via the `cargo xtask ...` alias defined in
//! `.cargo/config.toml`. Provides:
//!
//! ```text
//! cargo xtask ci
//! cargo xtask update-acvp [--commit <hash>]
//! ```
//!
//! which refreshes the vendored NIST ACVP-Server ECDSA SigGen test
//! vectors. The xtask is intentionally dependency-free (`std` only);
//! HTTP fetching is delegated to `curl`, which is universally
//! available on the dev environments this targets.

mod ci;

use std::env;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use yamlsigil_pinned_dir::PinnedDir;

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
        "ci" => {
            let remaining: Vec<_> = args.collect();
            if is_help_request(&remaining) {
                print_usage();
                ExitCode::SUCCESS
            } else if !remaining.is_empty() {
                eprintln!("ci does not accept arguments");
                print_usage();
                ExitCode::FAILURE
            } else {
                match ci::run(&workspace_root()) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("ci failed: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
        }
        "update-acvp" => {
            let remaining: Vec<_> = args.collect();
            if is_help_request(&remaining) {
                print_usage();
                return ExitCode::SUCCESS;
            }
            match parse_update_args(remaining.into_iter()) {
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
            }
        }
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
         cargo xtask ci\n  \
         cargo xtask update-acvp [--commit <hash>]\n\n\
         Runs the repository's complete non-release validation sequence.\n\n\
         Refreshes vendor/acvp/{VENDORED_FILE_NAME} and the matching\n\
         vendor/acvp/README.md to track the requested commit of\n\
         https://github.com/{UPSTREAM_REPO}."
    );
}

fn is_help_request(args: &[String]) -> bool {
    matches!(args, [arg] if matches!(arg.as_str(), "help" | "--help" | "-h"))
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
    let root = workspace_root().canonicalize()?;
    let root_dir = PinnedDir::open(&root)?;
    let vendor_root = root_dir.ensure_child("vendor")?;
    let vendor_dir = vendor_root.ensure_child("acvp")?;

    let url = format!("https://raw.githubusercontent.com/{UPSTREAM_REPO}/{commit}/{UPSTREAM_PATH}");
    let dest = root.join("vendor/acvp").join(VENDORED_FILE_NAME);
    let readme_path = root.join("vendor/acvp/README.md");

    println!("fetching: {url}");
    println!("dest:     {}", dest.display());

    let output = Command::new("curl")
        .arg("--fail")
        .arg("--location")
        .arg("--silent")
        .arg("--show-error")
        .arg(&url)
        .output()?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!(
            "curl exited with status {}: {}",
            output.status,
            detail.trim()
        )));
    }

    let size = u64::try_from(output.stdout.len())
        .map_err(|_| io::Error::other("download size does not fit u64"))?;
    vendor_dir.replace_regular_file(VENDORED_FILE_NAME, &output.stdout)?;

    let readme = render_readme(commit, size);
    vendor_dir.replace_regular_file("README.md", readme.as_bytes())?;

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
         The National Institute of Standards and Technology is explicitly\n\
         acknowledged as the source of this test data. The local file name\n\
         was changed; its contents were not modified. The NIST notice that\n\
         governs this snapshot is reproduced in\n\
         [`THIRD_PARTY_NOTICES.md`](../../../../THIRD_PARTY_NOTICES.md) and\n\
         must remain with distributions of this vendored file.\n\
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
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let sequence = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "yamlsigil-xtask-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        path
    }

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

    #[cfg(unix)]
    #[test]
    fn replace_regular_file_rejects_symlink_destination() {
        use std::os::unix::fs::symlink;

        let dir = test_dir("reject-destination-symlink");
        let target = dir.join("target.txt");
        fs::write(&target, b"keep").expect("write target");
        let destination = dir.join("destination.txt");
        symlink(&target, &destination).expect("create destination symlink");
        let pinned = PinnedDir::open(&dir).expect("pin vendor directory");

        let error = pinned
            .replace_regular_file("destination.txt", b"replace")
            .expect_err("destination symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&target).expect("read target"), b"keep");
        drop(pinned);
        fs::remove_file(destination).expect("remove destination symlink");
        fs::remove_file(target).expect("remove target");
        fs::remove_dir(dir).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn ensure_directory_rejects_symlink() {
        use std::os::unix::fs::symlink;

        let root = test_dir("reject-directory-symlink");
        let target = root.join("target");
        fs::create_dir(&target).expect("create target directory");
        let linked = root.join("linked");
        symlink(&target, &linked).expect("create directory symlink");
        let pinned = PinnedDir::open(&root).expect("pin workspace root");

        let error = pinned
            .ensure_child("linked")
            .expect_err("directory symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        drop(pinned);
        fs::remove_file(linked).expect("remove directory symlink");
        fs::remove_dir(target).expect("remove target directory");
        fs::remove_dir(root).expect("remove test root");
    }
}
