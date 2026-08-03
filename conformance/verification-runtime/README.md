# Verification Runtime Conformance Fixtures

These fixtures exercise runtime state classification after structural
separation and signature metadata extraction succeed. They cover algorithm
support, successful cryptographic verification, and cryptographic mismatch
for both artifact forms.

## Sources and provenance

- [`verification-api.md`](../../verification-api.md) defines runtime ordering,
  algorithm support classification, and verifier states.
- [`algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md`](../../algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md)
  defines the algorithm used to create the control signature.
- [FIPS 186-5](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-5.pdf),
  [*Standards for Efficient Cryptography 1 (SEC 1)*](https://www.secg.org/sec1-v2.pdf),
  and
  [*Standards for Efficient Cryptography 2 (SEC 2)*](https://www.secg.org/sec2-v2.pdf)
  define the procedures and parameters used by the repository's P-256
  generator. Their source terms and caveats are recorded in
  [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md).

The private scalar, nonce, and payloads are locally selected test values. The
fixtures are locally generated rather than copied test vectors. Exact derived
values and their provenance appear in
[`runtime-classification.expected.txt`](./runtime-classification.expected.txt).

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/). See the
top-level [conformance README](../README.md) for the Docker build and run
commands. On a steady-state branch, re-running the rebuilder MUST reproduce
every fixture in this directory bit-identically.

Each artifact stem has one `.yaml` file and one `.binpb` file. The generator
writes each member as an independent regular file.

## Fixtures

Both pairs declare `ECDSA_SECP256R1_SHA256_RAW_RS64`, carry a non-empty,
structurally valid 64-octet signature, omit `keyid`, and use payload bytes that
fit the YAML envelope.

| Files | Difference | Primary use |
| --- | --- | --- |
| `valid.yaml` / `valid.binpb` | Signature verifies over the carried payload with the documented public key. | `Verified` control and unsupported-algorithm classification. |
| `cryptographic-mismatch.yaml` / `cryptographic-mismatch.binpb` | Carries the control signature over a different payload of the same length. | Cryptographic mismatch after all structural and runtime checks pass. |
| `runtime-classification.expected.txt` | Exact key, signature, payloads, semantic verifier configurations, and expected calls. | Reproducible driver instructions. |

## Required runtime matrix

Run each artifact case in both forms. Configure the implementation to resolve
the documented public key. These fixtures do not prescribe the key's API or
serialized representation.

| Scenario | Input | Verifier configuration | `PreVerify` | `Verify` |
| --- | --- | --- | --- | --- |
| Supported algorithm. | `valid.*` | Implements ECDSA and resolves the documented valid key. | `Ok` | `Verified`, with the exact valid payload bytes. |
| Unsupported algorithm. | `valid.*` | Does not implement the schema-defined ECDSA algorithm; caller configuration otherwise passes invocation validation. | `Ok` | `SignedButAlgorithmUnsupported`. |
| Cryptographic mismatch. | `cryptographic-mismatch.*` | Uses the supported-algorithm configuration. | `Ok` | `SignedButFailedVerification`. |

Only the supported-algorithm scenario returns `verified_payload_bytes`.
