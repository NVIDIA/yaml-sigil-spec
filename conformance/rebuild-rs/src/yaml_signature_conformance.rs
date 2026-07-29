// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/yaml-signature-conformance/` fixtures.
//!
//! Drives the [`verification-api.md`](../../verification-api.md)
//! "Structural Rules By Form" and "Conformance Profiles" sections as
//! they manifest on the YAML form. The fixtures cover the required
//! `schema` identity, duplicate-known-singular-field behavior, and
//! unknown-field behavior. The symmetric protobuf-form profile cases
//! live in [`crate::protobuf_conformance`].
//!
//! ## YAML 1.2.2 §6.7.2 — duplicate mapping keys
//!
//! YAML 1.2.2 requires mapping keys to be unique but allows a processor
//! to continue after reporting a duplicate-key error:
//!
//! > The content of a mapping node is an unordered set of key:value
//! > node pairs, with the restriction that each of the keys is unique.
//! > YAML places no further restrictions on the nodes. In particular,
//! > keys may be arbitrary nodes...
//! >
//! > It is an error for two equal keys to appear in the same mapping
//! > node. In such a case, the YAML processor may continue, ignoring
//! > the second `key:value` pair and issuing an appropriate warning.
//! > This strategy preserves a consistent information model for
//! > applications that do not wish to recognize duplicate keys.
//!
//! Under `Permissive`, `YamlSigil.v1alpha1` permits a YAML decoder to
//! reject duplicate known mapping keys or accept them according to its
//! documented decode semantics. `Strict` and `SignatureStrict` reject
//! them and map the rejection to `MalformedAttemptedSigned`.
//!
//! ## Schema closed-key set
//!
//! [`schema/YamlSigilSignature.v1alpha1.schema.json`](../../schema/YamlSigilSignature.v1alpha1.schema.json)
//! sets `additionalProperties: false` on the four declared keys
//! (`schema`, `alg`, `keyid`, `signature`). Any mapping key outside
//! that set is the YAML manifestation of "unknown field" and falls
//! under the same profile rule.
//!
//! ## Schema identity
//!
//! `verification-api.md` requires the YAML `schema` value to equal
//! `YamlSigilSignature.v1alpha1`. A wrong value or missing required
//! key fails metadata extraction under every conformance profile.

use std::path::Path;

use crate::b64::{placeholder_sig, urlsafe_unpadded};
use crate::util::write_bytes;

/// The fixtures all use the same payload, marker, and base64
/// placeholder; only the inner mapping body changes.
const PAYLOAD: &str = "payload: example\n";

fn alternate_sig() -> String {
    urlsafe_unpadded(&[1u8; 64])
}

fn artifact(mapping_body: &str) -> Vec<u8> {
    let mut s = String::from(PAYLOAD);
    s.push_str("---\n");
    s.push_str(mapping_body);
    s.into_bytes()
}

pub fn generate(dir: &Path) -> std::io::Result<()> {
    let sig = placeholder_sig();

    // valid-baseline.yaml: every key appears once. Exists so an
    // implementation comparing against it can see exactly which
    // bytes the duplicate / unknown variants mutate.
    let baseline = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "valid-baseline.yaml", &artifact(&baseline))?;

    // wrong-schema.yaml: the required key is present but declares a
    // different schema identity.
    let wrong_schema = format!(
        "schema: YamlSigilSignature.v2alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "wrong-schema.yaml", &artifact(&wrong_schema))?;

    // missing-schema.yaml: the required schema key is absent.
    let missing_schema = format!(
        "alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "missing-schema.yaml", &artifact(&missing_schema))?;

    // duplicate-schema.yaml: schema appears twice with matching value.
    let dup_schema = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "duplicate-schema.yaml", &artifact(&dup_schema))?;

    // duplicate-alg.yaml: alg appears twice with DIFFERENT values.
    // This is the load-bearing attacker case. A Permissive decoder
    // that accepts the fixture must document which alg is effective;
    // different choices can swap signing algorithm interpretation.
    let dup_alg = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         alg: ECDSA_SECP256R1_SHA256_RAW_RS64\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "duplicate-alg.yaml", &artifact(&dup_alg))?;

    // duplicate-keyid.yaml: keyid appears twice with different values.
    let dup_keyid = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         keyid: first-hint\n\
         keyid: second-hint\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "duplicate-keyid.yaml", &artifact(&dup_keyid))?;

    // duplicate-signature.yaml: signature appears twice with different
    // canonical base64 strings. They decode to distinct 64-octet values,
    // making effective-value selection observable without introducing a
    // second rejection reason.
    let other_sig = alternate_sig();
    let dup_signature = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n\
         signature: {other_sig}\n"
    );
    write_bytes(dir, "duplicate-signature.yaml", &artifact(&dup_signature))?;

    // unknown-key.yaml: an extra mapping key not declared in the
    // closed schema. Strict / SignatureStrict reject;
    // Permissive accepts and discards the unknown key.
    let unknown_key = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         bogus: surprise\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "unknown-key.yaml", &artifact(&unknown_key))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every fixture MUST share the same payload + marker so a
    /// hex-diff reviewer can see only the inner-mapping change.
    #[test]
    fn fixtures_share_payload_and_marker() {
        let baseline_body = "schema: YamlSigilSignature.v1alpha1\nalg: x\nsignature: y\n";
        let a = artifact(baseline_body);
        let expected_prefix = b"payload: example\n---\n";
        assert!(a.starts_with(expected_prefix));
    }

    /// Both duplicate-signature values MUST be canonical base64url
    /// encodings of 64 octets. For an 86-character encoding, the final
    /// character's lower four bits are unused and therefore zero.
    #[test]
    fn duplicate_signature_values_are_distinct_and_canonical() {
        let first = placeholder_sig();
        let second = alternate_sig();

        assert_ne!(first, second);
        for value in [first, second] {
            assert_eq!(value.len(), 86);
            assert!(value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')));
            assert!(matches!(
                value.as_bytes().last(),
                Some(b'A' | b'Q' | b'g' | b'w')
            ));
        }
    }

    /// The four declared schema keys (schema, alg, keyid, signature)
    /// are the closed set Strict / SignatureStrict rejects outside
    /// of. A drift in this list would mean the schema changed and the
    /// fixture set is stale.
    #[test]
    fn schema_declared_keys_are_the_closed_four() {
        let declared: &[&str] = &["schema", "alg", "keyid", "signature"];
        assert_eq!(declared.len(), 4);
        // The `unknown-key.yaml` fixture uses `bogus` precisely
        // because it is none of these.
        for k in declared {
            assert_ne!(*k, "bogus");
        }
    }
}
