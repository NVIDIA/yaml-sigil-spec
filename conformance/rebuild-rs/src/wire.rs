// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Protobuf wire-format helpers + shared YAML / artifact builders.
//!
//! Hand-crafted so an auditor can verify the resulting bytes against
//! the published Protocol Buffers wire format without trusting a
//! library. All encoding rules below come from the canonical
//! [Protocol Buffers encoding spec](https://protobuf.dev/programming-guides/encoding/).
//!
//! ## Wire types and tag encoding (relevant excerpt)
//!
//! > Each field in a Protobuf message has a wire type. Each tag is
//! > built from a field number and a wire type. The wire type is
//! > stored in the bottom three bits of the tag. The remaining bits
//! > encode the field number.
//!
//! Wire types used by this crate:
//!
//! - `0` (VARINT) — for the `alg` field (a varint enum).
//! - `2` (LEN, "length-delimited") — for `payload`, `signature` (bytes),
//!   `keyid` (string), and the inner `YamlSigilSignature` submessage.
//!
//! ## Varint encoding (relevant excerpt)
//!
//! > Variable-width integers, or varints, are at the core of the wire
//! > format. They allow encoding unsigned 64-bit integers using anywhere
//! > between one and ten bytes, with small values using fewer bytes.
//! > Each byte in the varint has a continuation bit that indicates if
//! > the byte that follows it is part of the varint. This is the most
//! > significant bit (MSB) of the byte (sometimes also called the sign
//! > bit). The lower 7 bits are a payload; the resulting integer is
//! > built by appending together the 7-bit payloads of its constituent
//! > bytes.
//!
//! ## Length-delimited fields (relevant excerpt)
//!
//! > LEN-encoded records start with a VARINT-encoded length, followed
//! > by the specified amount of data.
//!
//! The three helpers below ([`varint`], [`tag`], [`lendel`]) are direct
//! implementations of those three rules; the message-shape helpers
//! ([`yss`], [`signed_yaml_artifact`]) compose them per the field
//! numbers declared in `proto/yaml_sigil/v1alpha1/yaml_sigil.proto`.
//! These are the exact encoding rules used by every protobuf fixture
//! in `conformance/`.

use crate::b64;

/// Encode `n` as a base-128 varint per the encoding-spec excerpt above.
///
/// Each output byte stores 7 bits of payload (little-endian groups) with
/// the high bit set on every byte except the last.
pub fn varint(mut n: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(10);
    while n > 0x7F {
        out.push(((n & 0x7F) as u8) | 0x80);
        n >>= 7;
    }
    out.push((n & 0x7F) as u8);
    out
}

/// Encode the `(field_number, wire_type)` tag byte sequence: a varint
/// containing `(field_number << 3) | wire_type`.
pub fn tag(field_number: u64, wire_type: u64) -> Vec<u8> {
    varint((field_number << 3) | wire_type)
}

/// Length-delimited field (wire type `2`).
///
/// Per the spec excerpt: tag (LEN), then varint-encoded length, then
/// the raw `data` octets.
pub fn lendel(field_number: u64, data: &[u8]) -> Vec<u8> {
    let mut out = tag(field_number, 2);
    out.extend(varint(data.len() as u64));
    out.extend_from_slice(data);
    out
}

/// Varint field (wire type `0`): tag (VARINT), then the varint payload.
pub fn varint_field(field_number: u64, value: u64) -> Vec<u8> {
    let mut out = tag(field_number, 0);
    out.extend(varint(value));
    out
}

/// `YamlSigilSignature` inner message.
///
/// Fields (from `proto/yaml_sigil/v1alpha1/yaml_sigil.proto`):
///   * `1` — `alg` (varint)
///   * `2` — `keyid` (string, optional)
///   * `3` — `signature` (bytes)
pub fn yss(alg: u64, keyid: Option<&str>, signature: &[u8]) -> Vec<u8> {
    let mut out = varint_field(1, alg);
    if let Some(k) = keyid {
        out.extend(lendel(2, k.as_bytes()));
    }
    out.extend(lendel(3, signature));
    out
}

/// `SignedYamlArtifact` outer message.
///
/// Fields:
///   * `1` — `payload` (bytes)
///   * `2` — `signature` (`YamlSigilSignature` submessage)
#[allow(dead_code)]
pub fn signed_yaml_artifact(payload: &[u8], inner: &[u8]) -> Vec<u8> {
    let mut out = lendel(1, payload);
    out.extend(lendel(2, inner));
    out
}

/// Build a YAML-form artifact: payload + constrained marker +
/// signature document.
///
/// `payload_text` is the payload exactly as it should appear (caller
/// owns the line terminators). `signature_octets` are encoded under
/// the URL-safe unpadded base64 profile defined in [`crate::b64`].
#[allow(dead_code)]
pub fn yaml_artifact(
    payload_text: &str,
    alg_name: &str,
    signature_octets: &[u8],
    keyid: Option<&str>,
) -> Vec<u8> {
    let sig_b64 = b64::urlsafe_unpadded(signature_octets);
    let mut s = String::new();
    s.push_str(payload_text);
    s.push_str("---\n");
    s.push_str("schema: YamlSigilSignature.v1alpha1\n");
    s.push_str("alg: ");
    s.push_str(alg_name);
    s.push('\n');
    if let Some(k) = keyid {
        s.push_str("keyid: ");
        s.push_str(k);
        s.push('\n');
    }
    s.push_str("signature: ");
    s.push_str(&sig_b64);
    s.push('\n');
    s.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Known-answer varint vectors from the encoding spec's worked
    /// examples (1 -> 0x01; 150 -> 0x96 0x01; 300 -> 0xAC 0x02).
    #[test]
    fn varint_known_values() {
        assert_eq!(varint(0), vec![0x00]);
        assert_eq!(varint(1), vec![0x01]);
        assert_eq!(varint(127), vec![0x7F]);
        assert_eq!(varint(128), vec![0x80, 0x01]);
        assert_eq!(varint(150), vec![0x96, 0x01]);
        assert_eq!(varint(300), vec![0xAC, 0x02]);
    }

    /// The tag for `(field=1, wire=2)` is `(1 << 3) | 2 = 0x0A`.
    /// The tag for `(field=2, wire=0)` is `(2 << 3) | 0 = 0x10`.
    #[test]
    fn tag_pack_fields_and_wires() {
        assert_eq!(tag(1, 2), vec![0x0A]);
        assert_eq!(tag(2, 0), vec![0x10]);
        assert_eq!(tag(3, 2), vec![0x1A]);
    }

    /// Length-delimited field with a known payload: tag, length-varint,
    /// then payload bytes.
    #[test]
    fn lendel_layout() {
        // field=1, payload = "abc" -> 0x0A 0x03 'a' 'b' 'c'
        assert_eq!(lendel(1, b"abc"), vec![0x0A, 0x03, b'a', b'b', b'c']);
        // field=2, empty payload -> 0x12 0x00
        assert_eq!(lendel(2, b""), vec![0x12, 0x00]);
    }

    #[test]
    fn varint_field_layout() {
        // field=1, value=1 -> 0x08 0x01
        assert_eq!(varint_field(1, 1), vec![0x08, 0x01]);
        // field=1, value=300 -> 0x08 0xAC 0x02
        assert_eq!(varint_field(1, 300), vec![0x08, 0xAC, 0x02]);
    }

    #[test]
    fn yss_alg_only() {
        // alg=1, no keyid, empty signature.
        // 0x08 0x01  (alg varint) | 0x1A 0x00 (signature LEN, zero bytes)
        assert_eq!(yss(1, None, b""), vec![0x08, 0x01, 0x1A, 0x00]);
    }

    #[test]
    fn yss_with_keyid() {
        // alg=2, keyid="k", signature=[0xAA, 0xBB]
        // 0x08 0x02  | 0x12 0x01 'k' | 0x1A 0x02 0xAA 0xBB
        let got = yss(2, Some("k"), &[0xAA, 0xBB]);
        assert_eq!(
            got,
            vec![0x08, 0x02, 0x12, 0x01, b'k', 0x1A, 0x02, 0xAA, 0xBB]
        );
    }
}
