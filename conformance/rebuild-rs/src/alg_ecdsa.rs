// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/alg-ecdsa/` fixtures.
//!
//! This module composes [`crate::p256`] (which carries the SEC 2 v2.0
//! domain parameters and the SEC 1 v2.0 / FIPS 186-5 point-and-ECDSA
//! formulae) with SHA-256 from the `sha2` crate. The cryptographic
//! contracts cited in `p256.rs`'s module docstring apply transitively
//! and are not repeated here; what follows are the rules that are
//! specific to this generator.
//!
//! ## Hash function — FIPS 180-4 §6.2 / RFC 6234
//!
//! ECDSA with `SECP256R1_SHA256_RAW_RS64` hashes the message with
//! SHA-256 ([FIPS 180-4](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf),
//! also republished as [RFC 6234](https://www.rfc-editor.org/rfc/rfc6234)).
//! The resulting 256-bit digest is the integer `z` fed into ECDSA per
//! FIPS 186-5 §6.4.1:
//!
//! > 1. Use the selected hash function to compute `H = Hash(M)`.
//! >    The length of `H` must be at least equal to the length `n`
//! >    in bits.
//! > 2. Convert the leftmost `min(n_len, hashlen)` bits of `H` to an
//! >    integer `e`...
//!
//! For P-256 / SHA-256, `n_len = hashlen = 256` and the truncation is
//! a no-op: `z = int.from_be_bytes(SHA-256(message))`. That is exactly
//! what [`generate`] below computes via `Sha256::digest`.
//!
//! ## Fixed-width signature encoding — SEC 1 v2.0 §C.5
//!
//! [SEC 1 v2.0](https://www.secg.org/sec1-v2.pdf) §C.5 ("Octet-String
//! Encoding for ECDSA Signatures") permits the `R || S` raw encoding:
//!
//! > Alternatively, for some applications, it may be more convenient
//! > to encode an ECDSA signature as the concatenation of the octet
//! > strings R and S, each of length `log_256(n)` octets, padded with
//! > leading zero octets if necessary.
//!
//! For P-256, `log_256(n) = 32`, so the signature is 64 octets:
//! `R` as a 32-octet big-endian integer followed by `S` likewise.
//! [`sig_bytes`] and [`sig_field`] both emit that exact layout; the
//! "63-byte" / "65-byte" boundary fixtures inject off-by-one offsets
//! into that layout to exercise the verifier's strict-length rule.
//!
//! ## Identity-point rejection — SEC 1 v2.0 §2.3.3
//!
//! [SEC 1 v2.0 §2.3.3](https://www.secg.org/sec1-v2.pdf) ("Octet-String-
//! to-Elliptic-Curve-Point Conversion") gives the encoded form of the
//! point at infinity:
//!
//! > If P = O (the point at infinity), then the output M is the
//! > single octet 0x00.
//!
//! The `bad-key-identity.txt` fixture writes both encodings the
//! verifier might see (single-byte `0x00` and a 65-octet all-zero
//! string) and pins the expected `KeyResolutionFailure` outcome.

use std::path::Path;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;
use sha2::{Digest, Sha256};

use crate::acvp;
use crate::b64::urlsafe_unpadded;
use crate::p256;
use crate::util::{from_hex, hex_lower, write_bytes, write_text};
use crate::wire::{lendel, varint_field};

const ECDSA_ALG: u64 = 2;

const PAYLOAD: &[u8] = b"hello: world\n";

/// Pinned auditor-reproducible private key. Any non-zero `d < n` works
/// as long as the value is published; we pick the all-`0x42` pattern
/// so the constant is visually obvious in fixture hex dumps.
const D_HEX: &str = "4242424242424242424242424242424242424242424242424242424242424242";
/// Pinned ephemeral nonces for the happy-path and two-nonce-instability
/// fixtures. Distinct, non-zero, both `< n`.
const K1_HEX: &str = "CAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABE";
const K2_HEX: &str = "DEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF";

fn proto_artifact(payload: &[u8], signature: &[u8]) -> Vec<u8> {
    let mut inner = varint_field(1, ECDSA_ALG);
    inner.extend(lendel(3, signature));
    let mut out = lendel(1, payload);
    out.extend(lendel(2, &inner));
    out
}

fn yaml_artifact(payload: &[u8], signature: &[u8]) -> Vec<u8> {
    let sig_b64 = urlsafe_unpadded(signature);
    let pt = std::str::from_utf8(payload).expect("payload is ASCII");
    format!(
        "{pt}---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ECDSA_SECP256R1_SHA256_RAW_RS64\n\
         signature: {sig_b64}\n"
    )
    .into_bytes()
}

/// Standard `R || S` raw encoding (SEC 1 v2.0 §C.5).
fn sig_bytes(r: &BigInt, s: &BigInt) -> Vec<u8> {
    let mut out = p256::to_32_be(r);
    out.extend(p256::to_32_be(s));
    out
}

/// Same layout as [`sig_bytes`], but explicitly allowed to encode
/// integers that are zero or equal to `n` (the wire-rule rejects
/// `R = 0`, `S = 0`, `R = n`, `S = n` — these fixtures pin those
/// exact values for the boundary tests).
fn sig_field(r: &BigInt, s: &BigInt) -> Vec<u8> {
    let mut out = p256::to_32_be(r);
    out.extend(p256::to_32_be(s));
    out
}

fn parse_hex(s: &str) -> BigInt {
    BigInt::parse_bytes(s.as_bytes(), 16).expect("hex literal parses")
}

pub fn generate(dir: &Path) -> std::io::Result<()> {
    let d = parse_hex(D_HEX);
    let k1 = parse_hex(K1_HEX);
    let k2 = parse_hex(K2_HEX);
    let p = p256::params();

    assert!(d > BigInt::zero() && d < p.n);

    let q = p256::point_mul(&d, &p.g).expect("d*G non-identity");

    let mut pub_key = vec![0x04u8];
    pub_key.extend(p256::to_32_be(&q.x));
    pub_key.extend(p256::to_32_be(&q.y));
    assert_eq!(pub_key.len(), 65);

    println!("  pinned d:    {}", p256::hex64(&d));
    println!("  pinned Q:    {}", hex_lower(&pub_key));

    let hash = Sha256::digest(PAYLOAD);
    let z = BigInt::from_bytes_be(num_bigint::Sign::Plus, &hash);

    // Happy-path
    let (r1, s1) = p256::ecdsa_sign(&d, &z, &k1);
    assert!(p256::ecdsa_verify(&q, &z, &r1, &s1));
    let sig_happy = sig_bytes(&r1, &s1);
    write_bytes(
        dir,
        "verify-happy-path.binpb",
        &proto_artifact(PAYLOAD, &sig_happy),
    )?;
    write_bytes(
        dir,
        "verify-happy-path.yaml",
        &yaml_artifact(PAYLOAD, &sig_happy),
    )?;
    write_text(
        dir,
        "verify-happy-path.expected.txt",
        &format!(
            concat!(
                "# ECDSA P-256 / SHA-256 happy-path fixture.\n",
                "# private key d: {d_hex}\n",
                "# public key Q (uncompressed): {pub_hex}\n",
                "# payload (signed bytes): {payload_hex}  ({plen} bytes, 'hello: world\\n')\n",
                "# ephemeral k: {k1_hex}\n",
                "# R: {r_hex}\n# S: {s_hex}\n\n",
                "Verify(form=PROTOBUF, input=verify-happy-path.binpb,\n       ",
                "config.public_key_handle=<65-octet uncompressed PUB above>)\n",
                "Expected: Verified\n\n",
                "Verify(form=YAML, input=verify-happy-path.yaml,\n       ",
                "config.public_key_handle=<same 65-octet uncompressed PUB>)\n",
                "Expected: Verified\n",
            ),
            d_hex = p256::hex64(&d),
            pub_hex = hex_lower(&pub_key),
            payload_hex = hex_lower(PAYLOAD),
            plen = PAYLOAD.len(),
            k1_hex = p256::hex64(&k1),
            r_hex = p256::hex64(&r1),
            s_hex = p256::hex64(&s1),
        ),
    )?;

    // High-S / low-S
    let s_comp = (&p.n - &s1).mod_floor(&p.n);
    let half_n = &p.n / 2;
    let (high_s, low_s) = if s1 > half_n {
        (s1.clone(), s_comp.clone())
    } else {
        (s_comp.clone(), s1.clone())
    };
    assert!(p256::ecdsa_verify(&q, &z, &r1, &high_s));
    assert!(p256::ecdsa_verify(&q, &z, &r1, &low_s));
    write_bytes(
        dir,
        "high-s.binpb",
        &proto_artifact(PAYLOAD, &sig_bytes(&r1, &high_s)),
    )?;
    write_bytes(
        dir,
        "high-s.yaml",
        &yaml_artifact(PAYLOAD, &sig_bytes(&r1, &high_s)),
    )?;
    write_bytes(
        dir,
        "low-s.binpb",
        &proto_artifact(PAYLOAD, &sig_bytes(&r1, &low_s)),
    )?;
    write_bytes(
        dir,
        "low-s.yaml",
        &yaml_artifact(PAYLOAD, &sig_bytes(&r1, &low_s)),
    )?;
    write_text(
        dir,
        "high-s-low-s.expected.txt",
        &format!(
            concat!(
                "# High-S / low-S acceptance pair.\n",
                "# payload: {payload_hex}\n",
                "# public_key: {pub_hex}\n\n",
                "R (shared):  {r_hex}\n",
                "high-S:      {high_hex}\n",
                "low-S:       {low_hex}\n",
                "(low-S = n - high-S)\n\n",
                "Verify(*, input=high-s.binpb / .yaml, ...) -> Verified\n",
                "Verify(*, input=low-s.binpb  / .yaml, ...) -> Verified\n",
            ),
            payload_hex = hex_lower(PAYLOAD),
            pub_hex = hex_lower(&pub_key),
            r_hex = p256::hex64(&r1),
            high_hex = p256::hex64(&high_s),
            low_hex = p256::hex64(&low_s),
        ),
    )?;

    // Component-range rejection
    let zero = BigInt::zero();
    write_bytes(
        dir,
        "invalid-r-zero.binpb",
        &proto_artifact(PAYLOAD, &sig_field(&zero, &s1)),
    )?;
    write_bytes(
        dir,
        "invalid-s-zero.binpb",
        &proto_artifact(PAYLOAD, &sig_field(&r1, &zero)),
    )?;
    write_bytes(
        dir,
        "invalid-r-equals-n.binpb",
        &proto_artifact(PAYLOAD, &sig_field(&p.n, &s1)),
    )?;
    write_bytes(
        dir,
        "invalid-s-equals-n.binpb",
        &proto_artifact(PAYLOAD, &sig_field(&r1, &p.n)),
    )?;
    write_text(
        dir,
        "invalid-component-ranges.expected.txt",
        &format!(
            concat!(
                "# Range-rejection fixtures. Wire rule: 0 < R < n and 0 < S < n.\n",
                "# n (curve order): {n_hex}\n\n",
                "Verify(*, input=invalid-r-zero.binpb)      -> MalformedAttemptedSigned\n",
                "Verify(*, input=invalid-s-zero.binpb)      -> MalformedAttemptedSigned\n",
                "Verify(*, input=invalid-r-equals-n.binpb)  -> MalformedAttemptedSigned\n",
                "Verify(*, input=invalid-s-equals-n.binpb)  -> MalformedAttemptedSigned\n",
            ),
            n_hex = p256::hex64(&p.n),
        ),
    )?;

    // Non-fixed-width signatures
    let r_be = p256::to_32_be(&r1);
    let s_be = p256::to_32_be(&s1);
    let mut sig_63: Vec<u8> = r_be[1..].to_vec();
    sig_63.extend_from_slice(&s_be);
    let mut sig_65: Vec<u8> = vec![0x00];
    sig_65.extend_from_slice(&r_be);
    sig_65.extend_from_slice(&s_be);
    write_bytes(
        dir,
        "signature-63-bytes.binpb",
        &proto_artifact(PAYLOAD, &sig_63),
    )?;
    write_bytes(
        dir,
        "signature-65-bytes.binpb",
        &proto_artifact(PAYLOAD, &sig_65),
    )?;
    write_text(
        dir,
        "non-fixed-width.expected.txt",
        "# Non-fixed-width signature encodings.\n\n\
         Verify(*, input=signature-63-bytes.binpb) -> MalformedAttemptedSigned\n\
         Verify(*, input=signature-65-bytes.binpb) -> MalformedAttemptedSigned\n",
    )?;

    // Bad-key fixtures
    write_text(
        dir,
        "bad-key-identity.txt",
        &format!(
            concat!(
                "# Point at infinity (identity O) — MUST be rejected as KeyResolutionFailure.\n\n",
                "# SEC 1 §2.3.3 represents O as the single octet 0x00.\n",
                "Q-encoded-as-O-single-byte: 00\n\n",
                "# Some implementations might be presented with an all-zero 65-octet\n",
                "# string (04 || 0...0 || 0...0). This is also NOT a valid public key\n",
                "# and MUST be rejected.\n",
                "Q-encoded-all-zero-65: {zeros}\n",
            ),
            zeros = "00".repeat(65),
        ),
    )?;

    let bad_y = (&q.y + 1u32).mod_floor(&p.p);
    assert!(!p256::on_curve(&q.x, &bad_y));
    let mut bad_key_off = vec![0x04u8];
    bad_key_off.extend(p256::to_32_be(&q.x));
    bad_key_off.extend(p256::to_32_be(&bad_y));
    write_text(
        dir,
        "bad-key-off-curve.txt",
        &format!(
            concat!(
                "# Off-curve public key — MUST be rejected as KeyResolutionFailure.\n\n",
                "# (Qx, Qy+1 mod p) is not on secp256r1.\n",
                "public_key (hex): {hex}\n",
            ),
            hex = hex_lower(&bad_key_off),
        ),
    )?;

    let secp256k1_gx =
        parse_hex("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798");
    let secp256k1_gy =
        parse_hex("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8");
    let mut wrong_curve_key = vec![0x04u8];
    wrong_curve_key.extend(p256::to_32_be(&secp256k1_gx));
    wrong_curve_key.extend(p256::to_32_be(&secp256k1_gy));
    write_text(
        dir,
        "bad-key-wrong-curve.txt",
        &format!(
            concat!(
                "# Wrong-curve public key (secp256k1 generator) — MUST be rejected.\n\n",
                "# This point is on secp256k1, not secp256r1. A P-256 verifier\n",
                "# checking the curve equation will see y^2 != x^3 - 3x + b (P-256).\n",
                "public_key (hex): {hex}\n",
            ),
            hex = hex_lower(&wrong_curve_key),
        ),
    )?;

    // Two-nonce instability
    assert_ne!(k1, k2);
    let (r2, s2) = p256::ecdsa_sign(&d, &z, &k2);
    assert!(p256::ecdsa_verify(&q, &z, &r2, &s2));
    let sig_k1 = sig_bytes(&r1, &s1);
    let sig_k2 = sig_bytes(&r2, &s2);
    assert_ne!(sig_k1, sig_k2);

    write_bytes(
        dir,
        "two-nonce-instability-k1.binpb",
        &proto_artifact(PAYLOAD, &sig_k1),
    )?;
    write_bytes(
        dir,
        "two-nonce-instability-k2.binpb",
        &proto_artifact(PAYLOAD, &sig_k2),
    )?;
    write_text(
        dir,
        "two-nonce-instability.expected.txt",
        &format!(
            concat!(
                "# Deterministic-harness signature-octet instability.\n",
                "# Two signatures over the same (private key, payload) using two\n",
                "# explicitly chosen distinct nonces k1 != k2. Both verify; octets differ.\n\n",
                "# payload: {payload_hex}\n",
                "# private key d: {d_hex}\n",
                "# public key (uncompressed): {pub_hex}\n\n",
                "k1: {k1_hex}\nR1: {r1_hex}\nS1: {s1_hex}\n\n",
                "k2: {k2_hex}\nR2: {r2_hex}\nS2: {s2_hex}\n\n",
                "Verify(*, input=two-nonce-instability-k1.binpb) -> Verified\n",
                "Verify(*, input=two-nonce-instability-k2.binpb) -> Verified\n",
                "Octet equality:  signature(k1) != signature(k2)  (MUST hold)\n",
            ),
            payload_hex = hex_lower(PAYLOAD),
            d_hex = p256::hex64(&d),
            pub_hex = hex_lower(&pub_key),
            k1_hex = p256::hex64(&k1),
            r1_hex = p256::hex64(&r1),
            s1_hex = p256::hex64(&s1),
            k2_hex = p256::hex64(&k2),
            r2_hex = p256::hex64(&r2),
            s2_hex = p256::hex64(&s2),
        ),
    )?;

    write_text(
        dir,
        "algorithm-parameters-present.expected.txt",
        "# algorithm_parameters MUST be absent or zero-length for this algorithm.\n\n\
         Sign(alg=ECDSA_SECP256R1_SHA256_RAW_RS64,\n     \
         algorithm_parameters=b'\\x00')\n\
         -> SignerInvocationError(InvalidAlgorithmParameters)\n\n\
         Verify(..., config.algorithm_parameters=b'\\x00')\n\
         -> InvocationError(InvalidAlgorithmParameters)\n",
    )?;

    emit_acvp_anchored_fixture(dir)?;

    println!("  ecdsa fixtures written.");
    Ok(())
}

/// Emit a NIST-anchored happy-path fixture from the vendored
/// ACVP-Server ECDSA SigGen FIPS 186-5 test set.
///
/// Picks the first AFT (Algorithm Functional Test) case of the first
/// `curve = P-256 / hashAlg = SHA2-256` group. Our hand-rolled signer
/// replays `sign(d, SHA-256(message), k)` and we assert byte-equality
/// against the published `(r, s)` before writing the fixture — so
/// fixture generation is itself a NIST-vector conformance check.
fn emit_acvp_anchored_fixture(dir: &Path) -> std::io::Result<()> {
    let file = acvp::load();
    let group = acvp::p256_sha256_aft_groups(&file)
        .next()
        .expect("vendored ACVP file has at least one P-256 / SHA-256 AFT group");
    let case = group
        .tests
        .first()
        .expect("AFT group has at least one test case");

    let d = parse_hex(&group.d);
    let qx = parse_hex(&group.qx);
    let qy = parse_hex(&group.qy);
    let k = parse_hex(&case.k);
    let message = from_hex(&case.message);
    let expected_r = parse_hex(&case.r);
    let expected_s = parse_hex(&case.s);

    let z = BigInt::from_bytes_be(num_bigint::Sign::Plus, &Sha256::digest(&message));
    let (r, s) = p256::ecdsa_sign(&d, &z, &k);
    assert_eq!(
        r, expected_r,
        "ACVP tcId {}: sign produced wrong R",
        case.tc_id
    );
    assert_eq!(
        s, expected_s,
        "ACVP tcId {}: sign produced wrong S",
        case.tc_id
    );

    let q = p256::Point { x: qx, y: qy };
    assert!(p256::ecdsa_verify(&q, &z, &r, &s));

    let mut pub_key = vec![0x04u8];
    pub_key.extend(p256::to_32_be(&q.x));
    pub_key.extend(p256::to_32_be(&q.y));

    let signature = sig_bytes(&r, &s);
    let stem = format!("acvp-fips186-5-p256-sha256-tc{tc}", tc = case.tc_id);

    write_bytes(
        dir,
        &format!("{stem}.binpb"),
        &proto_artifact(&message, &signature),
    )?;
    write_text(
        dir,
        &format!("{stem}.expected.txt"),
        &format!(
            concat!(
                "# NIST ACVP-Server FIPS 186-5 ECDSA SigGen AFT vector.\n",
                "# Source:        https://github.com/{repo} @ {commit}\n",
                "# Upstream path: {path}\n",
                "# (see vendor/acvp/README.md for the manual-verification hash steps)\n",
                "#\n",
                "# tgId:        {tg_id}\n",
                "# tcId:        {tc_id}\n",
                "# curve:       P-256\n",
                "# hashAlg:     SHA2-256\n",
                "# testType:    AFT\n",
                "#\n",
                "# d:           {d_hex}\n",
                "# Q (uncompressed, hex): {pub_hex}\n",
                "# k:           {k_hex}\n",
                "# message ({mlen} bytes, hex):\n",
                "#   {msg_hex}\n",
                "# R:           {r_hex}\n",
                "# S:           {s_hex}\n",
                "#\n",
                "# The rebuilder replays sign(d, SHA-256(message), k) and asserts\n",
                "# byte-equality against the (R, S) above before writing the\n",
                "# .binpb. The fixture is therefore a NIST-anchored conformance\n",
                "# artifact, not a locally-generated value.\n",
                "\n",
                "Verify(form=PROTOBUF, input={stem}.binpb,\n",
                "       config.public_key_handle=<65-octet uncompressed PUB above>)\n",
                "Expected: Verified\n",
            ),
            repo = "usnistgov/ACVP-Server",
            commit = acvp::VENDORED_COMMIT,
            path = acvp::VENDORED_PATH,
            tg_id = group.tg_id,
            tc_id = case.tc_id,
            d_hex = p256::hex64(&d),
            pub_hex = hex_lower(&pub_key),
            k_hex = p256::hex64(&k),
            mlen = message.len(),
            msg_hex = hex_lower(&message),
            r_hex = p256::hex64(&r),
            s_hex = p256::hex64(&s),
            stem = stem,
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    /// Pinned constants must decode as positive integers in `[1, n)`.
    #[test]
    fn pinned_scalars_are_in_range() {
        let p = p256::params();
        for hex in &[D_HEX, K1_HEX, K2_HEX] {
            let x = parse_hex(hex);
            assert!(x > BigInt::zero());
            assert!(x < p.n);
        }
    }

    /// SHA-256 of the empty string is the published value
    /// `e3b0c442 98fc1c14 9afbf4c8 996fb924 27ae41e4 649b934c a495991b 7852b855`
    /// (FIPS 180-4 §C). Tests the hash dependency, not just our code.
    #[test]
    fn sha256_empty_known_answer() {
        let got = hex_lower(&Sha256::digest([]));
        assert_eq!(
            got,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// secp256k1's generator must NOT satisfy the secp256r1 curve
    /// equation (otherwise the `bad-key-wrong-curve` fixture would
    /// be a happy-path key).
    #[test]
    fn secp256k1_generator_is_off_secp256r1() {
        let gx = parse_hex("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798");
        let gy = parse_hex("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8");
        assert!(!p256::on_curve(&gx, &gy));
    }

    /// Replay every P-256 / SHA-256 AFT case from the vendored
    /// ACVP-Server file: run `sign(d, SHA-256(message), k)` through
    /// our hand-rolled signer and assert byte-equality with the
    /// published `(r, s)`. This is the safety net behind the
    /// NIST-anchored `acvp-fips186-5-*` conformance fixture — if our
    /// signer ever drifts from NIST's reference output, the test
    /// fails before any fixture is written.
    #[test]
    fn p256_sha256_acvp_aft_replay_matches() {
        let file = crate::acvp::load();
        let mut total = 0usize;
        for group in crate::acvp::p256_sha256_aft_groups(&file) {
            let d = parse_hex(&group.d);
            for case in &group.tests {
                let k = parse_hex(&case.k);
                let message = from_hex(&case.message);
                let expected_r = parse_hex(&case.r);
                let expected_s = parse_hex(&case.s);
                let z = BigInt::from_bytes_be(num_bigint::Sign::Plus, &Sha256::digest(&message));
                let (r, s) = p256::ecdsa_sign(&d, &z, &k);
                assert_eq!(
                    r, expected_r,
                    "tgId {}, tcId {}: R mismatch",
                    group.tg_id, case.tc_id
                );
                assert_eq!(
                    s, expected_s,
                    "tgId {}, tcId {}: S mismatch",
                    group.tg_id, case.tc_id
                );
                total += 1;
            }
        }
        assert!(
            total > 0,
            "vendored ACVP file produced no P-256/SHA-256 AFT cases"
        );
    }
}
