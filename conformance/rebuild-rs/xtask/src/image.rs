// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli::{ImageArgs, ImageEngine};
use crate::process::{probe, Invocation, Runner};

const DOCKER_GUIDANCE: &str = "Set up Docker Engine: https://docs.docker.com/engine/install/";
const BUILDAH_GUIDANCE: &str =
    "Set up Buildah: https://github.com/containers/buildah/blob/main/install.md";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn engine_program(engine: ImageEngine) -> &'static str {
    match engine {
        ImageEngine::Docker => "docker",
        ImageEngine::Buildah => "buildah",
        ImageEngine::Auto => unreachable!("resolve the engine before building"),
    }
}

fn select_engine(
    engine: ImageEngine,
    root: &Path,
    runner: &mut impl Runner,
) -> io::Result<ImageEngine> {
    let candidates: &[ImageEngine] = match engine {
        ImageEngine::Auto => &[ImageEngine::Docker, ImageEngine::Buildah],
        ImageEngine::Docker => &[ImageEngine::Docker],
        ImageEngine::Buildah => &[ImageEngine::Buildah],
    };
    let mut errors = Vec::new();
    for candidate in candidates {
        let guidance = if *candidate == ImageEngine::Docker {
            DOCKER_GUIDANCE
        } else {
            BUILDAH_GUIDANCE
        };
        match probe(
            runner,
            &Invocation::new(engine_program(*candidate), &["info"], root),
            guidance,
        ) {
            Ok(_) => return Ok(*candidate),
            Err(error) => errors.push(error.to_string()),
        }
    }
    Err(io::Error::other(errors.join("\n\n")))
}

fn build_command(root: &Path, engine: ImageEngine, tag: &str) -> Invocation {
    let operation = if engine == ImageEngine::Docker {
        "build"
    } else {
        "bud"
    };
    let mut command = Invocation::new(engine_program(engine), &[operation, "--file"], root);
    command.args.push(
        root.join("conformance/rebuild-rs/Dockerfile")
            .into_os_string(),
    );
    command
        .args
        .extend(["--tag".into(), tag.into(), root.as_os_str().into()]);
    command
}

pub(crate) fn run(root: &Path, args: &ImageArgs, runner: &mut impl Runner) -> io::Result<()> {
    let engine = select_engine(args.engine, root, runner)?;
    runner.run(&build_command(root, engine, &args.tag))?;
    let source = root.join("conformance");
    let expected = fixture_files(&source)?;
    if expected.is_empty() {
        return Err(io::Error::other(
            "the conformance fixture inventory is empty",
        ));
    }
    let temporary = TemporaryDirectory::new()?;
    let work = temporary.path.join("fixtures");
    fs::create_dir(&work)?;
    fs::set_permissions(&work, fs::Permissions::from_mode(0o777))?;
    for suite in fixture_suites(&source)? {
        let path = work.join(suite);
        fs::create_dir(&path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o777))?;
    }
    let result = run_smoke(root, engine, &args.tag, &temporary.name, &work, runner)
        .and_then(|()| verify_fixtures(&expected, &work));
    let cleanup = temporary.remove();
    combine_cleanup(result, cleanup)?;
    println!(
        "image {} verified {} fixture files with {}",
        args.tag,
        expected.len(),
        engine_program(engine)
    );
    Ok(())
}

fn run_smoke(
    root: &Path,
    engine: ImageEngine,
    tag: &str,
    name: &str,
    work: &Path,
    runner: &mut impl Runner,
) -> io::Result<()> {
    let (create, run, cleanup) = match engine {
        ImageEngine::Docker => {
            let mount = format!("type=bind,source={},target=/work", work.display());
            (
                Invocation::new(
                    "docker",
                    &[
                        "create",
                        "--name",
                        name,
                        "--network",
                        "none",
                        "--mount",
                        &mount,
                        tag,
                    ],
                    root,
                ),
                Invocation::new("docker", &["start", "--attach", name], root),
                Invocation::new("docker", &["rm", "--force", name], root),
            )
        }
        ImageEngine::Buildah => {
            let mount = format!("{}:/work:rw", work.display());
            (
                Invocation::new("buildah", &["from", "--name", name, tag], root),
                Invocation::new(
                    "buildah",
                    &[
                        "run",
                        "--network=none",
                        "--volume",
                        &mount,
                        name,
                        "--",
                        "/usr/local/bin/rebuild_all",
                    ],
                    root,
                ),
                Invocation::new("buildah", &["rm", name], root),
            )
        }
        ImageEngine::Auto => unreachable!("engine resolved before smoke testing"),
    };
    let result = runner.run(&create).and_then(|()| runner.run(&run));
    combine_cleanup(result, runner.run(&cleanup))
}

fn combine_cleanup(result: io::Result<()>, cleanup: io::Result<()>) -> io::Result<()> {
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(io::Error::other(format!("cleanup failed: {error}"))),
        (Err(error), Err(cleanup)) => Err(io::Error::other(format!(
            "{error}; cleanup also failed: {cleanup}"
        ))),
    }
}

fn fixture_suites(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut suites = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_name() == "rebuild-rs" {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(io::Error::other("fixture suites must not be symlinks"));
        }
        if kind.is_dir() {
            suites.push(PathBuf::from(entry.file_name()));
        }
    }
    suites.sort();
    Ok(suites)
}

fn fixture_files(root: &Path) -> io::Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut fixtures = BTreeMap::new();
    for suite in fixture_suites(root)? {
        for entry in fs::read_dir(root.join(&suite))? {
            let entry = entry?;
            if entry.file_name() == "README.md" {
                continue;
            }
            if !entry.file_type()?.is_file() {
                return Err(io::Error::other(format!(
                    "fixture is not a regular file: {}",
                    entry.path().display()
                )));
            }
            fixtures.insert(suite.join(entry.file_name()), fs::read(entry.path())?);
        }
    }
    Ok(fixtures)
}

fn verify_fixtures(expected: &BTreeMap<PathBuf, Vec<u8>>, work: &Path) -> io::Result<()> {
    let actual = fixture_files(work)?;
    if *expected == actual {
        return Ok(());
    }
    let missing_or_changed: Vec<_> = expected
        .iter()
        .filter(|(path, bytes)| actual.get(*path) != Some(*bytes))
        .map(|(path, _)| path.display().to_string())
        .collect();
    let unexpected: Vec<_> = actual
        .keys()
        .filter(|path| !expected.contains_key(*path))
        .map(|path| path.display().to_string())
        .collect();
    Err(io::Error::other(format!(
        "image fixture regeneration differs; missing or changed: {}; unexpected: {}",
        missing_or_changed.join(", "),
        unexpected.join(", ")
    )))
}

struct TemporaryDirectory {
    path: PathBuf,
    name: String,
}

impl TemporaryDirectory {
    fn new() -> io::Result<Self> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let name = format!("yamlsigil-image-{}-{time}-{sequence}", std::process::id());
        let path = std::env::temp_dir().join(&name);
        fs::DirBuilder::new().mode(0o700).create(&path)?;
        Ok(Self { path, name })
    }

    fn remove(self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::tests::{output, FakeRunner};

    #[test]
    fn auto_prefers_usable_docker_then_buildah() {
        let mut runner = FakeRunner::default();
        assert_eq!(
            select_engine(ImageEngine::Auto, Path::new("."), &mut runner).unwrap(),
            ImageEngine::Docker
        );
        assert_eq!(runner.probes.len(), 1);
        let mut runner = FakeRunner::default();
        runner
            .outputs
            .push_back(Ok(output(1, "daemon unavailable")));
        assert_eq!(
            select_engine(ImageEngine::Auto, Path::new("."), &mut runner).unwrap(),
            ImageEngine::Buildah
        );
        assert_eq!(runner.probes.len(), 2);
        let mut runner = FakeRunner::default();
        runner
            .outputs
            .push_back(Ok(output(1, "daemon unavailable")));
        assert!(select_engine(ImageEngine::Docker, Path::new("."), &mut runner).is_err());
        assert_eq!(runner.probes.len(), 1);
    }

    #[test]
    fn a_build_failure_does_not_switch_engines() {
        let mut runner = FakeRunner {
            fail_run: Some(1),
            ..FakeRunner::default()
        };
        let args = ImageArgs {
            engine: ImageEngine::Auto,
            tag: "local:test".into(),
        };
        assert!(run(Path::new("."), &args, &mut runner).is_err());
        assert_eq!(runner.probes.len(), 1);
        assert_eq!(runner.commands.len(), 1);
    }

    #[test]
    fn smoke_containers_are_offline_and_always_removed() {
        for engine in [ImageEngine::Docker, ImageEngine::Buildah] {
            for failure in [None, Some(1), Some(2)] {
                let mut runner = FakeRunner {
                    fail_run: failure,
                    ..FakeRunner::default()
                };
                let result = run_smoke(
                    Path::new("."),
                    engine,
                    "local:test",
                    "test-container",
                    Path::new("/tmp/fixtures"),
                    &mut runner,
                );
                assert_eq!(result.is_err(), failure.is_some());
                assert!(runner.commands.last().unwrap().args.contains(&"rm".into()));
                assert!(!runner
                    .commands
                    .iter()
                    .any(|command| command.args.contains(&"push".into())));
                if failure != Some(1) {
                    assert!(runner
                        .commands
                        .iter()
                        .any(|command| command.display().contains("--network none")
                            || command.display().contains("--network=none")));
                }
            }
        }
    }

    #[test]
    fn fixture_verification_detects_missing_changed_and_extra_bytes() {
        let temp = TemporaryDirectory::new().unwrap();
        let suite = temp.path.join("suite");
        fs::create_dir(&suite).unwrap();
        fs::write(suite.join("case.bin"), [0, 1, 255]).unwrap();
        let expected = fixture_files(&temp.path).unwrap();
        verify_fixtures(&expected, &temp.path).unwrap();
        fs::write(suite.join("case.bin"), [0, 2, 255]).unwrap();
        assert!(verify_fixtures(&expected, &temp.path).is_err());
        fs::remove_file(suite.join("case.bin")).unwrap();
        assert!(verify_fixtures(&expected, &temp.path).is_err());
        fs::write(suite.join("case.bin"), [0, 1, 255]).unwrap();
        fs::write(suite.join("extra.bin"), []).unwrap();
        assert!(verify_fixtures(&expected, &temp.path).is_err());
        let path = temp.path.clone();
        temp.remove().unwrap();
        assert!(!path.exists());
    }
}
