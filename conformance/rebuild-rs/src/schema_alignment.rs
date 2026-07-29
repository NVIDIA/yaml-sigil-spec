// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/schema-alignment/` fixtures.
//!
//! Drives the cross-walk between four authoritative artifacts:
//!
//! - The repo-level `README.md` "The Signature Document" section,
//!   which lists the canonical (non-prefixed) algorithm names.
//! - `proto/yaml_sigil/v1alpha1/yaml_sigil.proto`,
//!   which assigns integer values to each algorithm in the protobuf
//!   `Algorithm` enum (slot `1` = Ed25519, slot `2` = ECDSA).
//! - `schema/YamlSigilSignature.v1alpha1.schema.json`, which restricts
//!   the YAML `alg` field to the canonical (non-prefixed) string set.
//! - `verification-api.md` "Algorithm Policy", which maps
//!   schema-unknown names and integers plus `ALGORITHM_UNSPECIFIED`
//!   to `MalformedAttemptedSigned`. It reserves
//!   `SignedButAlgorithmUnsupported` for schema-defined algorithms
//!   that the verifier does not implement. Its runtime ordering also
//!   maps an empty signature to `MalformedAttemptedSigned` before
//!   classifying algorithm support.
//!
//! ## Why both forms ship per fixture
//!
//! These fixtures pin the *string* form (YAML) and the *integer* form
//! (protobuf) for the same logical algorithm slot. The protobuf enum
//! field is a varint per the
//! [Protocol Buffers spec](https://protobuf.dev/programming-guides/proto3/#enum):
//!
//! > During deserialization, unrecognized enum values are preserved
//! > in the message... If the message is serialized again, the
//! > unrecognized value is still serialized with the message.
//!
//! For YAML, the JSON Schema's `enum` constraint rejects strings
//! outside its set. The fixture matrix covers the recognised slots
//! plus the unknown-string / unknown-integer / `UNSPECIFIED` cases
//! that drive each verifier state. The empty-signature pair pins the
//! precedence between malformed signature content and runtime
//! algorithm-support classification across both forms.

use std::path::Path;

use crate::b64::placeholder_sig;
use crate::util::write_bytes;
use crate::wire::{lendel, varint_field};

const PAYLOAD: &[u8] = b"hello: world\n";

pub fn generate(dir: &Path) -> std::io::Result<()> {
    let sig = placeholder_sig();
    let sig64 = [0u8; 64];

    let yaml_artifact = |alg_value: &str, signature_value: &str| -> Vec<u8> {
        format!(
            "payload: example\n\
             ---\n\
             schema: YamlSigilSignature.v1alpha1\n\
             alg: {alg_value}\n\
             signature: {signature_value}\n"
        )
        .into_bytes()
    };

    let proto_artifact = |alg_value: u64, signature: &[u8]| -> Vec<u8> {
        let mut inner = varint_field(1, alg_value);
        inner.extend(lendel(3, signature));
        let mut out = lendel(1, PAYLOAD);
        out.extend(lendel(2, &inner));
        out
    };

    let yaml_cases: &[(&str, &str)] = &[
        (
            "yaml-alg-ed25519.yaml",
            "ED25519_PUREEDDSA_RAW_RS64_CANONICAL",
        ),
        ("yaml-alg-ecdsa.yaml", "ECDSA_SECP256R1_SHA256_RAW_RS64"),
        ("yaml-alg-unknown-string.yaml", "FOO_BAR_BAZ"),
        (
            "yaml-alg-prefixed-rejected.yaml",
            "ALGORITHM_ED25519_PUREEDDSA_RAW_RS64_CANONICAL",
        ),
        (
            "yaml-alg-unspecified-rejected.yaml",
            "ALGORITHM_UNSPECIFIED",
        ),
    ];
    for (name, alg) in yaml_cases {
        write_bytes(dir, name, &yaml_artifact(alg, &sig))?;
    }
    write_bytes(
        dir,
        "yaml-alg-ecdsa-empty-signature.yaml",
        &yaml_artifact("ECDSA_SECP256R1_SHA256_RAW_RS64", "\"\""),
    )?;

    let proto_cases: &[(&str, u64)] = &[
        ("proto-alg-ed25519.binpb", 1),
        ("proto-alg-ecdsa.binpb", 2),
        ("proto-alg-unspecified.binpb", 0),
        ("proto-alg-unknown-integer.binpb", 42),
    ];
    for (name, alg) in proto_cases {
        write_bytes(dir, name, &proto_artifact(*alg, &sig64))?;
    }
    write_bytes(
        dir,
        "proto-alg-ecdsa-empty-signature.binpb",
        &proto_artifact(2, &[]),
    )?;
    write_bytes(
        dir,
        "empty-signature-before-unsupported.expected.txt",
        b"Verifier configuration: ECDSA_SECP256R1_SHA256_RAW_RS64 is schema-defined but not implemented.\n\
          PreVerify(*, input=yaml-alg-ecdsa-empty-signature.yaml) -> Ok\n\
          PreVerify(*, input=proto-alg-ecdsa-empty-signature.binpb) -> Ok\n\
          Verify(*, input=yaml-alg-ecdsa-empty-signature.yaml) -> MalformedAttemptedSigned\n\
          Verify(*, input=proto-alg-ecdsa-empty-signature.binpb) -> MalformedAttemptedSigned\n",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::wire::{lendel, varint_field};

    #[test]
    fn proto_alg_ed25519_starts_with_expected_alg_varint() {
        // Inner: alg varint (field=1, value=1) is 0x08 0x01.
        let inner_start = varint_field(1, 1);
        assert_eq!(inner_start, vec![0x08, 0x01]);
    }

    #[test]
    fn proto_alg_unspecified_is_value_zero() {
        // Unrecognised-integer fixtures use value 0 for UNSPECIFIED
        // and value 42 for a non-zero unknown. Both encode cleanly.
        assert_eq!(varint_field(1, 0), vec![0x08, 0x00]);
        assert_eq!(varint_field(1, 42), vec![0x08, 0x2A]);
    }

    #[test]
    fn empty_signature_is_explicit_zero_length_bytes() {
        assert_eq!(lendel(3, &[]), vec![0x1A, 0x00]);
    }

    #[test]
    fn lendel_payload_length_is_visible_after_tag() {
        let payload: &[u8] = b"hello: world\n";
        let v = lendel(1, payload);
        assert_eq!(v[1] as usize, payload.len());
    }
}
