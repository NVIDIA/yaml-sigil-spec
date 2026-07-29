# ECDSA (`ECDSA_SECP256R1_SHA256_RAW_RS64`) Conformance Fixtures

Drives the
[ECDSA algorithm specification](../../algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md).

## Upstream sources

These fixtures combine canonical curve / algorithm parameters from
published standards with a locally-pinned test key + nonces. See the
top-level [conformance README](../README.md) `#### Compromise (ECDSA)`
subsection for the locally-generated caveat.

- [FIPS 186-5](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-5.pdf) —
  ECDSA (§6); nonce generation (§6.3); signature generation and verification
  (§6.4); CSPRNG requirements (§A.2).
- [*Standards for Efficient Cryptography 1 (SEC 1): Elliptic Curve
  Cryptography*, Version 2.0](https://www.secg.org/sec1-v2.pdf) — point
  encoding (§2.3.3), point decoding (§2.3.4), public-key admissibility
  (§3.2.2), ECDSA (§4.1), fixed-width `R ‖ S` (§C.5).
- [*Standards for Efficient Cryptography 2 (SEC 2): Recommended Elliptic Curve
  Domain Parameters*, Version 2.0](https://www.secg.org/sec2-v2.pdf) —
  secp256r1 / NIST P-256 domain parameters (`P`, `A`, `B`, `N`, `Gx`, `Gy`);
  copied verbatim into the generator.
- [NIST CAVP — Digital Signatures](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/digital-signatures)
  and [NIST ACVP](https://pages.nist.gov/ACVP/) — official P-256 test vectors.
  **NOT pinned in this directory.** Auditors targeting FIPS conformance MUST
  additionally run against the CAVP / ACVP vector files.
- [`algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md`](../../algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md) — slot specification.

The SEC-derived operations, encodings, and parameters are third-party
standards material, not material relicensed under Apache-2.0. The applicable
source notices and patent/IP caveats are recorded in
[`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md). Binary and signed
YAML fixtures remain exact-byte inputs, so their provenance lives here, in
safe text sidecars, and in the generator source.

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the
Docker build and run commands. On a steady-state branch,
re-running the rebuilder MUST reproduce the bytes of every
fixture in this directory bit-identically; a non-empty diff
is either a generator defect or an intended spec change that
has propagated through the generator.

## How to use

For each fixture, the `*.binpb` is a serialized `SignedYamlArtifact`
ready for `Verify(form = TRANSCRIPTION_FORM_PROTOBUF, …)`. The matching
`*.yaml` carries the same `(payload, alg, signature)` content under
the YAML form. Per-fixture `expected.txt` files name both the expected
verifier-state outcome and the verification-key / nonce inputs used
to construct the fixture.

For configured-key failure fixtures, the verifier MUST be invoked
with the specific public key documented in `expected.txt` —
`KeyResolutionFailure` is an invocation-error category that depends
on caller-supplied key material, not on artifact bytes.

## Fixtures

### Happy-path vectors

| File | Targets | Expected outcome |
| --- | --- | --- |
| `verify-happy-path.binpb` / `.yaml` | A fresh signature over a known payload; verifier uses the public key documented in `verify-happy-path.expected.txt`. | `Verified` |
| `acvp-fips186-5-p256-sha256-tc131.binpb` | NIST ACVP-Server AFT vector (tgId 14, tcId 131); the rebuilder replays sign and asserts byte-equality with the published `(R, S)` before writing. | `Verified` |

The `verify-happy-path` fixture exercises the YAML envelope (its
payload is ASCII ending in `\n`); the `acvp-fips186-5-*` fixture
exercises NIST byte-equality against a published reference vector
(its payload is a 128-octet random message and so ships in protobuf
form only). The companion `expected.txt` files name the source for
each. See [`../rebuild-rs/vendor/acvp/README.md`](../rebuild-rs/vendor/acvp/README.md)
for the ACVP-Server commit pin and the manual SHA-256 verification
commands; the rebuilder's test suite additionally replays every
P-256 / SHA-256 AFT case in the vendored file against our hand-rolled
signer (see `p256_sha256_acvp_aft_replay_matches` in
`rebuild-rs/src/alg_ecdsa.rs`).

### High-S / low-S acceptance pair

| File | Targets | Expected outcome |
| --- | --- | --- |
| `high-s.binpb` / `.yaml` | Signature with `S > n/2` over the happy-path payload. | `Verified` |
| `low-s.binpb` / `.yaml` | The matching `(R, n − S)` signature for the SAME `(payload, key)`. | `Verified` |

Both verify against the same public key. This proves the
implementation's high-S acceptance and absence of any low-S preference.

### Invalid component ranges (artifact bytes)

| File | Component | Expected outcome |
| --- | --- | --- |
| `invalid-r-zero.binpb` | `R = 0` | `MalformedAttemptedSigned` |
| `invalid-s-zero.binpb` | `S = 0` | `MalformedAttemptedSigned` |
| `invalid-r-equals-n.binpb` | `R = n` (curve order) | `MalformedAttemptedSigned` |
| `invalid-s-equals-n.binpb` | `S = n` | `MalformedAttemptedSigned` |

The wire-rule range is `0 < R < n` and `0 < S < n`. Boundary fixtures
test the strict-inequality rejection.

### Non-fixed-width encoding

| File | Targets | Expected outcome |
| --- | --- | --- |
| `signature-63-bytes.binpb` | 63-octet signature (one byte short of fixed width) | `MalformedAttemptedSigned` |
| `signature-65-bytes.binpb` | 65-octet signature (one byte over) | `MalformedAttemptedSigned` |

### Invalid public keys (configured-key — `KeyResolutionFailure`)

| File | Key | Expected outcome |
| --- | --- | --- |
| `bad-key-identity.txt` | `Q = O` (point at infinity, encoded as the all-zero byte string) | `KeyResolutionFailure` |
| `bad-key-off-curve.txt` | A 65-octet uncompressed-form encoding `04 || X || Y` whose `(X, Y)` does not satisfy the secp256r1 curve equation | `KeyResolutionFailure` |
| `bad-key-wrong-curve.txt` | A secp256k1 public key (Bitcoin's curve) handed to a P-256 verifier | `KeyResolutionFailure` |

### Deterministic two-nonce instability

| File | Targets | Expected outcome |
| --- | --- | --- |
| `two-nonce-instability.expected.txt` | Two signatures over the same `(private key, payload)` with explicitly chosen distinct nonces `k1 ≠ k2`. Both signatures MUST verify against the same public key; the signature octets MUST differ. | Both verify; octets differ. |
| `two-nonce-instability-k1.binpb` | First signature using `k1` | `Verified` |
| `two-nonce-instability-k2.binpb` | Second signature using `k2`; same payload and key as `k1` | `Verified` |

The explicit-`k` design (vs. "just sign twice with the library's RNG")
removes the vanishing-probability nonce-collision concern; the fixture
is a deterministic conformance test, not a probabilistic one.

### `algorithm_parameters` rejection

| File | Targets | Expected outcome |
| --- | --- | --- |
| `algorithm-parameters-present.expected.txt` | A `SignRequest` / `VerifyRequest` carrying a non-empty `algorithm_parameters` value. The algorithm defines NO parameters. | Signer: `SignerInvocationError(InvalidAlgorithmParameters)`. Verifier: `InvocationError(InvalidAlgorithmParameters)`. |
