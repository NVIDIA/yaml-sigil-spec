// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! URL-safe base64 encoding, unpadded.
//!
//! Implements the alphabet and padding rules from
//! [RFC 4648 §5](https://www.rfc-editor.org/rfc/rfc4648#section-5)
//! ("Base 64 Encoding with URL and Filename Safe Alphabet"). The exact
//! alphabet, terminator handling, and "no padding" decision used by
//! this module are taken verbatim from that section:
//!
//! > This encoding may be referred to as "base64url".  This encoding
//! > should not be regarded as the same as the "base64" encoding and
//! > should not be referred to as only "base64".  Unless clarified
//! > otherwise, "base64" refers to the base 64 in the previous section.
//! >
//! > This encoding is technically identical to the previous one, except
//! > for the 62:nd and 63:rd alphabet character, as indicated in Table 2.
//! >
//! > The pad character "=" is typically percent-encoded when used in an
//! > URI \[9\], but if the data length is known implicitly, this can be
//! > avoided by skipping the padding; see section 3.2.
//!
//! Table 2 (excerpted) gives the alphabet entries that differ from §4:
//! index 62 maps to `-` and index 63 maps to `_`. The remaining entries
//! match the §4 table (`A`..`Z`, `a`..`z`, `0`..`9`). The implementation
//! below embeds this exact 64-character alphabet as a static byte string
//! and elides any `=` padding, matching the "padding can be avoided"
//! clause above. These are the exact alphabet and padding rules used by
//! every signature and key encoding in the rebuilder.
//!
//! The cited alphabet, rules, and test values are third-party RFC material,
//! not material relicensed under the Apache-2.0 declaration on this
//! NVIDIA-authored implementation. See the repository
//! `THIRD_PARTY_NOTICES.md` for the applicable source attribution and terms.

/// 64-character base64url alphabet from RFC 4648 §5 Table 2.
///
/// Indices 0..61 are `A`..`Z`, `a`..`z`, `0`..`9` (matching §4 Table 1);
/// indices 62 and 63 are `-` and `_` respectively (the URL/filename-safe
/// substitutions specified in §5).
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode `input` as URL-safe base64 with no `=` padding.
///
/// Per RFC 4648 §5, padding MAY be omitted when the data length is known
/// implicitly. Every consumer in this crate carries the length out of
/// band (signature blobs are fixed at 64 octets; the empty case is
/// length-zero), so this encoder never emits `=`.
pub fn urlsafe_unpadded(input: &[u8]) -> String {
    let mut out = Vec::with_capacity(input.len().div_ceil(3) * 4);
    let mut chunks = input.chunks_exact(3);
    for c in chunks.by_ref() {
        let n = (u32::from(c[0]) << 16) | (u32::from(c[1]) << 8) | u32::from(c[2]);
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize]);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize]);
        out.push(ALPHABET[((n >> 6) & 0x3F) as usize]);
        out.push(ALPHABET[(n & 0x3F) as usize]);
    }
    let rem = chunks.remainder();
    match rem.len() {
        0 => {}
        1 => {
            let n = u32::from(rem[0]) << 16;
            out.push(ALPHABET[((n >> 18) & 0x3F) as usize]);
            out.push(ALPHABET[((n >> 12) & 0x3F) as usize]);
        }
        2 => {
            let n = (u32::from(rem[0]) << 16) | (u32::from(rem[1]) << 8);
            out.push(ALPHABET[((n >> 18) & 0x3F) as usize]);
            out.push(ALPHABET[((n >> 12) & 0x3F) as usize]);
            out.push(ALPHABET[((n >> 6) & 0x3F) as usize]);
        }
        _ => unreachable!(),
    }
    String::from_utf8(out).expect("alphabet is ASCII")
}

/// Canonical encoding of 64 zero octets — the placeholder signature
/// used by every YAML-form fixture that does NOT exercise a real
/// signature. 86 characters of `A` (zero is alphabet index 0).
pub fn placeholder_sig() -> String {
    urlsafe_unpadded(&[0u8; 64])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-answer vectors from RFC 4648 §10 (Test Vectors), re-encoded
    // under the §5 alphabet. The §10 table is given in the standard
    // (§4) alphabet; the §5 alphabet differs only at indices 62/63
    // (`+` -> `-`, `/` -> `_`), so the published vectors that don't use
    // those indices pass through unchanged. Inputs are ASCII bytes.
    #[test]
    fn rfc4648_section_10_vectors_pass_through() {
        // Empty input encodes to empty output.
        assert_eq!(urlsafe_unpadded(b""), "");
        // "f" -> "Zg" (§10 says "Zg==", padding stripped).
        assert_eq!(urlsafe_unpadded(b"f"), "Zg");
        // "fo" -> "Zm8" (§10: "Zm8=").
        assert_eq!(urlsafe_unpadded(b"fo"), "Zm8");
        // "foo" -> "Zm9v" (§10: "Zm9v").
        assert_eq!(urlsafe_unpadded(b"foo"), "Zm9v");
        // "foob" -> "Zm9vYg" (§10: "Zm9vYg==").
        assert_eq!(urlsafe_unpadded(b"foob"), "Zm9vYg");
        // "fooba" -> "Zm9vYmE" (§10: "Zm9vYmE=").
        assert_eq!(urlsafe_unpadded(b"fooba"), "Zm9vYmE");
        // "foobar" -> "Zm9vYmFy" (§10: "Zm9vYmFy").
        assert_eq!(urlsafe_unpadded(b"foobar"), "Zm9vYmFy");
    }

    /// 0xFB encodes to `+` under §4 and `-` under §5; 0xFF to `/` under
    /// §4 and `_` under §5. Exercise both substitutions explicitly.
    #[test]
    fn urlsafe_substitutions_at_indices_62_and_63() {
        // 0xFB FF -> index 62 then 63 in the first two output chars.
        // Bytes 0xFB 0xFF 0xFF: 11111011 11111111 11111111
        //   -> 111110 111111 111111 111111
        //   -> indices 62, 63, 63, 63 -> "-___"
        assert_eq!(urlsafe_unpadded(&[0xFB, 0xFF, 0xFF]), "-___");
    }

    #[test]
    fn placeholder_sig_is_86_a_characters() {
        let s = placeholder_sig();
        assert_eq!(s.len(), 86);
        assert!(s.chars().all(|c| c == 'A'));
    }
}
