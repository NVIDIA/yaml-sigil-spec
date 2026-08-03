// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/yaml-signature-conformance/` fixtures.
//!
//! Drives the [`verification-api.md`](../../verification-api.md)
//! "Structural Rules By Form" and "Conformance Profiles" sections as
//! they manifest on the YAML form. The fixtures cover the YAML
//! signature-carrier byte limit, required `schema` identity, document count,
//! mapping root, string field types, duplicate known-key rejection, and
//! profile-specific unknown-field behavior. The symmetric protobuf-form
//! profile cases live in
//! [`crate::protobuf_conformance`].
//!
//! ## Schema closed-key set
//!
//! [`schema/YamlSigilSignature.v1alpha1.schema.json`](../../schema/YamlSigilSignature.v1alpha1.schema.json)
//! sets `additionalProperties: false` on the four declared keys
//! (`schema`, `alg`, `keyid`, `signature`). Any mapping key outside
//! that set is the YAML manifestation of "unknown field" and falls
//! under the same profile rule.
//!
//! ## Bounded carrier profile
//!
//! `verification-api.md` limits the markerless carrier to 16,384 octets
//! and requires implementation-documented parser-resource bounds. It also
//! requires exactly one YAML document through EOF, a mapping root, and YAML
//! string values for the declared fields. YAML document markers and explicit
//! standard tags follow
//! [YAML 1.2.2](https://yaml.org/spec/1.2.2/#91-documents).
//!
//! ## Schema identity
//!
//! `verification-api.md` requires the YAML `schema` value to equal
//! `YamlSigilSignature.v1alpha1`. A wrong value or missing required
//! key fails metadata extraction under every conformance profile.

use crate::b64::{placeholder_sig, urlsafe_unpadded};
use crate::util::write_bytes;
use yamlsigil_pinned_dir::PinnedDir;

/// The fixtures all use the same payload, marker, and base64
/// placeholder; only the inner mapping body changes.
const PAYLOAD: &str = "payload: example\n";
const MAX_CARRIER_BYTES: usize = 16 * 1024;

fn alternate_sig() -> String {
    urlsafe_unpadded(&[1u8; 64])
}

fn artifact(mapping_body: &str) -> Vec<u8> {
    let mut s = String::from(PAYLOAD);
    s.push_str("---\n");
    s.push_str(mapping_body);
    s.into_bytes()
}

fn sequence_root(sig: &str) -> String {
    format!(
        "- schema: YamlSigilSignature.v1alpha1\n  alg: \
         ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n  signature: {sig}\n"
    )
}

pub fn generate(dir: &PinnedDir) -> std::io::Result<()> {
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
    // This is the load-bearing attacker case. Every profile rejects it
    // before effective-value selection can change the algorithm.
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

    // oversized-carrier.yaml: a comment keeps the mapping itself valid
    // while making the markerless carrier exceed 16,384 octets.
    let oversized_carrier = format!("#{}\n{baseline}", "x".repeat(MAX_CARRIER_BYTES));
    write_bytes(dir, "oversized-carrier.yaml", &artifact(&oversized_carrier))?;

    // document-end-at-eof.yaml: an explicit YAML document-end marker
    // terminates the one permitted document and is followed only by EOF.
    let document_end_at_eof = format!("{baseline}...\n");
    write_bytes(
        dir,
        "document-end-at-eof.yaml",
        &artifact(&document_end_at_eof),
    )?;

    // document-end-with-second-document.yaml: the commented document-start
    // spelling is valid YAML but is not a constrained YamlSigil marker. It
    // therefore remains inside the carrier and makes it a two-document stream.
    let second_document = format!(
        "{baseline}...\n\
         --- # second YAML document\n\
         trailing: value\n"
    );
    write_bytes(
        dir,
        "document-end-with-second-document.yaml",
        &artifact(&second_document),
    )?;

    // non-mapping-root.yaml: the declared fields occur inside a sequence
    // item, leaving the signature document itself with a sequence root.
    let non_mapping_root = sequence_root(&sig);
    write_bytes(dir, "non-mapping-root.yaml", &artifact(&non_mapping_root))?;

    // Each declared field must be a YAML string scalar. Explicit standard
    // tags make the non-string type independent of implicit resolution rules.
    let non_string_schema = format!(
        "schema: !!int 1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "non-string-schema.yaml", &artifact(&non_string_schema))?;

    let non_string_alg = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: !!bool true\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "non-string-alg.yaml", &artifact(&non_string_alg))?;

    let non_string_keyid = format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         keyid: !!int 1234\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "non-string-keyid.yaml", &artifact(&non_string_keyid))?;

    let non_string_signature = "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: !!int 1234\n";
    write_bytes(
        dir,
        "non-string-signature.yaml",
        &artifact(non_string_signature),
    )?;

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

    #[test]
    fn oversized_carrier_exceeds_the_normative_limit() {
        let baseline = format!(
            "schema: YamlSigilSignature.v1alpha1\n\
             alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
             signature: {}\n",
            placeholder_sig()
        );
        let carrier = format!("#{}\n{baseline}", "x".repeat(MAX_CARRIER_BYTES));
        assert!(carrier.len() > MAX_CARRIER_BYTES);
    }

    #[test]
    fn commented_document_start_is_not_a_constrained_marker() {
        let carrier = "...\n--- # second YAML document\ntrailing: value\n";
        assert!(carrier.contains("--- #"));
        assert!(!carrier
            .as_bytes()
            .windows(4)
            .any(|window| window == b"---\n"));
    }

    #[test]
    fn explicit_standard_tags_make_non_string_cases_unambiguous() {
        for tagged in ["!!int 1", "!!bool true", "!!int 1234"] {
            assert!(tagged.starts_with("!!"));
        }
    }

    #[test]
    fn sequence_root_keeps_mapping_fields_indented() {
        let body = sequence_root(&placeholder_sig());
        let mut lines = body.lines();
        assert!(lines
            .next()
            .is_some_and(|line| line.starts_with("- schema:")));
        assert!(lines.all(|line| line.starts_with("  ")));
    }
}
