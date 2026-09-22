// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Clone, Debug)]
pub(crate) struct Invocation {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) cwd: PathBuf,
}

impl Invocation {
    pub(crate) fn new(program: impl AsRef<OsStr>, args: &[&str], cwd: &Path) -> Self {
        Self {
            program: program.as_ref().into(),
            args: args.iter().map(OsString::from).collect(),
            cwd: cwd.into(),
        }
    }

    pub(crate) fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args).current_dir(&self.cwd);
        command
    }

    pub(crate) fn display(&self) -> String {
        std::iter::once(self.program.as_os_str())
            .chain(self.args.iter().map(OsString::as_os_str))
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(crate) trait Runner {
    fn output(&mut self, command: &Invocation) -> io::Result<Output>;
    fn run(&mut self, command: &Invocation) -> io::Result<()>;
}

pub(crate) struct SystemRunner;

impl Runner for SystemRunner {
    fn output(&mut self, command: &Invocation) -> io::Result<Output> {
        command.command().output()
    }

    fn run(&mut self, command: &Invocation) -> io::Result<()> {
        eprintln!("+ {} (cwd {})", command.display(), command.cwd.display());
        let status = command.command().status().map_err(|error| {
            io::Error::new(error.kind(), format!("{}: {error}", command.display()))
        })?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "{} failed with {status}",
                command.display()
            )))
        }
    }
}

pub(crate) fn probe(
    runner: &mut impl Runner,
    command: &Invocation,
    guidance: &str,
) -> io::Result<Output> {
    let output = runner.output(command).map_err(|error| {
        let description = if error.kind() == io::ErrorKind::NotFound {
            "was not found".to_owned()
        } else {
            format!("is installed but could not launch: {error}")
        };
        io::Error::new(
            error.kind(),
            format!(
                "{} {description}.\n{guidance}",
                command.program.to_string_lossy()
            ),
        )
    })?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{} is installed but unusable ({}): {}\n{guidance}",
            command.program.to_string_lossy(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    #[derive(Default)]
    pub(crate) struct FakeRunner {
        pub(crate) probes: Vec<Invocation>,
        pub(crate) commands: Vec<Invocation>,
        pub(crate) outputs: VecDeque<io::Result<Output>>,
        pub(crate) fail_run: Option<usize>,
    }

    impl Runner for FakeRunner {
        fn output(&mut self, command: &Invocation) -> io::Result<Output> {
            self.probes.push(command.clone());
            self.outputs
                .pop_front()
                .unwrap_or_else(|| Ok(output(0, "")))
        }
        fn run(&mut self, command: &Invocation) -> io::Result<()> {
            self.commands.push(command.clone());
            if self.fail_run == Some(self.commands.len()) {
                Err(io::Error::other("simulated failure"))
            } else {
                Ok(())
            }
        }
    }

    pub(crate) fn output(code: i32, stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(code << 8),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn missing_and_unusable_tools_have_distinct_actionable_errors() {
        let invocation = Invocation::new("cargo-tarpaulin", &["--version"], Path::new("."));
        let guidance = "cargo install --locked cargo-tarpaulin";
        let mut runner = FakeRunner::default();
        runner
            .outputs
            .push_back(Err(io::Error::from(io::ErrorKind::NotFound)));
        let error = probe(&mut runner, &invocation, guidance)
            .unwrap_err()
            .to_string();
        assert!(error.contains("was not found") && error.contains(guidance));
        runner
            .outputs
            .push_back(Ok(output(1, "missing runtime library")));
        let error = probe(&mut runner, &invocation, guidance)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("installed but unusable")
                && error.contains("missing runtime library")
                && error.contains(guidance)
        );
    }
}
