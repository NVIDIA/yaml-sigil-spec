// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Local entry point for the repository's non-release validation sequence.

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUF_VERSION_REQUIREMENT: &str = ">=1.73.0";
const BUF_INSTALL_GUIDANCE: &str = "Install a supported buf-toolchain release with:\n    \
     cargo install --locked --force --version '>=1.73.0-rc.3' buf-toolchain\n\n\
     Then ensure $CARGO_HOME/bin is on PATH.\n\
     See https://buf.build/docs/cli/installation/ for official alternatives.";
const CARGO_DENY_INSTALL_COMMAND: &str = "cargo install --locked cargo-deny --version 0.20.2";
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

    fn command(self) -> Command {
        let mut command = Command::new(self.program);
        command.args(self.args);
        command
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
        label: "Rust dependency policy",
        program: "cargo-deny",
        args: &[
            "--locked",
            "--workspace",
            "check",
            "bans",
            "licenses",
            "sources",
            "-D",
            "warnings",
        ],
        working_directory: WorkingDirectory::RebuildWorkspace,
    },
    Step {
        label: "Rust dependency audit",
        program: "cargo",
        args: &["audit"],
        working_directory: WorkingDirectory::RebuildWorkspace,
    },
];

pub(crate) fn run(repository_root: &Path, rebuild_root: &Path) -> io::Result<()> {
    require_cargo_machete()?;
    require_cargo_deny()?;
    let buf = resolve_buf()?;

    for step in CI_STEPS {
        let current_dir = match step.working_directory {
            WorkingDirectory::Repository => repository_root,
            WorkingDirectory::RebuildWorkspace => rebuild_root,
        };
        eprintln!("+ {} (cwd {})", step.command_line(), current_dir.display());
        let mut command = if step.program == "buf" {
            Command::new(&buf)
        } else {
            step.command()
        };
        if step.program == "buf" {
            command.args(step.args);
        }
        let status = command
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
    require_cargo_tool("cargo-machete", CARGO_MACHETE_INSTALL_COMMAND)
}

fn require_cargo_deny() -> io::Result<()> {
    require_cargo_tool("cargo-deny", CARGO_DENY_INSTALL_COMMAND)
}

fn require_cargo_tool(program: &str, install_command: &str) -> io::Result<()> {
    let output = Command::new(program)
        .arg("--version")
        .output()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "{program} is required but was not found.\n\n\
                         Install it with:\n    {install_command}"
                    ),
                )
            } else {
                io::Error::new(error.kind(), format!("failed to run {program}: {error}"))
            }
        })?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{program} --version failed with {}.\n\n{install_command}",
            output.status
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

    validate_buf_version(&output.stdout)?;

    Ok(buf)
}

fn validate_buf_version(stdout: &[u8]) -> io::Result<()> {
    let text = std::str::from_utf8(stdout)
        .map_err(|error| buf_prerequisite_error(format!("Buf version is not UTF-8: {error}")))?;
    let version = semver::Version::parse(text.trim()).map_err(|error| {
        buf_prerequisite_error(format!(
            "Invalid Buf CLI version {:?}: {error}",
            text.trim()
        ))
    })?;
    let requirement = semver::VersionReq::parse(BUF_VERSION_REQUIREMENT)
        .expect("Buf CLI requirement is valid semver");
    if !requirement.matches(&version) {
        return Err(buf_prerequisite_error(format!(
            "Buf CLI {version} does not satisfy {BUF_VERSION_REQUIREMENT}."
        )));
    }
    Ok(())
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
            5
        );
    }

    #[test]
    fn buf_version_accepts_the_minimum_and_newer_releases() {
        for version in ["1.73.0\n", "1.73.1", "1.74.0", "2.0.0", "1.73.0+build.1"] {
            validate_buf_version(version.as_bytes()).unwrap();
        }
    }

    #[test]
    fn unsupported_buf_versions_report_install_guidance() {
        for version in [
            b"1.72.0".as_slice(),
            b"1.73.0-rc.3",
            b"1.74.0-rc.1",
            b"",
            b"not-a-version",
            b"\xff",
        ] {
            let error = validate_buf_version(version).unwrap_err().to_string();
            assert!(error.contains(BUF_INSTALL_GUIDANCE), "{error}");
        }
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
    fn cargo_deny_guidance_is_aligned_and_actionable() {
        assert_eq!(
            CARGO_DENY_INSTALL_COMMAND,
            "cargo install --locked cargo-deny --version 0.20.2"
        );
        assert!(AGENT_GUIDANCE.contains(CARGO_DENY_INSTALL_COMMAND));
        assert!(AGENT_GUIDANCE
            .contains("cargo-deny --locked --workspace check bans licenses sources -D warnings"));
    }
}
