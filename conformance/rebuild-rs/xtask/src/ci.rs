// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Local entry point for the repository's non-release validation sequence.

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUF_INSTALL_GUIDANCE: &str = "Install or update the latest buf-toolchain release with:\n    \
     cargo install --force buf-toolchain\n\n\
     Then ensure $CARGO_HOME/bin is on PATH.\n\
     See https://buf.build/docs/cli/installation/ for official alternatives.";
const CARGO_MACHETE_INSTALL_COMMAND: &str = "cargo install --locked cargo-machete --version 0.9.2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkingDirectory {
    Repository,
    RebuildWorkspace,
}

#[derive(Clone, Copy, Debug)]
struct Step {
    label: &'static str,
    program: &'static str,
    args: &'static [&'static str],
    working_directory: WorkingDirectory,
}

impl Step {
    fn command_line(self) -> String {
        std::iter::once(self.program)
            .chain(self.args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

const CI_STEPS: &[Step] = &[
    Step {
        label: "Markdown lint",
        program: "rumdl",
        args: &["check", "."],
        working_directory: WorkingDirectory::Repository,
    },
    Step {
        label: "Protobuf build",
        program: "buf",
        args: &["build", "proto"],
        working_directory: WorkingDirectory::Repository,
    },
    Step {
        label: "Protobuf lint",
        program: "buf",
        args: &["lint", "proto"],
        working_directory: WorkingDirectory::Repository,
    },
    Step {
        label: "Protobuf formatting",
        program: "buf",
        args: &["format", "proto", "--diff", "--exit-code"],
        working_directory: WorkingDirectory::Repository,
    },
    Step {
        label: "JSON Schema validation",
        program: "jq",
        args: &["empty", "schema/YamlSigilSignature.v1alpha1.schema.json"],
        working_directory: WorkingDirectory::Repository,
    },
    Step {
        label: "Rust formatting",
        program: "cargo",
        args: &["fmt", "--all", "--check"],
        working_directory: WorkingDirectory::RebuildWorkspace,
    },
    Step {
        label: "Rust lint",
        program: "cargo",
        args: &[
            "clippy",
            "--locked",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        working_directory: WorkingDirectory::RebuildWorkspace,
    },
    Step {
        label: "Rust tests",
        program: "cargo",
        args: &["test", "--locked", "--workspace", "--all-features"],
        working_directory: WorkingDirectory::RebuildWorkspace,
    },
    // A Cargo-launched xtask must invoke this binary directly. In cargo-machete
    // 0.9.2, inherited Cargo package variables otherwise make `cargo machete`
    // parse its subcommand name as an input path.
    Step {
        label: "Unused Rust dependencies",
        program: "cargo-machete",
        args: &["--with-metadata"],
        working_directory: WorkingDirectory::Repository,
    },
    Step {
        label: "Rust dependency audit",
        program: "cargo",
        args: &["audit"],
        working_directory: WorkingDirectory::RebuildWorkspace,
    },
];

pub(crate) fn run(rebuild_root: &Path) -> io::Result<()> {
    require_cargo_machete()?;
    let buf = resolve_buf()?;
    let repository_root = rebuild_root
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("rebuild-rs is not nested under conformance/"))?;

    for step in CI_STEPS {
        let current_dir = match step.working_directory {
            WorkingDirectory::Repository => repository_root,
            WorkingDirectory::RebuildWorkspace => rebuild_root,
        };
        eprintln!("+ {} (cwd {})", step.command_line(), current_dir.display());
        let mut command = if step.program == "buf" {
            Command::new(&buf)
        } else {
            Command::new(step.program)
        };
        let status = command
            .args(step.args)
            .current_dir(current_dir)
            .status()
            .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", step.label)))?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "{} failed with {status}",
                step.label
            )));
        }
    }
    Ok(())
}

fn require_cargo_machete() -> io::Result<()> {
    let output = Command::new("cargo-machete")
        .arg("--version")
        .output()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "cargo-machete is required but was not found.\n\n\
                         Install it with:\n    {CARGO_MACHETE_INSTALL_COMMAND}"
                    ),
                )
            } else {
                io::Error::new(
                    error.kind(),
                    format!("failed to run cargo-machete: {error}"),
                )
            }
        })?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "cargo-machete --version failed with {}.\n\n{}",
            output.status, CARGO_MACHETE_INSTALL_COMMAND
        )));
    }

    Ok(())
}

fn resolve_buf() -> io::Result<PathBuf> {
    let buf = env::var_os("BUF")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("buf"));

    let output = Command::new(&buf)
        .arg("--version")
        .output()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Buf CLI is required but was not found.\n\n{BUF_INSTALL_GUIDANCE}"),
                )
            } else {
                io::Error::new(
                    error.kind(),
                    format!("failed to run {} --version: {error}", buf.display()),
                )
            }
        })?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(buf_prerequisite_error(format!(
            "{} --version failed with {}: {}",
            buf.display(),
            output.status,
            detail.trim()
        )));
    }

    if String::from_utf8_lossy(&output.stdout).trim().is_empty() {
        return Err(buf_prerequisite_error(format!(
            "{} --version returned no version.",
            buf.display()
        )));
    }

    Ok(buf)
}

fn buf_prerequisite_error(summary: String) -> io::Error {
    io::Error::other(format!("{summary}\n\n{BUF_INSTALL_GUIDANCE}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const AGENT_GUIDANCE: &str = include_str!("../../../../AGENTS.md");

    #[test]
    fn agent_guidance_documents_every_local_ci_step() {
        for step in CI_STEPS {
            let expected = step.command_line();
            assert!(
                AGENT_GUIDANCE.contains(&expected),
                "AGENTS.md is missing `{expected}`"
            );
        }
    }

    #[test]
    fn rebuild_steps_use_the_rebuild_working_directory() {
        assert_eq!(
            CI_STEPS
                .iter()
                .filter(|step| step.working_directory == WorkingDirectory::RebuildWorkspace)
                .count(),
            4
        );
    }

    #[test]
    fn buf_version_policy_is_aligned_and_actionable() {
        assert!(BUF_INSTALL_GUIDANCE.contains("cargo install --force buf-toolchain"));
        assert!(!BUF_INSTALL_GUIDANCE.contains("buf-toolchain@"));
        assert!(BUF_INSTALL_GUIDANCE.contains("$CARGO_HOME/bin"));
        assert!(BUF_INSTALL_GUIDANCE.contains("https://buf.build/docs/cli/installation/"));
        assert!(AGENT_GUIDANCE.contains("cargo install --force buf-toolchain"));
        assert!(AGENT_GUIDANCE.contains("rolling latest release"));
        assert!(AGENT_GUIDANCE.contains("Keep `cargo xtask ci` provider-neutral"));
    }

    #[test]
    fn cargo_machete_guidance_is_aligned_and_actionable() {
        assert_eq!(
            CARGO_MACHETE_INSTALL_COMMAND,
            "cargo install --locked cargo-machete --version 0.9.2"
        );
        assert!(AGENT_GUIDANCE.contains(CARGO_MACHETE_INSTALL_COMMAND));
        assert!(AGENT_GUIDANCE.contains("cargo-machete --with-metadata"));
    }
}
