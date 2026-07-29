// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Small shared helpers: file I/O wrappers plus hex encode / decode.
//!
//! The hex helpers implement the standard 16-character alphabet
//! `0-9a-f` (lower-case). This is the de-facto convention shared by
//! every spec the rebuilder consumes — RFC 8032 §7.1 publishes its
//! test vectors in lower-case hex, FIPS 186-5 CAVP / ACVP vector
//! files use lower-case hex, and *Standards for Efficient Cryptography 2
//! (SEC 2)* Version 2.0 prints curve parameters in lower-case hex. No
//! fancier format is involved.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path};

/// Replace the regular file at `<dir>/<name>` without following symlinks and
/// emit a one-line progress log to stdout.
///
/// Rust documents [`OpenOptions::create_new`] as an atomic existence check:
///
/// > No file is allowed to exist at the target location, also no (dangling)
/// > symlink.
///
/// The helper removes an existing regular fixture first. If any entry appears
/// before creation, `create_new` fails instead of opening that entry for output.
///
/// [`OpenOptions::create_new`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new
pub fn write_bytes(dir: &Path, name: &str, content: &[u8]) -> io::Result<()> {
    let mut components = Path::new(name).components();
    if !matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    ) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("fixture name is not a single path component: {name}"),
        ));
    }

    let dir_metadata = fs::symlink_metadata(dir)?;
    if !dir_metadata.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "fixture directory is not a non-symlink directory: {}",
                dir.display()
            ),
        ));
    }

    let path = dir.join(name);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            fs::remove_file(&path)?;
        }
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "refusing to replace non-regular fixture path: {}",
                    path.display()
                ),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    file.write_all(content)?;
    println!("  {}: {} bytes", name, content.len());
    Ok(())
}

/// Same as [`write_bytes`] but accepts a UTF-8 `&str`.
pub fn write_text(dir: &Path, name: &str, content: &str) -> io::Result<()> {
    write_bytes(dir, name, content.as_bytes())
}

/// Encode `bytes` as lower-case hex (two characters per octet, no
/// separator), matching the format used by every upstream cited in
/// this crate.
pub fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Decode a hex string. Whitespace is stripped first; the cleaned
/// length MUST be even. Panics on malformed input — every caller in
/// this crate passes a compile-time hex literal taken verbatim from
/// the upstream spec.
pub fn from_hex(s: &str) -> Vec<u8> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        cleaned.len().is_multiple_of(2),
        "odd-length hex: {}",
        cleaned.len()
    );
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).expect("invalid hex"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let sequence = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "yamlsigil-rebuild-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        path
    }

    #[test]
    fn write_bytes_replaces_regular_file() {
        let dir = test_dir("replace-regular");
        let path = dir.join("fixture.txt");
        fs::write(&path, b"old").expect("write original fixture");

        write_bytes(&dir, "fixture.txt", b"new").expect("replace fixture");

        assert_eq!(fs::read(&path).expect("read fixture"), b"new");
        fs::remove_file(path).expect("remove fixture");
        fs::remove_dir(dir).expect("remove test directory");
    }

    #[test]
    fn write_bytes_rejects_nested_fixture_name() {
        let dir = test_dir("reject-nested-name");

        let error = write_bytes(&dir, "../outside.txt", b"content")
            .expect_err("nested fixture name must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        fs::remove_dir(dir).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn write_bytes_rejects_symlink_destination() {
        use std::os::unix::fs::symlink;

        let root = test_dir("reject-destination-symlink");
        let output_dir = root.join("output");
        fs::create_dir(&output_dir).expect("create output directory");
        let target = root.join("outside.txt");
        fs::write(&target, b"keep").expect("write symlink target");
        let destination = output_dir.join("fixture.txt");
        symlink(&target, &destination).expect("create destination symlink");

        let error = write_bytes(&output_dir, "fixture.txt", b"replace")
            .expect_err("destination symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&target).expect("read target"), b"keep");
        assert!(fs::symlink_metadata(&destination)
            .expect("inspect destination")
            .file_type()
            .is_symlink());
        fs::remove_file(destination).expect("remove destination symlink");
        fs::remove_file(target).expect("remove target");
        fs::remove_dir(output_dir).expect("remove output directory");
        fs::remove_dir(root).expect("remove test root");
    }

    #[cfg(unix)]
    #[test]
    fn write_bytes_rejects_symlink_directory() {
        use std::os::unix::fs::symlink;

        let root = test_dir("reject-directory-symlink");
        let real_dir = root.join("real");
        fs::create_dir(&real_dir).expect("create real directory");
        let linked_dir = root.join("linked");
        symlink(&real_dir, &linked_dir).expect("create directory symlink");

        let error = write_bytes(&linked_dir, "fixture.txt", b"content")
            .expect_err("directory symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!real_dir.join("fixture.txt").exists());
        fs::remove_file(linked_dir).expect("remove directory symlink");
        fs::remove_dir(real_dir).expect("remove real directory");
        fs::remove_dir(root).expect("remove test root");
    }

    #[test]
    fn hex_round_trip() {
        let cases: &[&[u8]] = &[b"", b"\x00", b"\xff", b"\x00\x10\x80\xff"];
        for c in cases {
            let s = hex_lower(c);
            assert_eq!(from_hex(&s), *c, "round trip for {c:?}");
        }
    }

    #[test]
    fn hex_lower_pads_each_byte_to_two_chars() {
        assert_eq!(hex_lower(&[0x00]), "00");
        assert_eq!(hex_lower(&[0x0f]), "0f");
        assert_eq!(hex_lower(&[0xab, 0xcd]), "abcd");
    }

    #[test]
    fn from_hex_strips_whitespace() {
        assert_eq!(from_hex("de ad\nbe ef"), vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    #[should_panic(expected = "odd-length hex")]
    fn from_hex_rejects_odd_length() {
        from_hex("abc");
    }
}
