// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Linux-only subprocess containment for protected candidate validation.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicU64, Ordering};

const CARGO_SEED_ENV: &str = "YAML_SIGIL_CARGO_SEED";
const CARGO_STATE_ROOT_ENV: &str = "YAML_SIGIL_CARGO_STATE_ROOT";
static CARGO_STATE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct FreshCargoState {
    phase: PathBuf,
}

impl FreshCargoState {
    fn from_environment(command: &mut Command) -> io::Result<Option<Self>> {
        let seed = std::env::var_os(CARGO_SEED_ENV);
        let state_root = std::env::var_os(CARGO_STATE_ROOT_ENV);
        match (seed, state_root) {
            (None, None) => Ok(None),
            (Some(seed), Some(state_root)) => {
                Self::prepare(command, Path::new(&seed), Path::new(&state_root)).map(Some)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "incomplete protected Cargo state boundary",
            )),
        }
    }

    fn prepare(command: &mut Command, seed: &Path, state_root: &Path) -> io::Result<Self> {
        let seed = seed.canonicalize()?;
        let state_root = state_root.canonicalize()?;
        if !seed.is_dir() || !state_root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "protected Cargo seed and state root must be directories",
            ));
        }

        let sequence = CARGO_STATE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let phase = state_root.join(format!("rust-phase-{}-{sequence}", std::process::id()));
        std::fs::create_dir(&phase)?;
        let cargo_home = phase.join("cargo-home");
        let target = phase.join("target");
        std::fs::create_dir(&cargo_home)?;
        std::fs::create_dir(&target)?;
        link_seed_entries(&seed, &cargo_home)?;

        let mut inherited = std::env::vars_os()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        inherited.extend(command.get_envs().map(|(name, _)| name.to_os_string()));
        for name in inherited {
            let text = name.to_string_lossy();
            if text.starts_with("CARGO_ALIAS_") || text.starts_with("CARGO_TARGET_") {
                command.env_remove(name);
            }
        }
        for name in [
            "CARGO_BUILD_RUSTC",
            "CARGO_BUILD_RUSTC_WRAPPER",
            "CARGO_BUILD_RUSTDOC",
            "CARGO_BUILD_TARGET",
            "CARGO_ENCODED_RUSTFLAGS",
            "RUSTC",
            "RUSTC_WRAPPER",
            "RUSTC_WORKSPACE_WRAPPER",
            "RUSTDOC",
            "RUSTDOCFLAGS",
        ] {
            command.env_remove(name);
        }
        command
            .env("CARGO_HOME", cargo_home)
            .env("CARGO_TARGET_DIR", target)
            .env("CARGO_NET_OFFLINE", "true");
        Ok(Self { phase })
    }

    fn cleanup(self) -> io::Result<()> {
        std::fs::remove_dir_all(&self.phase).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "remove disposable Cargo state {}: {error}",
                    self.phase.display()
                ),
            )
        })
    }
}

#[cfg(unix)]
fn link_seed_entries(seed: &Path, cargo_home: &Path) -> io::Result<()> {
    for name in ["registry", "git", "advisory-db"] {
        let entry = seed.join(name);
        if entry.try_exists()? {
            let metadata = std::fs::symlink_metadata(&entry)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("protected Cargo seed {name} is not a direct directory"),
                ));
            }
            std::os::unix::fs::symlink(&entry, cargo_home.join(name))?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn link_seed_entries(_seed: &Path, _cargo_home: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "protected Cargo state isolation is Linux-only",
    ))
}

fn combine<T>(result: io::Result<T>, cleanup: io::Result<()>, label: &str) -> io::Result<T> {
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(cleanup_error)) => Err(cleanup_error),
        (Err(error), Err(cleanup_error)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; {label} cleanup failed: {cleanup_error}"),
        )),
    }
}

pub(crate) fn status(command: &mut Command) -> io::Result<ExitStatus> {
    let state = FreshCargoState::from_environment(command)?;
    let result = platform::status(command);
    let cleanup = match state {
        Some(state) => state.cleanup(),
        None => Ok(()),
    };
    combine(result, cleanup, "Cargo state")
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    use std::collections::BTreeSet;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use std::thread;
    use std::time::{Duration, Instant};

    use rustix::process::{
        kill_process, kill_process_group, set_child_subreaper, waitpid, Pid, Signal, WaitOptions,
    };

    const CLEANUP_TIMEOUT: Duration = Duration::from_secs(10);
    const POLL_INTERVAL: Duration = Duration::from_millis(10);
    static SUBREAPER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct Scope {
        baseline: BTreeSet<i32>,
        was_enabled: bool,
        _lock: MutexGuard<'static, ()>,
    }

    impl Scope {
        fn enter() -> io::Result<Self> {
            let lock = SUBREAPER_LOCK
                .get_or_init(|| Mutex::new(()))
                .lock()
                .map_err(|_| io::Error::other("candidate-process subreaper lock was poisoned"))?;
            let was_enabled = rustix::process::child_subreaper()?.is_some();
            if !was_enabled {
                set_child_subreaper(Pid::from_raw(1))?;
            }
            Ok(Self {
                baseline: direct_children()?,
                was_enabled,
                _lock: lock,
            })
        }

        fn terminate_and_reap(&self) -> io::Result<()> {
            let deadline = Instant::now() + CLEANUP_TIMEOUT;
            loop {
                let adopted = direct_children()?
                    .difference(&self.baseline)
                    .copied()
                    .collect::<Vec<_>>();
                if adopted.is_empty() {
                    return Ok(());
                }
                for raw in &adopted {
                    if let Some(pid) = Pid::from_raw(*raw) {
                        let _ = kill_process(pid, Signal::KILL);
                    }
                }
                for raw in adopted {
                    if let Some(pid) = Pid::from_raw(raw) {
                        match waitpid(Some(pid), WaitOptions::NOHANG) {
                            Ok(_) | Err(rustix::io::Errno::CHILD) => {}
                            Err(error) => return Err(error.into()),
                        }
                    }
                }
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "candidate descendants were not quiescent between validation steps",
                    ));
                }
                thread::sleep(POLL_INTERVAL);
            }
        }
    }

    impl Drop for Scope {
        fn drop(&mut self) {
            if !self.was_enabled {
                let _ = set_child_subreaper(None);
            }
        }
    }

    fn direct_children() -> io::Result<BTreeSet<i32>> {
        let own_pid = i32::try_from(std::process::id())
            .map_err(|_| io::Error::other("current process ID is out of range"))?;
        let mut children = BTreeSet::new();
        for entry in std::fs::read_dir("/proc")? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Ok(pid) = name.parse::<i32>() else {
                continue;
            };
            let stat = match std::fs::read_to_string(entry.path().join("stat")) {
                Ok(value) => value,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            let (_, fields) = stat.rsplit_once(") ").ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "malformed /proc process stat")
            })?;
            let parent = fields
                .split_whitespace()
                .nth(1)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "truncated /proc process stat")
                })?
                .parse::<i32>()
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid /proc parent process ID",
                    )
                })?;
            if parent == own_pid {
                children.insert(pid);
            }
        }
        Ok(children)
    }

    pub(super) fn status(command: &mut Command) -> io::Result<ExitStatus> {
        let scope = Scope::enter()?;
        command
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let result = (|| {
            let mut child = command.spawn()?;
            let group = Pid::from_child(&child);
            let status = child.wait();
            let _ = kill_process_group(group, Signal::KILL);
            status
        })();
        combine(result, scope.terminate_and_reap(), "descendant")
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::*;

    pub(super) fn status(_command: &mut Command) -> io::Result<ExitStatus> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "protected candidate execution is Linux-only",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn fresh_state_discards_alias_wrapper_and_target_poison() {
        let seed = tempfile::tempdir().unwrap();
        for name in ["registry", "git", "advisory-db"] {
            std::fs::create_dir(seed.path().join(name)).unwrap();
        }
        let root = tempfile::tempdir().unwrap();

        let mut first = Command::new("/bin/sh");
        first
            .arg("-c")
            .arg("mkdir -p \"$CARGO_HOME/bin\"; printf poison > \"$CARGO_HOME/config.toml\"; printf poison > \"$CARGO_HOME/bin/cargo-audit\"; printf poison > \"$CARGO_TARGET_DIR/forged\"")
            .env("RUSTC_WRAPPER", "/candidate/wrapper")
            .env("CARGO_ALIAS_AUDIT", "version");
        let first_state = FreshCargoState::prepare(&mut first, seed.path(), root.path()).unwrap();
        let status = platform::status(&mut first).unwrap();
        assert!(status.success());
        first_state.cleanup().unwrap();

        let marker = root.path().join("clean");
        let mut second = Command::new("/bin/sh");
        second.arg("-c").arg(format!(
            "test ! -e \"$CARGO_HOME/config.toml\" && test ! -e \"$CARGO_HOME/bin/cargo-audit\" && test ! -e \"$CARGO_TARGET_DIR/forged\" && test -z \"${{RUSTC_WRAPPER-}}\" && test -z \"${{CARGO_ALIAS_AUDIT-}}\" && printf clean > {}",
            marker.display()
        ));
        let second_state = FreshCargoState::prepare(&mut second, seed.path(), root.path()).unwrap();
        let status = platform::status(&mut second).unwrap();
        assert!(status.success());
        second_state.cleanup().unwrap();
        assert_eq!(std::fs::read(marker).unwrap(), b"clean");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn silent_session_escape_is_terminated_before_return() {
        let temporary = tempfile::tempdir().unwrap();
        let marker = temporary.path().join("escapee.pid");
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(format!(
            "setsid /bin/sh -c 'printf %s \"$$\" > {0}; exec sleep 60' </dev/null >/dev/null 2>/dev/null & while test ! -s {0}; do sleep 0.01; done",
            marker.display(),
        ));
        let status = platform::status(&mut command).unwrap();
        assert!(status.success());
        let pid = std::fs::read_to_string(marker)
            .unwrap()
            .parse::<i32>()
            .unwrap();
        assert!(!Path::new("/proc").join(pid.to_string()).exists());
    }
}
