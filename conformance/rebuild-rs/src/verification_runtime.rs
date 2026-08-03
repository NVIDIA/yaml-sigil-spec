// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/verification-runtime/` fixtures.
//!
//! These fixtures drive the runtime result distinctions in
//! `verification-api.md`: implemented algorithm, unsupported algorithm,
//! successful verification, and cryptographic mismatch. The key representation
//! and resolution mechanism remain implementation-defined; the generated
//! sidecar describes the required semantics.
//!
//! The fixtures use the repository's ECDSA P-256 / SHA-256 algorithm profile.
//! ECDSA operations and secp256r1 parameters are supplied by [`crate::p256`],
//! whose rustdoc cites *Standards for Efficient Cryptography 1 (SEC 1)*,
//! *Standards for Efficient Cryptography 2 (SEC 2)*, and FIPS 186-5 and records
//! their provenance and terms. Those standards-derived methods and parameters
//! are not relicensed under this file's Apache-2.0 declaration. The payload,
//! test-only private scalar, and nonce below are locally selected; these are
//! not copied test vectors.

use num_bigint::{BigInt, Sign};
use sha2::{Digest, Sha256};
use yamlsigil_pinned_dir::PinnedDir;

use crate::p256;
use crate::util::{hex_lower, write_bytes, write_text};
use crate::wire::{signed_yaml_artifact, yaml_artifact, yss};

const YAML_ALG: &str = "ECDSA_SECP256R1_SHA256_RAW_RS64";
const PROTO_ALG: u64 = 2;
const VALID_PAYLOAD: &str = "payload: runtime classification\n";
const MISMATCH_PAYLOAD: &str = "payload: runtime classificatioo\n";

struct FixtureMaterial {
    private_scalar: BigInt,
    nonce: BigInt,
    public_key: Vec<u8>,
    signature: Vec<u8>,
}

fn digest_integer(payload: &[u8]) -> BigInt {
    BigInt::from_bytes_be(Sign::Plus, &Sha256::digest(payload))
}

fn raw_signature(r: &BigInt, s: &BigInt) -> Vec<u8> {
    let mut signature = p256::to_32_be(r);
    signature.extend(p256::to_32_be(s));
    signature
}

fn fixture_material() -> FixtureMaterial {
    // Locally selected non-secret test values within the P-256 scalar range.
    let private_scalar = BigInt::from(7u8);
    let nonce = BigInt::from(11u8);
    let params = p256::params();
    let public_point =
        p256::point_mul(&private_scalar, &params.g).expect("test public key is non-identity");

    let mut public_key = vec![0x04];
    public_key.extend(p256::to_32_be(&public_point.x));
    public_key.extend(p256::to_32_be(&public_point.y));

    let valid_digest = digest_integer(VALID_PAYLOAD.as_bytes());
    let (r, s) = p256::ecdsa_sign(&private_scalar, &valid_digest, &nonce);
    assert!(p256::ecdsa_verify(&public_point, &valid_digest, &r, &s));
    assert!(!p256::ecdsa_verify(
        &public_point,
        &digest_integer(MISMATCH_PAYLOAD.as_bytes()),
        &r,
        &s
    ));

    FixtureMaterial {
        private_scalar,
        nonce,
        public_key,
        signature: raw_signature(&r, &s),
    }
}

fn protobuf_artifact(payload: &[u8], signature: &[u8]) -> Vec<u8> {
    signed_yaml_artifact(payload, &yss(PROTO_ALG, None, signature))
}

pub fn generate(dir: &PinnedDir) -> std::io::Result<()> {
    let material = fixture_material();

    write_bytes(
        dir,
        "valid.binpb",
        &protobuf_artifact(VALID_PAYLOAD.as_bytes(), &material.signature),
    )?;
    write_bytes(
        dir,
        "valid.yaml",
        &yaml_artifact(VALID_PAYLOAD, YAML_ALG, &material.signature, None),
    )?;
    write_bytes(
        dir,
        "cryptographic-mismatch.binpb",
        &protobuf_artifact(MISMATCH_PAYLOAD.as_bytes(), &material.signature),
    )?;
    write_bytes(
        dir,
        "cryptographic-mismatch.yaml",
        &yaml_artifact(MISMATCH_PAYLOAD, YAML_ALG, &material.signature, None),
    )?;

    write_text(
        dir,
        "runtime-classification.expected.txt",
        &format!(
            concat!(
                "# Locally generated Verification runtime classification matrix.\n",
                "# ECDSA procedures and parameters derive from FIPS 186-5, Standards for\n",
                "# Efficient Cryptography 1 (SEC 1), and Standards for Efficient\n",
                "# Cryptography 2 (SEC 2). They are not relicensed under Apache-2.0.\n",
                "# See ../../THIRD_PARTY_NOTICES.md for source terms and caveats.\n",
                "# The test-only private scalar, nonce, and payloads are locally selected.\n\n",
                "algorithm: ECDSA_SECP256R1_SHA256_RAW_RS64\n",
                "private scalar d (hex): {d_hex}\n",
                "ephemeral nonce k (hex): {k_hex}\n",
                "public key Q, uncompressed (hex): {public_key_hex}\n",
                "signature R || S (hex): {signature_hex}\n",
                "valid payload (hex): {valid_payload_hex}\n",
                "mismatch payload (hex): {mismatch_payload_hex}\n\n",
                "Map the public key below into the implementation's key representation and\n",
                "resolution mechanism. No literal public_key_handle encoding is prescribed.\n",
                "Run each case for both the .yaml and .binpb member.\n\n",
                "supported-algorithm:\n",
                "  implemented algorithms include ECDSA_SECP256R1_SHA256_RAW_RS64\n",
                "  key resolution returns Q above\n",
                "  PreVerify(valid) -> Ok\n",
                "  Verify(valid) -> Verified\n",
                "  verified_payload_bytes -> valid payload above\n\n",
                "unsupported-algorithm:\n",
                "  ECDSA_SECP256R1_SHA256_RAW_RS64 is schema-defined but not implemented\n",
                "  caller configuration passes invocation validation\n",
                "  PreVerify(valid) -> Ok\n",
                "  Verify(valid) -> SignedButAlgorithmUnsupported\n",
                "  verified_payload_bytes -> absent\n\n",
                "cryptographic-mismatch:\n",
                "  use the supported-algorithm configuration\n",
                "  PreVerify(cryptographic-mismatch) -> Ok\n",
                "  Verify(cryptographic-mismatch) -> SignedButFailedVerification\n",
                "  verified_payload_bytes -> absent\n",
            ),
            d_hex = p256::hex64(&material.private_scalar),
            k_hex = p256::hex64(&material.nonce),
            public_key_hex = hex_lower(&material.public_key),
            signature_hex = hex_lower(&material.signature),
            valid_payload_hex = hex_lower(VALID_PAYLOAD.as_bytes()),
            mismatch_payload_hex = hex_lower(MISMATCH_PAYLOAD.as_bytes()),
        ),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_signature_and_mismatch_have_distinct_crypto_results() {
        let material = fixture_material();
        let r = BigInt::from_bytes_be(Sign::Plus, &material.signature[..32]);
        let s = BigInt::from_bytes_be(Sign::Plus, &material.signature[32..]);
        let public_point = p256::point_mul(&material.private_scalar, &p256::params().g)
            .expect("test public key is non-identity");

        assert!(p256::ecdsa_verify(
            &public_point,
            &digest_integer(VALID_PAYLOAD.as_bytes()),
            &r,
            &s
        ));
        assert!(!p256::ecdsa_verify(
            &public_point,
            &digest_integer(MISMATCH_PAYLOAD.as_bytes()),
            &r,
            &s
        ));
    }

    #[test]
    fn both_payloads_fit_the_yaml_envelope() {
        for payload in [VALID_PAYLOAD, MISMATCH_PAYLOAD] {
            assert!(!payload.is_empty());
            assert!(payload.ends_with('\n'));
            assert!(!payload.starts_with('\u{feff}'));
        }
    }

    #[test]
    fn public_key_and_signature_have_fixed_widths() {
        let material = fixture_material();
        assert_eq!(material.public_key.len(), 65);
        assert_eq!(material.public_key[0], 0x04);
        assert_eq!(material.signature.len(), 64);
    }
}
