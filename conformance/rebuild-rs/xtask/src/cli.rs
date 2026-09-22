// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Command-line interface for repository development and maintenance.
#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "Validate and maintain the specification repository"
)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) task: Task,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Task {
    /// Run the repository's non-release checks in canonical order.
    #[command(visible_alias = "ci")]
    Check(CheckArgs),
    /// Read the bounded ACVP corpus before executing candidate code.
    CandidatePreflight {
        #[arg(long, value_parser = nonempty_path)]
        candidate_root: PathBuf,
    },
    /// Refresh the pinned ACVP corpus and its provenance README.
    UpdateAcvp {
        #[arg(long, default_value = super::DEFAULT_COMMIT, value_parser = commit)]
        commit: String,
    },
    /// Generate an HTML coverage report for the locked Rust workspace.
    Coverage(CoverageArgs),
    /// Generate and open a fresh HTML coverage report.
    CoverageOpen(CoverageArgs),
    /// Build the local fixture-rebuild image and verify its offline output.
    Image(ImageArgs),
}

fn commit(value: &str) -> Result<String, String> {
    super::validate_commit(value)?;
    Ok(value.to_owned())
}

fn nonempty_path(value: &str) -> Result<PathBuf, String> {
    if value.is_empty() {
        Err("path must not be empty".to_owned())
    } else {
        Ok(PathBuf::from(value))
    }
}

#[derive(Clone, Debug, Default, Args)]
pub(crate) struct Features {
    /// Enable all features (the default when no feature option is supplied).
    #[arg(long, conflicts_with_all = ["features", "no_default_features"])]
    all_features: bool,
    /// Enable these Cargo features; may be combined with --no-default-features.
    #[arg(long, value_delimiter = ',', value_parser = clap::builder::NonEmptyStringValueParser::new())]
    features: Vec<String>,
    /// Disable default features.
    #[arg(long)]
    no_default_features: bool,
}

impl Features {
    pub(crate) fn args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        if self.all_features || (self.features.is_empty() && !self.no_default_features) {
            args.push("--all-features".into());
        }
        if !self.features.is_empty() {
            args.extend(["--features".into(), self.features.join(",").into()]);
        }
        if self.no_default_features {
            args.push("--no-default-features".into());
        }
        args
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum CheckStep {
    Markdown,
    BufBuild,
    BufLint,
    BufFmt,
    Schema,
    Fmt,
    Check,
    Clippy,
    Test,
    Machete,
    Deny,
    Audit,
}

impl CheckStep {
    pub(crate) const ORDER: [Self; 12] = [
        Self::Markdown,
        Self::BufBuild,
        Self::BufLint,
        Self::BufFmt,
        Self::Schema,
        Self::Fmt,
        Self::Check,
        Self::Clippy,
        Self::Test,
        Self::Machete,
        Self::Deny,
        Self::Audit,
    ];
}

#[derive(Debug, Default, Args)]
pub(crate) struct CheckArgs {
    /// Run only these checks, in registry order.
    #[arg(long, value_delimiter = ',', conflicts_with = "exclude")]
    only: Vec<CheckStep>,
    /// Run every check except these.
    #[arg(long, value_delimiter = ',')]
    exclude: Vec<CheckStep>,
    /// Validate this checkout using the current checkout's xtask.
    #[arg(long, value_parser = nonempty_path)]
    pub(crate) candidate_root: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) features: Features,
}

impl CheckArgs {
    pub(crate) fn selected(&self) -> io::Result<Vec<CheckStep>> {
        let selected: Vec<_> = CheckStep::ORDER
            .into_iter()
            .filter(|step| self.only.is_empty() || self.only.contains(step))
            .filter(|step| !self.exclude.contains(step))
            .collect();
        if selected.is_empty() {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "select at least one check",
            ))
        } else {
            Ok(selected)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum CoverageEngine {
    #[default]
    LlvmCov,
    Tarpaulin,
}

#[derive(Debug, Args)]
pub(crate) struct CoverageArgs {
    #[arg(long, value_enum, default_value_t = CoverageEngine::LlvmCov)]
    pub(crate) engine: CoverageEngine,
    #[arg(long)]
    pub(crate) open: bool,
    #[command(flatten)]
    pub(crate) features: Features,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum ImageEngine {
    #[default]
    Auto,
    Docker,
    Buildah,
}

#[derive(Debug, Args)]
pub(crate) struct ImageArgs {
    #[arg(long, value_enum, default_value_t = ImageEngine::Auto)]
    pub(crate) engine: ImageEngine,
    #[arg(long, default_value = "yamlsigil-conformance-rebuild-rs", value_parser = clap::builder::NonEmptyStringValueParser::new())]
    pub(crate) tag: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn parser_is_valid_and_alias_is_visible() {
        let mut command = Cli::command();
        command.clone().debug_assert();
        assert!(command.render_long_help().to_string().contains("ci"));
    }

    fn checks(args: &[&str]) -> CheckArgs {
        match Cli::try_parse_from(std::iter::once("xtask").chain(args.iter().copied()))
            .unwrap()
            .task
        {
            Task::Check(args) => args,
            _ => panic!("expected check"),
        }
    }

    #[test]
    fn check_and_ci_share_ordered_deduplicated_selection() {
        for command in ["check", "ci"] {
            let args = checks(&[command, "--only=test,fmt,test"]);
            assert_eq!(args.selected().unwrap(), [CheckStep::Fmt, CheckStep::Test]);
            assert_eq!(checks(&[command]).selected().unwrap(), CheckStep::ORDER);
        }
        assert!(!checks(&["check", "--exclude=fmt"])
            .selected()
            .unwrap()
            .contains(&CheckStep::Fmt));
    }

    #[test]
    fn invalid_and_empty_selections_fail() {
        for args in [
            vec!["xtask", "check", "--only="],
            vec!["xtask", "check", "--only=fmt,"],
            vec!["xtask", "check", "--only=unknown"],
            vec!["xtask", "check", "--only=fmt", "--exclude=test"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
        let names = CheckStep::value_variants()
            .iter()
            .map(|step| step.to_possible_value().unwrap().get_name().to_owned())
            .collect::<Vec<_>>()
            .join(",");
        assert!(checks(&["check", &format!("--exclude={names}")])
            .selected()
            .is_err());
    }

    #[test]
    fn features_follow_cargo_semantics() {
        assert_eq!(checks(&["check"]).features.args(), ["--all-features"]);
        assert_eq!(
            checks(&["check", "--features=one,two", "--no-default-features"])
                .features
                .args(),
            ["--features", "one,two", "--no-default-features"]
        );
        assert_eq!(
            checks(&["check", "--no-default-features"]).features.args(),
            ["--no-default-features"]
        );
        for args in [
            vec!["xtask", "check", "--all-features", "--features=one"],
            vec!["xtask", "check", "--all-features", "--no-default-features"],
            vec!["xtask", "check", "--features="],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
    }

    #[test]
    fn coverage_commands_share_feature_and_engine_options() {
        for name in ["coverage", "coverage-open"] {
            let cli = Cli::try_parse_from([
                "xtask",
                name,
                "--engine=tarpaulin",
                "--features=a,b",
                "--no-default-features",
            ])
            .unwrap();
            let (Task::Coverage(args) | Task::CoverageOpen(args)) = cli.task else {
                panic!("coverage expected")
            };
            assert_eq!(args.engine, CoverageEngine::Tarpaulin);
            assert_eq!(
                args.features.args(),
                ["--features", "a,b", "--no-default-features"]
            );
        }
    }
}
