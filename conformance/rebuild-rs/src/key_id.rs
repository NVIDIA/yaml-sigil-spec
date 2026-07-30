// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/key-id/` fixtures.
//!
//! The `keyid` field is an optional UTF-8 string bounded by 1..=1024
//! UTF-8 *octets* with CR and LF excluded. The repo-level `README.md`
//! "The Signature Document" section is normative. The JSON Schema
//! (`schema/YamlSigilSignature.v1alpha1.schema.json`) uses
//! `maxLength: 1024`, which in JSON Schema is measured in *code
//! points* — that mismatch is itself one of the things these fixtures
//! exercise.
//!
//! ## UTF-8 octet measurement — RFC 3629
//!
//! [RFC 3629 §3](https://www.rfc-editor.org/rfc/rfc3629#section-3)
//! ("UTF-8 definition") gives the per-code-point octet counts:
//!
//! > In UTF-8, characters from the U+0000..U+10FFFF range (the UTF-16
//! > accessible range) are encoded using sequences of 1 to 4 octets.
//!
//! Specifically:
//!
//! > Char. number range  |        UTF-8 octet sequence
//! >    (hexadecimal)    |              (binary)
//! > --------------------+---------------------------------------------
//! > 0000 0000-0000 007F | 0xxxxxxx
//! > 0000 0080-0000 07FF | 110xxxxx 10xxxxxx
//! > 0000 0800-0000 FFFF | 1110xxxx 10xxxxxx 10xxxxxx
//! > 0001 0000-0010 FFFF | 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
//!
//! This excerpt and table are RFC material, not material relicensed under
//! the Apache-2.0 declaration on this NVIDIA-authored generator. See the
//! repository `THIRD_PARTY_NOTICES.md` for the source attribution, copying
//! conditions, warranty disclaimer, and intellectual-property caveat.
//!
//! The multibyte fixtures below use `U+1F600` (😀, falling in the
//! 4-octet range), giving exactly 4 UTF-8 octets per code point.
//! This drives a wedge between octet-counting and code-point-counting
//! implementations: `256 × 😀` is 256 code points (passes JSON
//! Schema `maxLength: 1024`) but 1024 UTF-8 octets (right at the
//! protobuf/decoder octet limit); `257 × 😀` is 257 code points
//! (still passes the schema) but 1028 UTF-8 octets (over the limit).
//! The fixtures' purpose is to surface that disagreement.

use crate::b64::placeholder_sig;
use crate::util::write_bytes;
use crate::wire::{lendel, varint_field};
use yamlsigil_pinned_dir::PinnedDir;

const PAYLOAD: &[u8] = b"payload: example\n";

fn yaml_artifact(sig: &str, keyid_line: Option<&str>) -> Vec<u8> {
    let mut s = String::from(
        "payload: example\n\
         ---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n",
    );
    if let Some(line) = keyid_line {
        s.push_str(line);
        s.push('\n');
    }
    s.push_str("signature: ");
    s.push_str(sig);
    s.push('\n');
    s.into_bytes()
}

fn proto_artifact(keyid: Option<&str>) -> Vec<u8> {
    let mut inner = varint_field(1, 1);
    if let Some(k) = keyid {
        inner.extend(lendel(2, k.as_bytes()));
    }
    inner.extend(lendel(3, &[0u8; 64]));
    let mut out = lendel(1, PAYLOAD);
    out.extend(lendel(2, &inner));
    out
}

/// `keyid` that places a constrained marker inside a single-quoted carrier.
const MARKER_INJECTION_KEYID: &str = "kid\n\
     ---\n\
     schema: YamlSigilSignature.v1alpha1\n\
     alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL #";

/// Markerless carrier rejected by YAML Compose under `transcription-api.md`.
fn marker_injection_carrier(sig: &str) -> Vec<u8> {
    format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         keyid: '{MARKER_INJECTION_KEYID}'\n\
         signature: {sig}\n"
    )
    .into_bytes()
}

pub fn generate(dir: &PinnedDir) -> std::io::Result<()> {
    let sig = placeholder_sig();

    let key_1024: String = "a".repeat(1024);
    let key_1025: String = "a".repeat(1025);
    let key_mb_under: String = "\u{1F600}".repeat(256); // 256 cp, 1024 octets
    let key_mb_over: String = "\u{1F600}".repeat(257); //  257 cp, 1028 octets

    assert_eq!(key_1024.len(), 1024);
    assert_eq!(key_1025.len(), 1025);
    assert_eq!(key_mb_under.len(), 1024);
    assert_eq!(key_mb_over.len(), 1028);
    assert_eq!(key_mb_under.chars().count(), 256);
    assert_eq!(key_mb_over.chars().count(), 257);

    let write_pair = |dir: &PinnedDir,
                      stem: &str,
                      yaml_keyid_line: Option<&str>,
                      proto_keyid: Option<&str>|
     -> std::io::Result<()> {
        let yfn = format!("{stem}.yaml");
        let pfn = format!("{stem}.binpb");
        let yb = yaml_artifact(&sig, yaml_keyid_line);
        let pb = proto_artifact(proto_keyid);
        write_bytes(dir, &yfn, &yb)?;
        write_bytes(dir, &pfn, &pb)?;
        Ok(())
    };

    write_pair(dir, "keyid-absent", None, None)?;
    write_pair(dir, "keyid-present-empty", Some("keyid: \"\""), Some(""))?;
    write_pair(
        dir,
        "keyid-1024-ascii",
        Some(&format!("keyid: {key_1024}")),
        Some(&key_1024),
    )?;
    write_pair(
        dir,
        "keyid-1025-ascii",
        Some(&format!("keyid: {key_1025}")),
        Some(&key_1025),
    )?;
    write_pair(
        dir,
        "keyid-multibyte-under",
        Some(&format!("keyid: \"{key_mb_under}\"")),
        Some(&key_mb_under),
    )?;
    write_pair(
        dir,
        "keyid-multibyte-over",
        Some(&format!("keyid: \"{key_mb_over}\"")),
        Some(&key_mb_over),
    )?;
    write_pair(
        dir,
        "keyid-line-break",
        Some("keyid: \"kid\\nsuffix\""),
        Some("kid\nsuffix"),
    )?;
    write_bytes(
        dir,
        "keyid-marker-injection.carrier.txt",
        &marker_injection_carrier(&sig),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    /// `U+1F600` MUST encode as four UTF-8 octets per RFC 3629 §3.
    /// If a future Rust changes that, every multibyte fixture would
    /// stop reading at exactly 1024 / 1028 octets — fail loudly.
    #[test]
    fn u1f600_is_four_utf8_octets() {
        let s = "\u{1F600}";
        assert_eq!(s.len(), 4);
        assert_eq!(s.chars().count(), 1);
    }

    /// The boundary arithmetic the fixtures rely on.
    #[test]
    fn multibyte_octet_arithmetic() {
        assert_eq!("\u{1F600}".repeat(256).len(), 1024);
        assert_eq!("\u{1F600}".repeat(257).len(), 1028);
    }

    #[test]
    fn marker_injection_carrier_contains_a_constrained_marker() {
        let carrier = super::marker_injection_carrier(&crate::b64::placeholder_sig());
        assert!(carrier.windows(5).any(|window| window == b"\n---\n"));
    }
}
