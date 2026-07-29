// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/alg-ed25519/` fixtures.
//!
//! ## Test vectors — RFC 8032 §7.1
//!
//! Test vectors for PureEdDSA over edwards25519 are published in
//! [RFC 8032 §7.1](https://www.rfc-editor.org/rfc/rfc8032#section-7.1)
//! ("Test Vectors for Ed25519"). The first two of the published
//! vectors are reproduced verbatim below. Test 1 ("empty message"):
//!
//! ```text
//! -----TEST 1
//! ALGORITHM:     Ed25519
//! SECRET KEY:    9d61b19deffd5a60ba844af492ec2cc4
//!                4449c5697b326919703bac031cae7f60
//! PUBLIC KEY:    d75a980182b10ab7d54bfed3c964073a
//!                0ee172f3daa62325af021a68f707511a
//! MESSAGE (length 0 bytes):
//! SIGNATURE:     e5564300c360ac729086e2cc806e828a
//!                84877f1eb8e5d974d873e06522490155
//!                5fb8821590a33bacc61e39701cf9b46b
//!                d25bf5f0595bbe24655141438e7a100b
//! ```
//!
//! Test 2 ("one-octet message `0x72`"):
//!
//! ```text
//! -----TEST 2
//! ALGORITHM:     Ed25519
//! SECRET KEY:    4ccd089b28ff96da9db6c346ec114e0f
//!                5b8a319f35aba624da8cf6ed4fb8a6fb
//! PUBLIC KEY:    3d4017c3e843895a92b70aa74d1b7ebc
//!                9c982ccf2ec4968cc0cd55f12af4660c
//! MESSAGE (length 1 byte): 72
//! SIGNATURE:     92a009a9f0d4cab8720e820b5f642540
//!                a2b27b5416503f8fb3762223ebdb69da
//!                085ac1e43e15996e458f3613d0f11d8c
//!                387b2eaeb4302aeeb00d291612bb0c00
//! ```
//!
//! These are the exact bytes loaded by [`SEED_1_HEX`], [`PUB_1_HEX`],
//! [`SIG_1_HEX`], [`SEED_2_HEX`], [`PUB_2_HEX`], and [`SIG_2_HEX`].
//! They are third-party RFC test-vector material, not material relicensed
//! under the Apache-2.0 declaration on this NVIDIA-authored generator.
//! See the repository `THIRD_PARTY_NOTICES.md` for the applicable source
//! attribution and terms.
//!
//! ## Group order `L` — RFC 8032 §5.1
//!
//! [RFC 8032 §5.1](https://www.rfc-editor.org/rfc/rfc8032#section-5.1)
//! ("Ed25519ph, Ed25519ctx, and Ed25519") fixes the cofactor and the
//! prime order `L` of the base point:
//!
//! > The curve has 2^252 + 27742317777372353535851937790883648493
//! > points of prime order; this is also the order of the prime-order
//! > subgroup. The cofactor is 8.
//!
//! [`L_HEX`] encodes that prime in little-endian octets — the encoding
//! convention RFC 8032 uses for both the secret-scalar `s` and the
//! signature component `S` (§5.1.6 / §5.1.7).
//!
//! ## Small-order points — Chalkias / Garillot / Nikolaenko (2020)
//!
//! Chalkias, Garillot, and Nikolaenko, ["Taming the Many EdDSAs"](https://eprint.iacr.org/2020/1244)
//! (IACR ePrint 2020/1244), Table 5 reports the eight small-order
//! encodings on edwards25519. Those numeric 32-octet hex strings are
//! recorded in [`SMALL_ORDER_POINTS`]; the `configured-key-small-order`
//! fixture writes them out for verifier-side `KeyResolutionFailure`
//! tests. The strict-variant verification rule (Algorithm 2 in the
//! same paper) is the rule this crate's spec adopts. See the repository
//! `THIRD_PARTY_NOTICES.md` for source attribution.

use std::path::Path;

use crate::b64::urlsafe_unpadded;
use crate::util::{from_hex, hex_lower, write_bytes, write_text};
use crate::wire::{lendel, varint_field};

const ED25519_ALG: u64 = 1;

/// RFC 8032 §7.1 Test 1 — secret key (a.k.a. seed), 32 octets, hex.
pub const SEED_1_HEX: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
/// RFC 8032 §7.1 Test 1 — public key, 32 octets, hex.
pub const PUB_1_HEX: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
/// RFC 8032 §7.1 Test 1 — signature (R || S), 64 octets, hex.
pub const SIG_1_HEX: &str = concat!(
    "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155",
    "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
);

/// RFC 8032 §7.1 Test 2 — secret key (seed) for the one-byte message.
pub const SEED_2_HEX: &str = "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb";
/// RFC 8032 §7.1 Test 2 — public key.
pub const PUB_2_HEX: &str = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
/// RFC 8032 §7.1 Test 2 — signature.
pub const SIG_2_HEX: &str = concat!(
    "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da",
    "085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
);

/// Prime order `L = 2^252 + 27742317777372353535851937790883648493`
/// of the base point, little-endian (RFC 8032 §5.1).
pub const L_HEX: &str = "edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";

/// The eight numeric 32-octet small-order edwards25519 point encodings
/// reported in Table 5 of "Taming the Many EdDSAs" (IACR ePrint 2020/1244).
pub const SMALL_ORDER_POINTS: &[&str] = &[
    "0100000000000000000000000000000000000000000000000000000000000000",
    "ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000080",
    "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05",
    "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a",
    "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc85",
    "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac03fa",
];

fn proto_artifact(payload: &[u8], signature: &[u8]) -> Vec<u8> {
    let mut inner = varint_field(1, ED25519_ALG);
    inner.extend(lendel(3, signature));
    let mut out = lendel(1, payload);
    out.extend(lendel(2, &inner));
    out
}

fn yaml_empty_payload(signature: &[u8]) -> Vec<u8> {
    let sig_b64 = urlsafe_unpadded(signature);
    format!(
        "---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig_b64}\n"
    )
    .into_bytes()
}

fn signature_with_s(r: &[u8], s: &[u8]) -> Vec<u8> {
    assert_eq!(r.len(), 32, "R must be 32 octets");
    assert_eq!(s.len(), 32, "S must be 32 octets");
    let mut signature = Vec::with_capacity(64);
    signature.extend_from_slice(r);
    signature.extend_from_slice(s);
    signature
}

pub fn generate(dir: &Path) -> std::io::Result<()> {
    let seed_1 = from_hex(SEED_1_HEX);
    let pub_1 = from_hex(PUB_1_HEX);
    let sig_1 = from_hex(SIG_1_HEX);

    let seed_2 = from_hex(SEED_2_HEX);
    let pub_2 = from_hex(PUB_2_HEX);
    let msg_2 = from_hex("72");
    let sig_2 = from_hex(SIG_2_HEX);

    let l_bytes = from_hex(L_HEX);
    {
        use num_bigint::BigUint;
        let l_int = BigUint::from_bytes_le(&l_bytes);
        let expected = (BigUint::from(1u64) << 252u32)
            + BigUint::parse_bytes(b"27742317777372353535851937790883648493", 10)
                .expect("decimal literal parses");
        assert_eq!(l_int, expected, "L curve-order mismatch");
        println!("  L = {l_int}  (matches 2^252 + 27742317777372353535851937790883648493)");
    }

    // Test 1 (both forms)
    write_bytes(
        dir,
        "rfc8032-vec1-empty-message.binpb",
        &proto_artifact(&[], &sig_1),
    )?;
    write_bytes(
        dir,
        "rfc8032-vec1-empty-message.yaml",
        &yaml_empty_payload(&sig_1),
    )?;
    write_text(
        dir,
        "rfc8032-vec1-empty-message.expected.txt",
        &format!(
            concat!(
                "# RFC 8032 §7.1 Test 1 (empty message)\n",
                "# Provenance: RFC 8032 §7.1 test-vector values; see ../../THIRD_PARTY_NOTICES.md.\n",
                "# seed: {seed}\n",
                "# public_key: {pub_}\n",
                "# message (signed bytes): (empty)\n",
                "# signature: {sig}\n\n",
                "Verify(form=PROTOBUF, input=rfc8032-vec1-empty-message.binpb,\n",
                "       config.public_key_handle=<the 32-octet public key above>)\n",
                "Expected: Verified\n\n",
                "Verify(form=YAML, input=rfc8032-vec1-empty-message.yaml,\n",
                "       config.public_key_handle=<same 32-octet public key>)\n",
                "Expected: Verified\n",
            ),
            seed = hex_lower(&seed_1),
            pub_ = hex_lower(&pub_1),
            sig = hex_lower(&sig_1),
        ),
    )?;

    // Test 2 (protobuf only)
    write_bytes(
        dir,
        "rfc8032-vec2-one-octet.binpb",
        &proto_artifact(&msg_2, &sig_2),
    )?;
    write_text(
        dir,
        "rfc8032-vec2-one-octet.expected.txt",
        &format!(
            concat!(
                "# RFC 8032 §7.1 Test 2 (one-byte message 0x72 = 'r')\n",
                "# Provenance: RFC 8032 §7.1 test-vector values; see ../../THIRD_PARTY_NOTICES.md.\n",
                "# seed: {seed}\n",
                "# public_key: {pub_}\n",
                "# message (signed bytes): {msg}\n",
                "# signature: {sig}\n\n",
                "Verify(form=PROTOBUF, input=rfc8032-vec2-one-octet.binpb,\n",
                "       config.public_key_handle=<the 32-octet public key above>)\n",
                "Expected: Verified\n\n",
                "# Note: no YAML form is shipped. The signed bytes are a single\n",
                "# byte (0x72) that cannot precede a constrained YAML marker\n",
                "# without inserting an extra newline that would change the\n",
                "# signed payload. The YAML form requires the payload to end\n",
                "# with 0x0A or 0x0D 0x0A (or be empty).\n",
            ),
            seed = hex_lower(&seed_2),
            pub_ = hex_lower(&pub_2),
            msg = hex_lower(&msg_2),
            sig = hex_lower(&sig_2),
        ),
    )?;

    // Canonical-encoding rejection fixtures
    let mut nc_r = vec![0xED];
    nc_r.extend(std::iter::repeat_n(0xFFu8, 30));
    nc_r.push(0x7F);
    let mut nc_r_sig = nc_r.clone();
    nc_r_sig.extend(std::iter::repeat_n(0u8, 32));

    // Retain the valid R component from RFC 8032 §7.1 Test 1 so these
    // fixtures isolate the S range violation.
    let r_1 = &sig_1[..32];
    assert!(
        !SMALL_ORDER_POINTS
            .iter()
            .any(|point| from_hex(point) == r_1),
        "RFC 8032 Test 1 R unexpectedly matches a small-order encoding"
    );
    let s_eq_l = signature_with_s(r_1, &l_bytes);

    let l_plus_1_le = {
        use num_bigint::BigUint;
        let v = BigUint::from_bytes_le(&l_bytes) + BigUint::from(1u64);
        let mut bytes = v.to_bytes_le();
        bytes.resize(32, 0);
        bytes
    };
    let s_eq_l_plus_1 = signature_with_s(r_1, &l_plus_1_le);

    write_bytes(dir, "noncanonical-R.binpb", &proto_artifact(&[], &nc_r_sig))?;
    write_bytes(
        dir,
        "noncanonical-S-equals-L.binpb",
        &proto_artifact(&[], &s_eq_l),
    )?;
    write_bytes(
        dir,
        "noncanonical-S-equals-L-plus-1.binpb",
        &proto_artifact(&[], &s_eq_l_plus_1),
    )?;
    write_text(
        dir,
        "noncanonical-encoding.expected.txt",
        &format!(
            concat!(
                "# Three canonical-encoding rejection fixtures.\n",
                "# Provenance: RFC 8032 §§5.1 and 7.1 values; see ../../THIRD_PARTY_NOTICES.md.\n",
                "# All use payload = (empty), public_key = RFC 8032 §7.1 Test 1's public key.\n",
                "# The S-boundary fixtures retain Test 1's valid R component.\n",
                "# public_key: {pub_}\n\n",
                "Verify(*, input=noncanonical-R.binpb, ...)               -> MalformedAttemptedSigned\n",
                "Verify(*, input=noncanonical-S-equals-L.binpb, ...)      -> MalformedAttemptedSigned\n",
                "Verify(*, input=noncanonical-S-equals-L-plus-1.binpb, ...) -> MalformedAttemptedSigned\n",
            ),
            pub_ = hex_lower(&pub_1),
        ),
    )?;

    let mut small_order_lines = String::new();
    for p in SMALL_ORDER_POINTS {
        small_order_lines.push_str(p);
        small_order_lines.push('\n');
    }
    write_text(
        dir,
        "configured-key-small-order.txt",
        &format!(
            concat!(
                "# Eight small-order public-key encodings on edwards25519.\n",
                "# Each line is a 32-octet public key in hex (lower-case).\n",
                "# Source: Chalkias, Garillot, Nikolaenko, \"Taming the Many EdDSAs\"\n",
                "# (IACR ePrint 2020/1244) Table 5.\n\n",
                "# Expected: a verifier configured with any of these as\n",
                "# config.public_key_handle MUST return KeyResolutionFailure.\n\n",
                "{small_order_lines}",
            ),
            small_order_lines = small_order_lines,
        ),
    )?;

    write_text(
        dir,
        "stable-resign.txt",
        &format!(
            concat!(
                "# Stable re-signing.\n",
                "# Provenance: RFC 8032 §7.1 Test 1 values; see ../../THIRD_PARTY_NOTICES.md.\n",
                "# Sign (seed, message) twice; both invocations MUST produce byte-identical signatures.\n\n",
                "seed:               {seed}\n",
                "public_key:         {pub_}\n",
                "message:            (empty)\n",
                "expected signature: {sig}\n",
                "                    (both first and second invocation produce this byte-for-byte)\n",
            ),
            seed = hex_lower(&seed_1),
            pub_ = hex_lower(&pub_1),
            sig = hex_lower(&sig_1),
        ),
    )?;

    write_text(
        dir,
        "algorithm-parameters-present.expected.txt",
        "# algorithm_parameters MUST be absent or zero-length for this algorithm.\n\n\
         Sign(alg=ED25519_PUREEDDSA_RAW_RS64_CANONICAL,\n     \
         algorithm_parameters=b'\\x00')\n\
         -> SignerInvocationError(InvalidAlgorithmParameters)\n\n\
         Verify(..., config.algorithm_parameters=b'\\x00')\n\
         -> InvocationError(InvalidAlgorithmParameters)\n",
    )?;

    println!("  ed25519 fixtures written.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every RFC 8032 §7.1 hex constant has the length the RFC pins.
    #[test]
    fn rfc8032_constant_widths() {
        assert_eq!(SEED_1_HEX.len(), 64); // 32 octets
        assert_eq!(PUB_1_HEX.len(), 64);
        assert_eq!(SIG_1_HEX.len(), 128); // 64 octets
        assert_eq!(SEED_2_HEX.len(), 64);
        assert_eq!(PUB_2_HEX.len(), 64);
        assert_eq!(SIG_2_HEX.len(), 128);
        assert_eq!(L_HEX.len(), 64);
    }

    /// `L` decoded little-endian MUST equal
    /// `2^252 + 27742317777372353535851937790883648493` (RFC 8032 §5.1).
    #[test]
    fn group_order_matches_rfc8032() {
        use num_bigint::BigUint;
        let bytes = from_hex(L_HEX);
        let l = BigUint::from_bytes_le(&bytes);
        let expected = (BigUint::from(1u64) << 252u32)
            + BigUint::parse_bytes(b"27742317777372353535851937790883648493", 10).unwrap();
        assert_eq!(l, expected);
    }

    /// Table 5 of "Taming the Many EdDSAs" lists exactly eight
    /// small-order encodings, each 32 octets (64 hex characters).
    #[test]
    fn small_order_table_has_eight_32_octet_entries() {
        assert_eq!(SMALL_ORDER_POINTS.len(), 8);
        for p in SMALL_ORDER_POINTS {
            assert_eq!(p.len(), 64);
            assert_eq!(from_hex(p).len(), 32);
        }
    }

    /// The S-boundary fixtures retain RFC 8032 §7.1 Test 1's valid
    /// `R`, and that encoding is not one of the eight small-order points.
    #[test]
    fn rfc8032_test_1_r_is_not_small_order() {
        let signature = from_hex(SIG_1_HEX);
        let r = &signature[..32];
        assert!(!SMALL_ORDER_POINTS.iter().any(|point| from_hex(point) == r));
    }
}
