// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/base64/` fixtures.
//!
//! Drives the strict YamlSigil base64 profile (URL-safe alphabet,
//! unpadded, no whitespace, no non-zero trailing bits) against the
//! published RFC 4648 rules. The relevant excerpts from
//! [RFC 4648 §3](https://www.rfc-editor.org/rfc/rfc4648#section-3)
//! ("Implementation Discrepancies") and §5 are:
//!
//! > Some specifications include additional characters in their
//! > base alphabets... Implementations MUST reject the encoded data
//! > if it contains characters outside the base alphabet when
//! > interpreting base-encoded data, unless the specification
//! > referring to this document explicitly states otherwise.
//!
//! > The pad character "=" is used to signal the end of a base-encoded
//! > stream when the input data is not an integral multiple of three
//! > octets. It SHOULD be encoded as the equivalent of "%3D" when
//! > used in URIs... When fewer than 24 input bits are available in
//! > the final group, bits with no corresponding output bits are set
//! > to zero on encoding and discarded on decoding.
//!
//! ## Non-zero trailing bits — RFC 4648 §3.5
//!
//! [RFC 4648 §3.5](https://www.rfc-editor.org/rfc/rfc4648#section-3.5)
//! ("Canonical Encoding") goes further:
//!
//! > The padding step in base 64 and base 32 encoding can, if
//! > improperly implemented, lead to non-significant alterations of
//! > the encoded data. For example, if the input is only one octet
//! > for a base 64 encoding, then all six bits of the first symbol
//! > are used, but only the first two bits of the next symbol are
//! > used. These pad bits MUST be set to zero by conforming encoders,
//! > which is described in the descriptions on padding below. If
//! > this property does not hold, there is no canonical
//! > representation of base-encoded data, and multiple base-encoded
//! > strings can be decoded to the same binary data.
//!
//! The `nonzero-trailing-bits.txt` fixture pins a single-bit
//! perturbation of the final symbol: the canonical encoding of 64
//! zero octets has its last character as `A` (six zero bits); the
//! fixture flips it to `B` (`000001`), which decodes to the same
//! binary but violates §3.5's canonical-encoding rule. These are the
//! exact rules the [`crate::b64`] encoder satisfies on output and that
//! the strict-decoder spec enforces on input.
//!
//! The cited rules and test values are third-party RFC material, not material
//! relicensed under the Apache-2.0 declaration on this NVIDIA-authored
//! generator. See the repository `THIRD_PARTY_NOTICES.md` for the applicable
//! source attribution and terms. The raw parser-input fixtures cannot carry
//! comments without changing the values under test.

use std::path::Path;

use crate::b64::urlsafe_unpadded;
use crate::util::write_bytes;

pub fn generate(dir: &Path) -> std::io::Result<()> {
    // 86-char URL-safe unpadded base64 of 64 zero bytes (all 'A')
    let valid = urlsafe_unpadded(&[0u8; 64]).into_bytes();
    assert_eq!(valid.len(), 86);
    assert!(valid.iter().all(|&b| b == b'A'));
    write_bytes(dir, "valid-64-octet.txt", &valid)?;

    // Empty (NOT a base64 decode failure; empty string is the
    // encoding of zero bytes; content-layer rule is downstream)
    write_bytes(dir, "empty.txt", &[])?;

    // Invalid alphabet: `+` (standard, not URL-safe)
    let mut plus = vec![b'+'];
    plus.extend_from_slice(&valid[1..]);
    write_bytes(dir, "invalid-alphabet-plus.txt", &plus)?;

    // Invalid alphabet: `/`
    let mut slash = vec![b'/'];
    slash.extend_from_slice(&valid[1..]);
    write_bytes(dir, "invalid-alphabet-slash.txt", &slash)?;

    // Padding present
    let mut padded = valid.clone();
    padded.extend_from_slice(b"==");
    write_bytes(dir, "padding-present.txt", &padded)?;

    // Length mod 4 == 1
    write_bytes(dir, "length-mod-4-eq-1.txt", &valid[..valid.len() - 1])?;

    // Internal whitespace
    let mid = valid.len() / 2;
    let mut whitespace = Vec::with_capacity(valid.len() + 1);
    whitespace.extend_from_slice(&valid[..mid]);
    whitespace.push(b' ');
    whitespace.extend_from_slice(&valid[mid..]);
    write_bytes(dir, "whitespace-internal.txt", &whitespace)?;

    // Non-zero trailing bits: flip last 'A' (0b000000) to 'B' (0b000001).
    // Per RFC 4648 §3.5 the strict decoder MUST reject; the binary
    // value decoded would be identical.
    let mut nz = valid[..valid.len() - 1].to_vec();
    nz.push(b'B');
    write_bytes(dir, "nonzero-trailing-bits.txt", &nz)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::b64::urlsafe_unpadded;

    /// Sanity-anchor: 64 zero octets MUST encode to 86 chars of `A`
    /// under the URL-safe unpadded profile. Every fixture in this
    /// module mutates that exact baseline.
    #[test]
    fn baseline_is_86_a_characters() {
        let v = urlsafe_unpadded(&[0u8; 64]);
        assert_eq!(v.len(), 86);
        assert!(v.chars().all(|c| c == 'A'));
    }
}
