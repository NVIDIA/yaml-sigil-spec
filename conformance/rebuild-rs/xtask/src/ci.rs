// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Local entry point for the repository's non-release validation sequence.

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::isolated_process;

const BUF_INSTALL_GUIDANCE: &str = "Install or update the latest buf-toolchain release with:\n    \
     cargo install --force buf-toolchain\n\n\
     Then ensure $CARGO_HOME/bin is on PATH.\n\
     See https://buf.build/docs/cli/installation/ for official alternatives.";
const CARGO_MACHETE_INSTALL_COMMAND: &str = "cargo install --locked cargo-machete --version 0.9.2";
const PROTECTED_MARKER_ENV: &str = "YAML_SIGIL_TERMINAL_CANDIDATE";
const PROTECTED_AUDIT_ENV: &str = "YAML_SIGIL_CARGO_AUDIT";
const PROTECTED_SEED_ENV: &str = "YAML_SIGIL_CARGO_SEED";
const PROTECTED_STATE_ENV: &str = "YAML_SIGIL_CARGO_STATE_ROOT";

#[derive(Clone, Debug, Eq, PartialEq)]
enum ExecutionBoundary {
    Ordinary,
    Protected { cargo_audit: PathBuf },
}

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

    fn command(self, boundary: &ExecutionBoundary) -> Command {
        if self.is_dependency_audit() {
            if let ExecutionBoundary::Protected { cargo_audit } = boundary {
                let mut command = Command::new(cargo_audit);
                command.args(["audit", "--no-fetch"]);
                return command;
            }
        }
        let mut command = Command::new(self.program);
        command.args(self.args);
        command
    }

    fn is_dependency_audit(self) -> bool {
        self.program == "cargo" && self.args == ["audit"]
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
    let boundary = execution_boundary()?;
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
            step.command(&boundary)
        };
        if step.program == "buf" {
            command.args(step.args);
        }
        let status = isolated_process::status(command.current_dir(current_dir))
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

fn execution_boundary() -> io::Result<ExecutionBoundary> {
    let marker = env::var_os(PROTECTED_MARKER_ENV);
    let audit = env::var_os(PROTECTED_AUDIT_ENV);
    let seed = env::var_os(PROTECTED_SEED_ENV);
    let state = env::var_os(PROTECTED_STATE_ENV);
    if marker.is_none() && audit.is_none() && seed.is_none() && state.is_none() {
        return Ok(ExecutionBoundary::Ordinary);
    }
    let Some(audit) = audit.filter(|value| !value.is_empty()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "protected dependency-audit boundary is incomplete",
        ));
    };
    if marker.as_deref() != Some(std::ffi::OsStr::new("1"))
        || seed.as_ref().is_none_or(|value| value.is_empty())
        || state.as_ref().is_none_or(|value| value.is_empty())
        || env::var_os("CARGO_NET_OFFLINE").as_deref() != Some(std::ffi::OsStr::new("true"))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "protected dependency-audit boundary is incomplete",
        ));
    }
    let cargo_audit = PathBuf::from(audit);
    if !cargo_audit.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "protected cargo-audit path is not absolute",
        ));
    }
    Ok(ExecutionBoundary::Protected { cargo_audit })
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

    #[test]
    fn ordinary_audit_can_seed_a_clean_cargo_home() {
        let audit = CI_STEPS
            .iter()
            .find(|step| step.label == "Rust dependency audit")
            .expect("dependency audit step is present");
        let clean = tempfile::tempdir().unwrap();
        let mut command = audit.command(&ExecutionBoundary::Ordinary);
        command.env("CARGO_HOME", clean.path());
        assert_eq!(command.get_program(), "cargo");
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["audit"]);
        assert!(!command.get_args().any(|argument| argument == "--no-fetch"));
    }

    #[test]
    fn protected_audit_uses_the_authenticated_binary_without_network() {
        let audit = CI_STEPS
            .iter()
            .copied()
            .find(|step| step.is_dependency_audit())
            .unwrap();
        let command = audit.command(&ExecutionBoundary::Protected {
            cargo_audit: PathBuf::from("/trusted-tools/bin/cargo-audit"),
        });
        assert_eq!(command.get_program(), "/trusted-tools/bin/cargo-audit");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["audit", "--no-fetch"]
        );
    }
}
