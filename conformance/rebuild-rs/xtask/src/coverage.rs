// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::cli::{CoverageArgs, CoverageEngine};
use crate::process::{probe, Invocation, Runner};

fn plan(workspace: &Path, args: &CoverageArgs) -> (Invocation, PathBuf) {
    let (cargo_args, directory, report): (&[&str], &str, &str) = match args.engine {
        CoverageEngine::LlvmCov => (
            &[
                "llvm-cov",
                "--locked",
                "--workspace",
                "--html",
                "--output-dir",
            ],
            "target/coverage/llvm-cov",
            "html/index.html",
        ),
        CoverageEngine::Tarpaulin => (
            &[
                "tarpaulin",
                "--locked",
                "--workspace",
                "--out",
                "Html",
                "--output-dir",
            ],
            "target/coverage/tarpaulin",
            "tarpaulin-report.html",
        ),
    };
    let directory = workspace.join(directory);
    let mut command = Invocation::new("cargo", cargo_args, workspace);
    command.args.push(directory.as_os_str().into());
    if args.engine == CoverageEngine::Tarpaulin {
        command.args.push("--target-dir".into());
        command.args.push(directory.join("build").into_os_string());
    }
    command.args.extend(args.features.args());
    (command, directory.join(report))
}

pub(crate) fn run(
    workspace: &Path,
    args: &CoverageArgs,
    open: bool,
    runner: &mut impl Runner,
) -> io::Result<()> {
    let (program, cargo_subcommand, guidance) = match args.engine {
        CoverageEngine::LlvmCov => (
            "cargo-llvm-cov",
            "llvm-cov",
            "cargo install --locked cargo-llvm-cov",
        ),
        CoverageEngine::Tarpaulin => (
            "cargo-tarpaulin",
            "tarpaulin",
            "cargo install --locked cargo-tarpaulin",
        ),
    };
    probe(
        runner,
        &Invocation::new(program, &[cargo_subcommand, "--version"], workspace),
        guidance,
    )?;
    if args.engine == CoverageEngine::LlvmCov {
        let components = probe(
            runner,
            &Invocation::new("rustup", &["component", "list", "--installed"], workspace),
            "rustup component add llvm-tools-preview",
        )?;
        if !String::from_utf8_lossy(&components.stdout)
            .lines()
            .any(|line| line.starts_with("llvm-tools"))
        {
            return Err(io::Error::other("LLVM tools are required.\nInstall them with: rustup component add llvm-tools-preview"));
        }
    }
    let (command, report) = plan(workspace, args);
    match fs::remove_file(&report) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    runner.run(&command)?;
    if !report.is_file() {
        return Err(io::Error::other(format!(
            "coverage completed without creating {}",
            report.display()
        )));
    }
    println!("coverage report: {}", report.display());
    if open {
        open_report(workspace, &report, runner)?;
    }
    Ok(())
}

fn open_report(workspace: &Path, report: &Path, runner: &mut impl Runner) -> io::Result<()> {
    match probe(
        runner,
        &Invocation::new("xdg-open", &["--version"], workspace),
        "Open the report path in your browser.",
    ) {
        Ok(_) => {
            let mut command = Invocation::new("xdg-open", &[], workspace);
            command.args.push(report.as_os_str().into());
            runner.run(&command)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            eprintln!(
                "No browser opener was found; open {} manually.",
                report.display()
            );
            Ok(())
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Features;
    use crate::process::tests::{output, FakeRunner};
    use std::process::Output;

    struct ReportRunner {
        inner: FakeRunner,
        report: PathBuf,
    }

    impl Runner for ReportRunner {
        fn output(&mut self, command: &Invocation) -> io::Result<Output> {
            let mut result = self.inner.output(command)?;
            if command.program == "rustup" {
                result.stdout = b"llvm-tools-x86_64-unknown-linux-gnu\n".to_vec();
            }
            Ok(result)
        }
        fn run(&mut self, command: &Invocation) -> io::Result<()> {
            if command.program == "cargo" {
                assert!(
                    !self.report.exists(),
                    "old report must be removed before generation"
                );
                fs::create_dir_all(self.report.parent().unwrap())?;
                fs::write(&self.report, "fresh report")?;
            } else {
                assert_eq!(fs::read_to_string(&self.report)?, "fresh report");
            }
            self.inner.run(command)
        }
    }

    #[test]
    fn every_opening_form_generates_before_opening() {
        use clap::Parser;
        for arguments in [vec!["coverage", "--open"], vec!["coverage-open"]] {
            for engine in ["llvm-cov", "tarpaulin"] {
                let root = crate::tests::test_dir("coverage");
                let cli = crate::Cli::try_parse_from(
                    std::iter::once("xtask")
                        .chain(arguments.iter().copied())
                        .chain(["--engine", engine]),
                )
                .unwrap();
                let (crate::cli::Task::Coverage(args) | crate::cli::Task::CoverageOpen(args)) =
                    cli.task
                else {
                    unreachable!()
                };
                let (_, report) = plan(&root, &args);
                fs::create_dir_all(report.parent().unwrap()).unwrap();
                fs::write(&report, "stale report").unwrap();
                let mut runner = ReportRunner {
                    inner: FakeRunner::default(),
                    report,
                };
                run(&root, &args, true, &mut runner).unwrap();
                assert_eq!(runner.inner.commands.len(), 2);
                assert_eq!(runner.inner.commands[0].program, "cargo");
                assert_eq!(runner.inner.probes[0].args[0], engine);
                assert!(runner.inner.commands[0].args.contains(&"--locked".into()));
                assert!(runner.inner.commands[0]
                    .args
                    .contains(&"--workspace".into()));
                assert!(runner.inner.commands[0]
                    .args
                    .contains(&"--all-features".into()));
                if engine == "tarpaulin" {
                    let build_dir = root.join("target/coverage/tarpaulin/build");
                    assert!(runner.inner.commands[0]
                        .args
                        .contains(&build_dir.into_os_string()));
                }
                assert_eq!(runner.inner.commands[1].program, "xdg-open");
                fs::remove_dir_all(root).unwrap();
            }
        }
    }

    #[test]
    fn generation_must_create_a_new_report() {
        let root = crate::tests::test_dir("coverage-missing");
        let args = CoverageArgs {
            engine: CoverageEngine::Tarpaulin,
            open: true,
            features: Features::default(),
        };
        let mut runner = FakeRunner::default();
        assert!(run(&root, &args, true, &mut runner)
            .unwrap_err()
            .to_string()
            .contains("without creating"));
        assert_eq!(runner.commands.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn llvm_requires_its_rustup_component_without_installing_it() {
        let args = CoverageArgs {
            engine: CoverageEngine::LlvmCov,
            open: false,
            features: Features::default(),
        };
        let mut runner = FakeRunner::default();
        runner.outputs.push_back(Ok(output(0, "")));
        runner.outputs.push_back(Ok(output(0, "")));
        let error = run(Path::new("."), &args, false, &mut runner).unwrap_err();
        assert!(error
            .to_string()
            .contains("rustup component add llvm-tools-preview"));
        assert!(runner.commands.is_empty());
    }
}
