// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Pinned-directory file operations for the conformance developer tools.
//!
//! Linux exposes an open directory through `/proc/self/fd/<fd>`. Resolving a
//! single child name through that handle keeps the operation attached to the
//! opened directory even if another process renames or replaces its original
//! pathname.

#[cfg(not(target_os = "linux"))]
compile_error!(
    "native conformance rebuilding requires Linux; use the repository's container workflow"
);

use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};

// Linux open(2) flags. The standard library supplies the access, creation,
// exclusive, and close-on-exec flags used by each OpenOptions call.
const O_DIRECTORY: i32 = 0o200000;
const O_NOFOLLOW: i32 = 0o400000;

/// An open directory used as the authority for child operations.
#[derive(Debug)]
pub struct PinnedDir {
    handle: File,
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

        let mut current = Self {
            handle: open_directory(Path::new("/"))?,
        };
        for component in path.components() {
            match component {
                Component::RootDir => {}
                Component::Normal(name) => {
                    current = Self {
                        handle: open_directory(&current.entry_path(name))?,
                    };
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "directory path is not a normalized absolute path: {}",
                            path.display()
                        ),
                    ));
                }
            }
        }
        Ok(current)
    }

    /// Open and pin an existing child directory without following a final
    /// symlink.
    pub fn open_child(&self, name: &str) -> io::Result<Self> {
        validate_name(name)?;
        Ok(Self {
            handle: open_directory(&self.entry_path(name))?,
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
                fs::create_dir(self.entry_path(name))?;
                self.open_child(name)
            }
            Err(error) => Err(error),
        }
    }

    /// Query a child entry without following a final symlink.
    pub fn symlink_metadata(&self, name: &str) -> io::Result<fs::Metadata> {
        validate_name(name)?;
        fs::symlink_metadata(self.entry_path(name))
    }

    /// Replace a regular child file without following a child or parent
    /// symlink.
    pub fn replace_regular_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        validate_name(name)?;
        match self.symlink_metadata(name) {
            Ok(metadata) if metadata.file_type().is_file() => {
                fs::remove_file(self.entry_path(name))?;
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

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(O_NOFOLLOW)
            .open(self.entry_path(name))?;
        file.write_all(content)
    }

    fn entry_path(&self, name: impl AsRef<OsStr>) -> PathBuf {
        PathBuf::from("/proc/self/fd")
            .join(self.handle.as_raw_fd().to_string())
            .join(name.as_ref())
    }
}

fn open_directory(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags(O_DIRECTORY | O_NOFOLLOW)
        .open(path)
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
