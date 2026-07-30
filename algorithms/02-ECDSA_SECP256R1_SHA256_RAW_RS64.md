# ECDSA_SECP256R1_SHA256_RAW_RS64

This algorithm is ECDSA over secp256r1, also known as NIST P-256, with
SHA-256 over the payload byte string, a non-deterministic ephemeral nonce per
[FIPS 186-5 section 6.3](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-5.pdf),
and a fixed 64-octet raw `R || S` signature layout.
The cited NIST and Standards for Efficient Cryptography material is
third-party material, not material relicensed under Apache-2.0.
[Third-Party Notices](../THIRD_PARTY_NOTICES.md) records the exact sources,
terms, and intellectual-property caveats.

| Identity | Value |
| --- | --- |
| Protobuf enum | `ALGORITHM_ECDSA_SECP256R1_SHA256_RAW_RS64` value `2`. |
| YAML `alg` | `ECDSA_SECP256R1_SHA256_RAW_RS64`. |

## Profile

| Property | Rule |
| --- | --- |
| Scheme | ECDSA per FIPS 186-5 section 6. |
| Curve | secp256r1 / NIST P-256. |
| Hash | SHA-256. |
| Nonce strategy | Non-deterministic per FIPS 186-5 section 6.3. |
| Signature layout | `R` (32 octets) followed by `S` (32 octets), fixed-width big-endian. |
| Component range | `0 < R < n` and `0 < S < n`. |
| S-component policy | High-S accepted. Low-S normalization is optional for signers. |
| Public-key encoding | Uncompressed *Standards for Efficient Cryptography 1 (SEC 1)* point only: 65 octets, `0x04 || X || Y`. |
| `algorithm_parameters` | No parameters. Absent or zero-length only. |
| Signature octet identity | Not stable. Repeated signing MAY produce different signature octets. |

## Inputs

The signed message is the payload octets exactly as the artifact carries them.
Implementations MUST NOT canonicalize YAML, add framing, or add a domain
separator. ECDSA hashes the payload once with SHA-256. Empty payload is valid.

`keyid` handling is defined by the overall specification, not by this
algorithm.

## Key Material

Private key material is an integer `d` with `1 <= d <= n - 1`, generated per
FIPS 186-5 or SEC 1 with CSPRNG-quality randomness.

The verification public key crossing this slot interface MUST use the
65-octet uncompressed SEC 1 form `0x04 || X || Y`, where `X` and `Y` are
32-octet big-endian fixed-width field elements. Compressed and hybrid point
forms are not accepted at the slot input, output, or conformance-fixture
surface.

Before verification, the public key `Q` MUST:

- Decode as the declared uncompressed point form.
- Lie on secp256r1.
- Not be the point at infinity.
- Satisfy `[n]Q = O`.

Failures in public-key resolution or admissibility are
`KeyResolutionFailure`, because the failing bytes are caller-supplied key
material, not artifact bytes.

## Signing

1. Compute `H = SHA-256(payload)`.
2. Generate ephemeral nonce `k` per FIPS 186-5 section 6.3 using a CSPRNG.
3. Compute `(R, S)` per FIPS 186-5 section 6.4.
4. Restart with a fresh `k` if `R = 0` or `S = 0`.
5. Encode `signature = R || S` per the wire rules below.
6. Place the 64-octet signature in `YamlSigilSignature.signature`. YAML form
   base64-encodes those bytes per [Base64 Requirements](../base64-requirements.md).

Signers MAY normalize `S` to low-S. Verifiers MUST NOT depend on that
normalization.

This algorithm defines no parameters. A non-empty `algorithm_parameters` value
is `InvalidAlgorithmParameters` for both signer and verifier APIs.

## Signature Octets

After YAML base64 decoding, or directly in protobuf form, the signature MUST
satisfy:

| Rule | Requirement |
| --- | --- |
| Length | Exactly 64 octets. |
| Layout | `R` (octets 0 through 31), then `S` (octets 32 through 63). |
| Component encoding | Unsigned big-endian, left-padded with `0x00` to exactly 32 octets. |
| Range | `0 < R < n` and `0 < S < n`. |

Wrong length, out-of-range components, non-fixed-width encoding, or YAML
base64 decode failure produces `MalformedAttemptedSigned`.

## Verification

1. Resolve and validate public key `Q`.
2. Recover and validate signature octets per the wire rules above.
3. Compute `H = SHA-256(payload)`.
4. Verify `(R, S)` over `H` per FIPS 186-5 section 6.4.

Cryptographic success returns `Verified`. Cryptographic mismatch returns
`SignedButFailedVerification`.

Verifiers MUST accept any ECDSA-valid `(R, S)`, including both `S` and
`n - S`. High-S rejection is not conforming for this algorithm.

Because the nonce is randomized and high-S signatures are accepted, callers
MUST NOT rely on byte equality of signature octets across signing operations.

## Exclusions

This identifier is not ASN.1 DER ECDSA, RFC 6979 deterministic ECDSA, JOSE,
JWS, JWT, COSE, or a key-recovery mechanism. A different behavior needs a
different algorithm identifier.

## Conformance Fixtures

Fixtures for this algorithm live under
[`conformance/alg-ecdsa/`](../conformance/alg-ecdsa/). The fixture set SHOULD
cover:

- NIST CAVP or ACVP ECDSA P-256 vectors.
- Empty payload signing and verification.
- Multi-document YAML payload bytes.
- High-S and low-S acceptance for equivalent signatures.
- `R = 0`, `S = 0`, `R = n`, and `S = n` failures.
- Non-fixed-width signature encodings.
- Invalid configured public keys.
- Signature octet instability across distinct nonces.
- Rejection of non-empty `algorithm_parameters`.

## References

- [FIPS 180-4](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf).
- [FIPS 186-5](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-5.pdf).
- [NIST CAVP Digital Signatures](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/digital-signatures).
- [NIST ACVP](https://pages.nist.gov/ACVP/).
- [*Standards for Efficient Cryptography 1 (SEC 1): Elliptic Curve Cryptography*, Version 2.0](https://www.secg.org/sec1-v2.pdf).
- [*Standards for Efficient Cryptography 2 (SEC 2): Recommended Elliptic Curve Domain Parameters*, Version 2.0](https://www.secg.org/sec2-v2.pdf).
- [RFC 7518 section 3.4](https://www.rfc-editor.org/rfc/rfc7518#section-3.4).
- [Base64 Requirements](../base64-requirements.md).
- [Third-Party Notices](../THIRD_PARTY_NOTICES.md).
