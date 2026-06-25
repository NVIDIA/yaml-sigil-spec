// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Small shared helpers: file I/O wrappers plus hex encode / decode.
//!
//! The hex helpers implement the standard 16-character alphabet
//! `0-9a-f` (lower-case). This is the de-facto convention shared by
//! every spec the rebuilder consumes — RFC 8032 §7.1 publishes its
//! test vectors in lower-case hex, FIPS 186-5 CAVP / ACVP vector
//! files use lower-case hex, and SEC 2 v2.0 prints curve parameters
//! in lower-case hex. No fancier format is involved.

use std::fs;
use std::io;
use std::path::Path;

/// Write `content` to `<dir>/<name>` (overwriting) and emit a one-line
/// progress log to stdout.
pub fn write_bytes(dir: &Path, name: &str, content: &[u8]) -> io::Result<()> {
    let path = dir.join(name);
    fs::write(&path, content)?;
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
