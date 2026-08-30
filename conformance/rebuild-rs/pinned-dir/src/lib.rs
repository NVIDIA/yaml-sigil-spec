// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Pinned-directory file operations for the conformance developer tools.
//!
//! Every child operation is resolved relative to an already-open directory
//! handle. This keeps the operation attached to that directory even if
//! another process renames or replaces its original pathname.

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

#[cfg(test)]
use std::fs;

/// An open directory used as the authority for child operations.
#[derive(Debug)]
pub struct PinnedDir {
    handle: Dir,
}

impl PinnedDir {
    /// Open and pin an existing absolute directory path without following
    /// symlinks in any component.
    pub fn open(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("directory path is not absolute: {}", path.display()),
            ));
        }

        let (root, components) = absolute_path_components(path)?;
        let mut current = Self {
            handle: Dir::open_ambient_dir(root, ambient_authority())?,
        };
        for component in components {
            current = Self {
                handle: current.handle.open_dir_nofollow(component)?,
            };
        }
        Ok(current)
    }

    /// Open and pin an existing child directory without following a final
    /// symlink.
    pub fn open_child(&self, name: &str) -> io::Result<Self> {
        validate_name(name)?;
        Ok(Self {
            handle: self.handle.open_dir_nofollow(name)?,
        })
    }

    /// Open a child directory, creating it if absent.
    pub fn ensure_child(&self, name: &str) -> io::Result<Self> {
        validate_name(name)?;
        match self.symlink_metadata(name) {
            Ok(metadata) if metadata.file_type().is_dir() => self.open_child(name),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("refusing non-directory or symlink path: {name}"),
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.handle.create_dir(name)?;
                self.open_child(name)
            }
            Err(error) => Err(error),
        }
    }

    /// Query a child entry without following a final symlink.
    pub fn symlink_metadata(&self, name: &str) -> io::Result<cap_std::fs::Metadata> {
        validate_name(name)?;
        self.handle.symlink_metadata(name)
    }

    /// Read one regular child file through the pinned directory handle.
    ///
    /// The file is opened without following a final symlink. Both its opened
    /// metadata and a limit-plus-one read enforce `max_bytes`, so a concurrent
    /// size change cannot turn the caller's bound into an unbounded read.
    pub fn read_regular_file_bounded(&self, name: &str, max_bytes: usize) -> io::Result<Vec<u8>> {
        validate_name(name)?;
        let sentinel = max_bytes.checked_add(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "file bound has no sentinel")
        })?;
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        let mut file = self.handle.open_with(name, &options)?;
        let metadata = file.metadata()?;
        if !metadata.file_type().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("refusing non-regular input: {name}"),
            ));
        }
        if metadata.len() > max_bytes as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("input exceeds {max_bytes}-byte limit: {name}"),
            ));
        }

        let mut content = Vec::with_capacity(metadata.len() as usize);
        (&mut file)
            .take(sentinel as u64)
            .read_to_end(&mut content)?;
        if content.len() > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("input exceeds {max_bytes}-byte limit while reading: {name}"),
            ));
        }
        Ok(content)
    }

    /// Replace a regular child file without following a child or parent
    /// symlink.
    pub fn replace_regular_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        validate_name(name)?;
        match self.symlink_metadata(name) {
            Ok(metadata) if metadata.file_type().is_file() => {
                self.handle.remove_file(name)?;
            }
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("refusing to replace non-regular path: {name}"),
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No);
        let mut file = self.handle.open_with(name, &options)?;
        file.write_all(content)
    }
}

fn absolute_path_components(path: &Path) -> io::Result<(PathBuf, Vec<OsString>)> {
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "directory path is not a normalized absolute path: {}",
                path.display()
            ),
        )
    };
    let mut root = PathBuf::new();
    let mut names = Vec::new();
    let mut saw_root = false;
    for component in path.components() {
        match component {
            Component::Prefix(prefix) if root.as_os_str().is_empty() && !saw_root => {
                root.push(prefix.as_os_str());
            }
            Component::RootDir if !saw_root => {
                root.push(std::path::MAIN_SEPARATOR_STR);
                saw_root = true;
            }
            Component::Normal(name) if saw_root => names.push(name.to_os_string()),
            _ => return Err(invalid()),
        }
    }
    if !saw_root {
        return Err(invalid());
    }
    Ok((root, names))
}

fn validate_name(name: &str) -> io::Result<()> {
    let mut components = Path::new(name).components();
    if matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    ) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path is not a single child name: {name}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let sequence = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "yamlsigil-pinned-dir-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        path
    }

    #[test]
    fn rejects_nested_child_name() {
        let root = test_dir("nested-name");
        let pinned = PinnedDir::open(&root).expect("pin root");

        let error = pinned
            .replace_regular_file("../outside", b"content")
            .expect_err("nested name must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        drop(pinned);
        fs::remove_dir(root).expect("remove test root");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_destination() {
        use std::os::unix::fs::symlink;

        let root = test_dir("destination-symlink");
        let target = root.join("target");
        fs::write(&target, b"keep").expect("write target");
        symlink(&target, root.join("destination")).expect("create symlink");
        let pinned = PinnedDir::open(&root).expect("pin root");

        let error = pinned
            .replace_regular_file("destination", b"replace")
            .expect_err("destination symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&target).expect("read target"), b"keep");
        drop(pinned);
        fs::remove_file(root.join("destination")).expect("remove symlink");
        fs::remove_file(target).expect("remove target");
        fs::remove_dir(root).expect("remove test root");
    }

    #[test]
    fn bounded_read_accepts_exact_limit_and_rejects_limit_plus_one() {
        let root = test_dir("bounded-read");
        fs::write(root.join("exact"), b"1234").expect("write exact input");
        fs::write(root.join("large"), b"12345").expect("write large input");
        let pinned = PinnedDir::open(&root).expect("pin root");

        assert_eq!(
            pinned
                .read_regular_file_bounded("exact", 4)
                .expect("read exact-sized input"),
            b"1234"
        );
        let error = pinned
            .read_regular_file_bounded("large", 4)
            .expect_err("limit-plus-one input must fail");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);

        drop(pinned);
        fs::remove_file(root.join("exact")).expect("remove exact input");
        fs::remove_file(root.join("large")).expect("remove large input");
        fs::remove_dir(root).expect("remove test root");
    }

    #[cfg(unix)]
    #[test]
    fn bounded_read_rejects_symlink_source() {
        use std::os::unix::fs::symlink;

        let root = test_dir("bounded-read-symlink");
        fs::write(root.join("target"), b"content").expect("write target");
        symlink(root.join("target"), root.join("source")).expect("create source symlink");
        let pinned = PinnedDir::open(&root).expect("pin root");

        pinned
            .read_regular_file_bounded("source", 16)
            .expect_err("source symlink must fail");

        drop(pinned);
        fs::remove_file(root.join("source")).expect("remove source symlink");
        fs::remove_file(root.join("target")).expect("remove target");
        fs::remove_dir(root).expect("remove test root");
    }

    #[cfg(unix)]
    #[test]
    fn initial_open_rejects_raced_parent_symlink() {
        use std::os::unix::fs::symlink;

        let root = test_dir("initial-parent-swap");
        let parent_path = root.join("parent");
        let moved_path = root.join("moved");
        let outside_path = root.join("outside");
        let output_path = parent_path.join("output");
        fs::create_dir(&parent_path).expect("create parent");
        fs::create_dir(&output_path).expect("create output");
        fs::create_dir(&outside_path).expect("create outside");
        fs::create_dir(outside_path.join("output")).expect("create outside output");
        let resolved_output = output_path.canonicalize().expect("resolve output");

        fs::rename(&parent_path, &moved_path).expect("move parent");
        symlink(&outside_path, &parent_path).expect("replace parent with symlink");

        PinnedDir::open(&resolved_output).expect_err("raced parent symlink must fail");

        fs::remove_file(parent_path).expect("remove replacement symlink");
        fs::remove_dir(moved_path.join("output")).expect("remove moved output");
        fs::remove_dir(moved_path).expect("remove moved parent");
        fs::remove_dir(outside_path.join("output")).expect("remove outside output");
        fs::remove_dir(outside_path).expect("remove outside");
        fs::remove_dir(root).expect("remove test root");
    }

    #[cfg(windows)]
    #[test]
    fn initial_open_rejects_intermediate_junction() {
        use std::process::Command;

        let root = test_dir("initial-parent-junction");
        let target = root.join("target");
        let junction = root.join("junction");
        fs::create_dir(&target).expect("create junction target");
        fs::create_dir(target.join("child")).expect("create target child");
        let output = Command::new("cmd.exe")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&target)
            .output()
            .expect("run mklink");
        assert!(
            output.status.success(),
            "mklink failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        PinnedDir::open(&junction.join("child"))
            .expect_err("intermediate junction must fail closed");

        fs::remove_dir(&junction).expect("remove junction");
        fs::remove_dir(target.join("child")).expect("remove target child");
        fs::remove_dir(target).expect("remove target");
        fs::remove_dir(root).expect("remove test root");
    }

    #[cfg(unix)]
    #[test]
    fn parent_path_swap_does_not_redirect_write() {
        use std::os::unix::fs::symlink;

        let root = test_dir("parent-swap");
        let output_path = root.join("output");
        let moved_path = root.join("moved");
        let outside_path = root.join("outside");
        fs::create_dir(&output_path).expect("create output");
        fs::create_dir(&outside_path).expect("create outside");
        let root_dir = PinnedDir::open(&root).expect("pin root");
        let output_dir = root_dir.open_child("output").expect("pin output");

        fs::rename(&output_path, &moved_path).expect("move pinned output");
        symlink(&outside_path, &output_path).expect("replace output with symlink");
        output_dir
            .replace_regular_file("fixture", b"pinned")
            .expect("write through pinned directory");

        assert_eq!(
            fs::read(moved_path.join("fixture")).expect("read pinned output"),
            b"pinned"
        );
        assert!(!outside_path.join("fixture").exists());

        drop(output_dir);
        drop(root_dir);
        fs::remove_file(output_path).expect("remove replacement symlink");
        fs::remove_file(moved_path.join("fixture")).expect("remove fixture");
        fs::remove_dir(moved_path).expect("remove moved directory");
        fs::remove_dir(outside_path).expect("remove outside directory");
        fs::remove_dir(root).expect("remove test root");
    }
}
