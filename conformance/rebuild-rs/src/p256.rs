// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Hand-rolled P-256 (secp256r1) ECDSA over `num-bigint`.
//!
//! No crypto-library trust — the only outside math is `num-bigint`'s
//! arbitrary-precision integer type. Every constant and every
//! arithmetic step is derived directly from the cited upstream
//! standards.
//!
//! ## Domain parameters — Standards for Efficient Cryptography 2 (SEC 2)
//!
//! [Standards for Efficient Cryptography 2 (SEC 2), Version 2.0](https://www.secg.org/sec2-v2.pdf)
//! §2.4.2 ("Recommended
//! Parameters secp256r1") publishes the P-256 (a.k.a. secp256r1, a.k.a.
//! NIST P-256) domain parameters. Reproduced verbatim:
//!
//! > The verifiably random elliptic curve domain parameters over Fp
//! > `secp256r1` are specified by the sextuple T = (p, a, b, G, n, h)
//! > where the finite field Fp is defined by:
//! >
//! > p =  FFFFFFFF 00000001 00000000 00000000 00000000 FFFFFFFF FFFFFFFF FFFFFFFF
//! >
//! > The curve E: y^2 = x^3 + ax + b over Fp is defined by:
//! >
//! > a =  FFFFFFFF 00000001 00000000 00000000 00000000 FFFFFFFF FFFFFFFF FFFFFFFC
//! > b =  5AC635D8 AA3A93E7 B3EBBD55 769886BC 651D06B0 CC53B0F6 3BCE3C3E 27D2604B
//! >
//! > The base point G in compressed form is:
//! >
//! > G =  03 6B17D1F2 E12C4247 F8BCE6E5 63A440F2 77037D81 2DEB33A0 F4A13945 D898C296
//! >
//! > and in uncompressed form is:
//! >
//! > G = 04 6B17D1F2 E12C4247 F8BCE6E5 63A440F2 77037D81 2DEB33A0 F4A13945 D898C296
//! >        4FE342E2 FE1A7F9B 8EE7EB4A 7C0F9E16 2BCE3357 6B315ECE CBB64068 37BF51F5
//! >
//! > Finally the order n of G and the cofactor are:
//! >
//! > n = FFFFFFFF 00000000 FFFFFFFF FFFFFFFF BCE6FAAD A7179E84 F3B9CAC2 FC632551
//! > h = 01
//!
//! These are the exact values loaded by [`params`] below. Note `a` is
//! given as `p - 3`; we store it as `(-3) mod p`, which equals that
//! number.
//!
//! ## Point operations — Standards for Efficient Cryptography 1 (SEC 1)
//!
//! [Standards for Efficient Cryptography 1 (SEC 1), Version 2.0](https://www.secg.org/sec1-v2.pdf)
//! §2.2.1 ("Elliptic
//! Curves over Fp") gives the addition and doubling formulae:
//!
//! > Let P1 = (x1, y1) ∈ E(Fp) and P2 = (x2, y2) ∈ E(Fp), where
//! > P1, P2 ≠ ∞. Then:
//! >
//! > 1. (x1, y1) + ∞ = ∞ + (x1, y1) = (x1, y1).
//! > 2. (x1, y1) + (x1, −y1) = ∞.
//! > 3. If P1 ≠ ±P2,  P1 + P2 = (x3, y3) where
//! >       x3 = λ^2 − x1 − x2  mod p
//! >       y3 = λ(x1 − x3) − y1  mod p
//! >       λ = (y2 − y1)/(x2 − x1)  mod p.
//! > 4. \[2\]P1 = (x3, y3) where
//! >       x3 = λ^2 − 2 x1  mod p
//! >       y3 = λ(x1 − x3) − y1  mod p
//! >       λ = (3 x1^2 + a) / (2 y1)  mod p.
//!
//! These are the exact formulae implemented in [`point_add`].
//!
//! ## ECDSA sign / verify — FIPS 186-5 §6.4
//!
//! ECDSA signature generation and verification follow the standard
//! formulation in [FIPS 186-5](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-5.pdf)
//! §6.4 (also matching SEC 1 v2.0 §4.1):
//!
//! - **Sign(d, z, k):** compute `R = k·G`, set `r = R.x mod n`. If
//!   `r = 0` retry (caller is responsible; this crate's fixtures pin
//!   `k` so the case is impossible and we `assert!`). Then compute
//!   `s = k^{-1} (z + r·d) mod n`. If `s = 0` retry.
//! - **Verify(Q, z, r, s):** reject if `r` or `s` is outside
//!   `[1, n-1]`. Otherwise compute `w = s^{-1} mod n`,
//!   `u1 = z·w mod n`, `u2 = r·w mod n`, and `R' = u1·G + u2·Q`.
//!   Accept iff `R' ≠ ∞` and `R'.x mod n = r`.
//!
//! [`ecdsa_sign`] and [`ecdsa_verify`] below are direct transcriptions.
//! Modular inverse is computed via Fermat's little theorem — `a^{p-2}
//! mod p` is the inverse over a prime modulus, and both `p` and `n`
//! are prime for secp256r1 (SEC 2 §2.4.2 above).

use std::sync::OnceLock;

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::Zero;

/// secp256r1 domain parameters, lazily initialised on first access.
pub struct Params {
    pub p: BigInt,
    pub a: BigInt,
    pub b: BigInt,
    pub n: BigInt,
    pub g: Point,
}

/// Affine point `(x, y)` on the curve. The point at infinity is
/// represented as `None` outside this struct.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Point {
    pub x: BigInt,
    pub y: BigInt,
}

fn h(s: &str) -> BigInt {
    BigInt::parse_bytes(s.as_bytes(), 16).expect("hex literal parses")
}

/// Singleton domain-parameter accessor. Constants below are the
/// verbatim hex strings from SEC 2 v2.0 §2.4.2 with whitespace
/// removed.
pub fn params() -> &'static Params {
    static PARAMS: OnceLock<Params> = OnceLock::new();
    PARAMS.get_or_init(|| {
        let p = h("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff");
        // SEC 2 prints `a` as `p - 3`; we store it as `(-3) mod p`.
        let a = (BigInt::from(-3) % &p + &p) % &p;
        let b = h("5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b");
        let n = h("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551");
        let gx = h("6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296");
        let gy = h("4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5");
        Params {
            p,
            a,
            b,
            n,
            g: Point { x: gx, y: gy },
        }
    })
}

/// Modular inverse over a prime modulus via Fermat's little theorem
/// (`a^{m-2} mod m`). Used for both `Fp` and the scalar field `Fn`;
/// both moduli are prime for secp256r1.
fn inv(a: &BigInt, m: &BigInt) -> BigInt {
    let a_mod = a.mod_floor(m);
    let exp = m - 2;
    a_mod.modpow(&exp, m)
}

fn mod_p(x: BigInt) -> BigInt {
    let p = &params().p;
    x.mod_floor(p)
}

/// Affine point addition / doubling, per the SEC 1 §2.2.1 formulae
/// quoted in the module docstring. `None` is the point at infinity.
pub fn point_add(p1: Option<&Point>, p2: Option<&Point>) -> Option<Point> {
    match (p1, p2) {
        (None, q) => q.cloned(),
        (q, None) => q.cloned(),
        (Some(a), Some(b)) => {
            let p = &params().p;
            let curve_a = &params().a;
            // Case (2): P + (-P) = ∞.
            if a.x == b.x && (&a.y + &b.y).mod_floor(p).is_zero() {
                return None;
            }
            let m = if a == b {
                // Doubling (case 4): λ = (3 x^2 + a) / (2 y) mod p.
                let num = (BigInt::from(3) * &a.x * &a.x + curve_a).mod_floor(p);
                let den = (BigInt::from(2) * &a.y).mod_floor(p);
                mod_p(num * inv(&den, p))
            } else {
                // Distinct add (case 3): λ = (y2 - y1) / (x2 - x1) mod p.
                let num = (&b.y - &a.y).mod_floor(p);
                let den = (&b.x - &a.x).mod_floor(p);
                mod_p(num * inv(&den, p))
            };
            let x3 = mod_p(&m * &m - &a.x - &b.x);
            let y3 = mod_p(&m * (&a.x - &x3) - &a.y);
            Some(Point { x: x3, y: y3 })
        }
    }
}

/// Scalar multiplication via the standard double-and-add ladder.
/// Per SEC 1 §3.2.2.1 ("Elliptic Curve Domain Parameters Validation"),
/// any positive scalar `k < n` is admissible.
pub fn point_mul(k: &BigInt, point: &Point) -> Option<Point> {
    let mut result: Option<Point> = None;
    let mut addend: Option<Point> = Some(point.clone());
    let mut kk = k.clone();
    while !kk.is_zero() {
        if kk.is_odd() {
            result = point_add(result.as_ref(), addend.as_ref());
        }
        addend = point_add(addend.as_ref(), addend.as_ref());
        kk >>= 1;
    }
    result
}

/// Curve-equation check: returns true iff `(x, y)` satisfies
/// `y^2 ≡ x^3 + ax + b (mod p)`. Used by [`crate::alg_ecdsa`] to
/// confirm the off-curve fixture really is off-curve.
pub fn on_curve(x: &BigInt, y: &BigInt) -> bool {
    let p = &params().p;
    let a = &params().a;
    let b = &params().b;
    let lhs = (y * y).mod_floor(p);
    let rhs = (x * x * x + a * x + b).mod_floor(p);
    lhs == rhs
}

/// ECDSA signature generation, FIPS 186-5 §6.4 / SEC 1 §4.1.3.
///
/// Caller pins `(d, z, k)`; this crate's fixtures all use
/// caller-pinned `k` so the `r = 0` / `s = 0` retry branches cannot
/// fire and are asserted instead.
pub fn ecdsa_sign(d: &BigInt, z: &BigInt, k: &BigInt) -> (BigInt, BigInt) {
    let n = &params().n;
    let r_point = point_mul(k, &params().g).expect("k*G non-identity");
    let r = r_point.x.mod_floor(n);
    assert!(!r.is_zero());
    let s = (inv(k, n) * (z + &r * d)).mod_floor(n);
    assert!(!s.is_zero());
    (r, s)
}

/// ECDSA verification, FIPS 186-5 §6.4 / SEC 1 §4.1.4.
pub fn ecdsa_verify(q: &Point, z: &BigInt, r: &BigInt, s: &BigInt) -> bool {
    let n = &params().n;
    let zero = BigInt::zero();
    if !(r > &zero && r < n && s > &zero && s < n) {
        return false;
    }
    let w = inv(s, n);
    let u1 = (z * &w).mod_floor(n);
    let u2 = (r * &w).mod_floor(n);
    let rp = point_add(
        point_mul(&u1, &params().g).as_ref(),
        point_mul(&u2, q).as_ref(),
    );
    match rp {
        None => false,
        Some(rp) => rp.x.mod_floor(n) == *r,
    }
}

/// 32-octet big-endian encoding of `x`, masked to the low 256 bits.
/// Per SEC 1 §2.3.5, integers in fixed-width encodings are written
/// most-significant-octet first; this function matches that rule.
pub fn to_32_be(x: &BigInt) -> Vec<u8> {
    let (_, bytes) = x.to_bytes_be();
    let mut out = vec![0u8; 32];
    if bytes.len() >= 32 {
        out.copy_from_slice(&bytes[bytes.len() - 32..]);
    } else {
        out[32 - bytes.len()..].copy_from_slice(&bytes);
    }
    out
}

/// Decode a big-endian octet string to a non-negative `BigInt`,
/// per SEC 1 §2.3.6 ("Octet-String-to-Integer Conversion").
#[allow(dead_code)]
pub fn from_be(bytes: &[u8]) -> BigInt {
    BigInt::from_bytes_be(Sign::Plus, bytes)
}

/// Lower-case 64-char hex of the 256-bit big-endian encoding.
pub fn hex64(x: &BigInt) -> String {
    let bytes = to_32_be(x);
    super::util::hex_lower(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::One;

    /// Generator `G` MUST satisfy the curve equation (SEC 2 §2.4.2).
    #[test]
    fn generator_is_on_curve() {
        let p = params();
        assert!(on_curve(&p.g.x, &p.g.y));
    }

    /// `n · G = ∞` is the defining property of `n` (the order of `G`).
    #[test]
    fn group_order_annihilates_generator() {
        let p = params();
        assert!(point_mul(&p.n, &p.g).is_none());
    }

    /// Doubling identity: `G + G = 2·G`.
    #[test]
    fn doubling_equals_self_add() {
        let p = params();
        let two = BigInt::from(2);
        let g = Some(p.g.clone());
        let doubled = point_add(g.as_ref(), g.as_ref());
        let mul = point_mul(&two, &p.g);
        assert_eq!(doubled, mul);
    }

    /// ECDSA sign / verify round-trip with a pinned key + nonce.
    #[test]
    fn ecdsa_sign_verify_roundtrip() {
        let p = params();
        let d = BigInt::from(7u32);
        let z = BigInt::from(0x12345678u64);
        let k = BigInt::from(11u32);
        let (r, s) = ecdsa_sign(&d, &z, &k);
        let q = point_mul(&d, &p.g).expect("d*G defined");
        assert!(ecdsa_verify(&q, &z, &r, &s));
        // Tampered `s` MUST be rejected.
        let bad_s = (&s + 1u32).mod_floor(&p.n);
        assert!(!ecdsa_verify(&q, &z, &r, &bad_s));
    }

    /// Range checks: r or s outside (0, n) MUST fail verification
    /// before any expensive math runs.
    #[test]
    fn ecdsa_verify_rejects_out_of_range_components() {
        let p = params();
        let q = p.g.clone();
        let z = BigInt::one();
        let zero = BigInt::zero();
        let n = p.n.clone();
        assert!(!ecdsa_verify(&q, &z, &zero, &BigInt::one()));
        assert!(!ecdsa_verify(&q, &z, &BigInt::one(), &zero));
        assert!(!ecdsa_verify(&q, &z, &n, &BigInt::one()));
        assert!(!ecdsa_verify(&q, &z, &BigInt::one(), &n));
    }

    /// `to_32_be` produces fixed-width 32-octet big-endian output.
    #[test]
    fn to_32_be_pads_and_truncates() {
        assert_eq!(to_32_be(&BigInt::zero()), vec![0u8; 32]);
        let one = to_32_be(&BigInt::one());
        assert_eq!(one.len(), 32);
        assert_eq!(one[31], 1);
        assert!(one[..31].iter().all(|&b| b == 0));
    }
}
