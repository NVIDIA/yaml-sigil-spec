# ED25519_PUREEDDSA_RAW_RS64_CANONICAL

This algorithm is Ed25519 PureEdDSA per
[RFC 8032 section 5.1](https://www.rfc-editor.org/rfc/rfc8032#section-5.1),
over the payload byte string, with the RFC 8032 64-octet raw `R || S`
signature layout and strict canonical-encoding verification from
["Taming the Many EdDSAs"](https://eprint.iacr.org/2020/1244) Algorithm 2.
RFC 8032 and "Taming the Many EdDSAs" are third-party material, not material
relicensed under Apache-2.0. [Third-Party Notices](../THIRD_PARTY_NOTICES.md)
records their source terms and identifies the adaptations made here.

| Identity | Value |
| --- | --- |
| Protobuf enum | `ALGORITHM_ED25519_PUREEDDSA_RAW_RS64_CANONICAL` value `1`. |
| YAML `alg` | `ED25519_PUREEDDSA_RAW_RS64_CANONICAL`. |

## Profile

| Property | Rule |
| --- | --- |
| Scheme | Pure EdDSA per RFC 8032 section 5.1. |
| Curve | edwards25519. |
| Internal hash | SHA-512 as mandated by RFC 8032. |
| Pre-hash or context | None. This is not Ed25519ph or Ed25519ctx. |
| Nonce strategy | Deterministic per RFC 8032 section 5.1.6. |
| Signature layout | `R` (32 octets) followed by `S` (32 octets), exactly 64 octets. |
| Public-key encoding | 32-octet compressed Edwards-y per RFC 8032 section 5.1.2. |
| Verification equation | Cofactored equation only. Cofactorless verification is not permitted. |
| `algorithm_parameters` | No parameters. Absent or zero-length only. |
| Signature octet identity | Stable. Repeated signing of the same `(seed, payload)` produces identical signature octets. |

## Inputs

The signed message is the payload octets exactly as the artifact carries them.
Implementations MUST NOT canonicalize YAML, add framing, add a domain
separator, or apply an application-layer pre-hash. Empty payload is valid.

`keyid` handling is defined by the overall specification, not by this
algorithm.

## Key Material

Private key material is a 32-octet seed per RFC 8032 section 5.1.5. Seed
generation MUST use a CSPRNG. The expanded form MAY be cached for
performance, but the seed is the canonical at-rest form.

The verification public key is the 32-octet compressed Edwards-y encoding of
`A = [s]B` per RFC 8032 section 5.1.2. Key resolution is governed by
`config.public_key_handle` and local trust policy.

Before verification, the public key `A` MUST:

- Decode as a canonical Edwards-y point.
- Satisfy the field bound for the encoded y-coordinate.
- Not be a small-order point.

Failures in public-key resolution or admissibility are
`KeyResolutionFailure`, because the failing bytes are caller-supplied key
material, not artifact bytes.

## Signing

1. Take payload octets `M` exactly.
2. Compute `signature = R || S` per RFC 8032 section 5.1.6.
3. Place the 64-octet signature in `YamlSigilSignature.signature`. YAML form
   base64-encodes those bytes per [Base64 Requirements](../base64-requirements.md).

Signing is deterministic. Hedged or randomized Ed25519 implementations MUST
NOT advertise this algorithm identifier.

This algorithm defines no parameters. A non-empty `algorithm_parameters` value
is `InvalidAlgorithmParameters` for both signer and verifier APIs.

## Signature Octets

After YAML base64 decoding, or directly in protobuf form, the signature MUST
satisfy:

| Rule | Requirement |
| --- | --- |
| Length | Exactly 64 octets. |
| Layout | `R` (octets 0 through 31), then `S` (octets 32 through 63). |
| `R` | Canonical compressed Edwards-y encoding that decodes to a valid curve point. |
| `S` | Little-endian 32-octet integer with `0 <= S < L`. |

`L` is the edwards25519 group order:
`2^252 + 27742317777372353535851937790883648493`.

Wrong length, non-canonical `R`, non-canonical `S`, `S >= L`, or YAML base64
decode failure produces `MalformedAttemptedSigned`.

The YAML-form signature scalar is 86 URL-safe unpadded base64 characters for a
valid 64-octet signature. No line wrapping is permitted inside the scalar.

## Verification

1. Recover and validate signature octets per the wire rules above.
2. Split `R` and `S`.
3. Resolve and validate public key `A`.
4. Compute `k = SHA-512(R || A || M) mod L`.
5. Verify the cofactored equation
   `[8][S]B = [8]R + [8][k]A`.

The cofactorless equation `[S]B = R + [k]A` is not permitted for this slot.
The slot permits canonical `R` values outside the prime-order subgroup, and
the cofactored equation keeps verdicts deterministic for those inputs.

Cryptographic success returns `Verified`. Cryptographic mismatch returns
`SignedButFailedVerification`.

Implementations MAY use batch verification, but each signature MUST satisfy
the canonical-encoding checks independently before the aggregate verdict is
used.

## Exclusions

This identifier is not Ed25519ph, Ed25519ctx, hedged Ed25519, randomized
Ed25519, ZIP-215, JOSE, JWS, JWT, COSE, or a key-recovery mechanism. A
different behavior needs a different algorithm identifier.

## Conformance Fixtures

Fixtures for this algorithm live under
[`conformance/alg-ed25519/`](../conformance/alg-ed25519/). The fixture set
SHOULD cover:

- RFC 8032 section 7.1 vectors, including the empty-message vector.
- YAML-form and protobuf-form signature encoding.
- Multi-document YAML payload bytes.
- Non-canonical `R` and `S` artifact failures.
- Small-order or non-canonical configured public-key failures.
- Signature octet stability across repeated signing.
- Rejection of non-empty `algorithm_parameters`.

## References

- [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032).
- [RFC 7748](https://www.rfc-editor.org/rfc/rfc7748).
- [FIPS 186-5](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-5.pdf).
- ["Taming the Many EdDSAs"](https://eprint.iacr.org/2020/1244).
- [Base64 Requirements](../base64-requirements.md).
- [Third-Party Notices](../THIRD_PARTY_NOTICES.md).
