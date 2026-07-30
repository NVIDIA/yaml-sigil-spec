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

use std::io;
use yamlsigil_pinned_dir::PinnedDir;

/// Replace a regular fixture through an already-pinned directory handle and
/// emit a one-line progress log to stdout.
pub fn write_bytes(dir: &PinnedDir, name: &str, content: &[u8]) -> io::Result<()> {
    dir.replace_regular_file(name, content)?;
    println!("  {}: {} bytes", name, content.len());
    Ok(())
}

/// Same as [`write_bytes`] but accepts a UTF-8 `&str`.
pub fn write_text(dir: &PinnedDir, name: &str, content: &str) -> io::Result<()> {
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
    use std::fs;
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
        let pinned = PinnedDir::open(&dir).expect("pin fixture directory");

        write_bytes(&pinned, "fixture.txt", b"new").expect("replace fixture");

        assert_eq!(fs::read(&path).expect("read fixture"), b"new");
        drop(pinned);
        fs::remove_file(path).expect("remove fixture");
        fs::remove_dir(dir).expect("remove test directory");
    }

    #[test]
    fn write_bytes_rejects_nested_fixture_name() {
        let dir = test_dir("reject-nested-name");
        let pinned = PinnedDir::open(&dir).expect("pin fixture directory");

        let error = write_bytes(&pinned, "../outside.txt", b"content")
            .expect_err("nested fixture name must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        drop(pinned);
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
        let pinned = PinnedDir::open(&output_dir).expect("pin output directory");

        let error = write_bytes(&pinned, "fixture.txt", b"replace")
            .expect_err("destination symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&target).expect("read target"), b"keep");
        assert!(fs::symlink_metadata(&destination)
            .expect("inspect destination")
            .file_type()
            .is_symlink());
        drop(pinned);
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

        let error = PinnedDir::open(&linked_dir).expect_err("directory symlink must fail");

        assert!(error.raw_os_error().is_some());
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
