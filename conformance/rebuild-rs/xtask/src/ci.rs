// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Provider-independent repository check planning and execution.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

use crate::cli::{CheckStep, Features};
use crate::process::{probe, Invocation, Runner};

const BUF_VERSION_REQUIREMENT: &str = ">=1.73.0";
const BUF_INSTALL_GUIDANCE: &str = "Install a supported buf-toolchain release with:\n    \
     cargo install --locked --force --version '>=1.73.0' buf-toolchain\n\n\
     Then ensure $CARGO_HOME/bin is on PATH.\n\
     See https://buf.build/docs/cli/installation/ for official alternatives.";
const CARGO_DENY_INSTALL_COMMAND: &str = "cargo install --locked cargo-deny --version 0.20.2";
const CARGO_MACHETE_INSTALL_COMMAND: &str = "cargo install --locked cargo-machete --version 0.9.2";

fn invocation(
    step: CheckStep,
    repository: &Path,
    workspace: &Path,
    features: &Features,
) -> Invocation {
    use crate::cli::CheckStep::*;
    let (program, args, cwd): (&str, &[&str], &Path) = match step {
        Markdown => ("rumdl", &["check", "."], repository),
        BufBuild => ("buf", &["build", "proto"], repository),
        BufLint => ("buf", &["lint", "proto"], repository),
        BufFmt => (
            "buf",
            &["format", "proto", "--diff", "--exit-code"],
            repository,
        ),
        Schema => (
            "jq",
            &["empty", "schema/YamlSigilSignature.v1alpha1.schema.json"],
            repository,
        ),
        Fmt => ("cargo", &["fmt", "--all", "--check"], workspace),
        Check => (
            "cargo",
            &["check", "--locked", "--workspace", "--all-targets"],
            workspace,
        ),
        Clippy => (
            "cargo",
            &["clippy", "--locked", "--workspace", "--all-targets"],
            workspace,
        ),
        Test => ("cargo", &["test", "--locked", "--workspace"], workspace),
        // Invoke cargo-machete directly: inherited Cargo package variables in
        // 0.9.2 make `cargo machete` parse its subcommand as an input path.
        Machete => ("cargo-machete", &["--with-metadata"], repository),
        // Dependency policy covers every feature, even when compilation is
        // narrowed by the caller. Keep this aligned with deny.toml.
        Deny => (
            "cargo-deny",
            &["--locked", "--workspace", "--all-features"],
            workspace,
        ),
        Audit => ("cargo", &["audit"], workspace),
    };
    let mut command = Invocation::new(program, args, cwd);
    if matches!(step, Check | Clippy | Test) {
        command.args.extend(features.args());
    }
    if step == Clippy {
        command
            .args
            .extend(["--", "-D", "warnings"].map(Into::into));
    }
    if step == Deny {
        command
            .args
            .extend(["check", "bans", "licenses", "sources", "-D", "warnings"].map(Into::into));
    }
    command
}

pub(crate) fn run(
    repository: &Path,
    workspace: &Path,
    selected: &[CheckStep],
    features: &Features,
    runner: &mut impl Runner,
) -> io::Result<()> {
    let mut buf = None;
    for step in selected {
        let mut command = invocation(*step, repository, workspace, features);
        match step {
            CheckStep::BufBuild | CheckStep::BufLint | CheckStep::BufFmt => {
                if buf.is_none() {
                    buf = Some(resolve_buf(repository, runner)?);
                }
                command.program = buf.as_ref().expect("resolved Buf").as_os_str().into();
            }
            _ => preflight(*step, &command.cwd, runner)?,
        }
        runner.run(&command)?;
    }
    Ok(())
}

fn preflight(step: CheckStep, cwd: &Path, runner: &mut impl Runner) -> io::Result<()> {
    let (program, args, guidance): (&str, &[&str], &str) = match step {
        CheckStep::Markdown => ("rumdl", &["--version"], "cargo install rumdl"),
        CheckStep::Schema => (
            "jq",
            &["--version"],
            "Install jq: https://jqlang.org/download/",
        ),
        CheckStep::Fmt => (
            "cargo",
            &["fmt", "--version"],
            "rustup component add rustfmt",
        ),
        CheckStep::Clippy => (
            "cargo",
            &["clippy", "--version"],
            "rustup component add clippy",
        ),
        CheckStep::Check | CheckStep::Test => {
            ("cargo", &["--version"], "Install Rust: https://rustup.rs/")
        }
        CheckStep::Machete => (
            "cargo-machete",
            &["--version"],
            CARGO_MACHETE_INSTALL_COMMAND,
        ),
        CheckStep::Deny => ("cargo-deny", &["--version"], CARGO_DENY_INSTALL_COMMAND),
        CheckStep::Audit => (
            "cargo-audit",
            &["--version"],
            "cargo +1.98.0 install --locked cargo-audit --version 0.22.2",
        ),
        CheckStep::BufBuild | CheckStep::BufLint | CheckStep::BufFmt => return Ok(()),
    };
    probe(runner, &Invocation::new(program, args, cwd), guidance)?;
    Ok(())
}

fn resolve_buf(cwd: &Path, runner: &mut impl Runner) -> io::Result<PathBuf> {
    let buf = env::var_os("BUF")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("buf"));
    let output = probe(
        runner,
        &Invocation::new(&buf, &["--version"], cwd),
        BUF_INSTALL_GUIDANCE,
    )?;
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
    use crate::process::tests::FakeRunner;

    const AGENT_GUIDANCE: &str = include_str!("../../../../AGENTS.md");

    #[test]
    fn agent_guidance_documents_every_default_check() {
        for step in CheckStep::ORDER {
            let command = invocation(
                step,
                Path::new("repo"),
                Path::new("workspace"),
                &Features::default(),
            );
            assert!(
                AGENT_GUIDANCE.contains(&command.display()),
                "missing {}",
                command.display()
            );
        }
    }

    #[test]
    fn formatting_needs_no_unselected_tool_or_features() {
        let mut runner = FakeRunner::default();
        run(
            Path::new("repo"),
            Path::new("workspace"),
            &[CheckStep::Fmt],
            &Features::default(),
            &mut runner,
        )
        .unwrap();
        assert_eq!(runner.probes.len(), 1);
        assert_eq!(runner.probes[0].display(), "cargo fmt --version");
        assert_eq!(runner.commands[0].display(), "cargo fmt --all --check");
        assert_eq!(runner.commands[0].cwd, Path::new("workspace"));
    }

    #[test]
    fn checks_preserve_target_flags_and_stop_after_failure() {
        let mut runner = FakeRunner {
            fail_run: Some(1),
            ..FakeRunner::default()
        };
        assert!(run(
            Path::new("repo"),
            Path::new("workspace"),
            &[CheckStep::Check, CheckStep::Test],
            &Features::default(),
            &mut runner
        )
        .is_err());
        assert_eq!(runner.commands.len(), 1);
        assert_eq!(
            runner.commands[0].display(),
            "cargo check --locked --workspace --all-targets --all-features"
        );
        let clippy = invocation(
            CheckStep::Clippy,
            Path::new("repo"),
            Path::new("workspace"),
            &Features::default(),
        );
        assert_eq!(
            clippy.display(),
            "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings"
        );
    }

    #[test]
    fn explicit_features_select_compilation_but_preserve_full_dependency_policy() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "xtask",
            "check",
            "--features=a,b",
            "--no-default-features",
        ])
        .unwrap();
        let crate::cli::Task::Check(args) = cli.task else {
            unreachable!()
        };
        for step in [CheckStep::Check, CheckStep::Clippy, CheckStep::Test] {
            let command = invocation(
                step,
                Path::new("repo"),
                Path::new("workspace"),
                &args.features,
            );
            assert!(!command.args.contains(&"--all-features".into()));
            assert!(command
                .display()
                .contains("--features a,b --no-default-features"));
        }
        let deny = invocation(
            CheckStep::Deny,
            Path::new("repo"),
            Path::new("workspace"),
            &args.features,
        );
        assert_eq!(
            deny.display(),
            "cargo-deny --locked --workspace --all-features check bans licenses sources -D warnings"
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
    fn dependency_install_guidance_matches_agent_instructions() {
        assert!(AGENT_GUIDANCE.contains(CARGO_MACHETE_INSTALL_COMMAND));
        assert!(AGENT_GUIDANCE.contains(CARGO_DENY_INSTALL_COMMAND));
    }
}
