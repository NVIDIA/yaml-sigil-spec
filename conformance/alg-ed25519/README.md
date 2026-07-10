# Ed25519 (`ED25519_PUREEDDSA_RAW_RS64_CANONICAL`) Conformance Fixtures

Drives the
[Ed25519 algorithm specification](../../algorithms/01-ED25519_PUREEDDSA_RAW_RS64_CANONICAL.md).

## Upstream sources

Cryptographic values are taken verbatim from the published canonical
sources:

- [RFC 8032 §7.1](https://www.rfc-editor.org/rfc/rfc8032#section-7.1) —
  Test 1 (empty message) and Test 2 (one-byte `0x72`) vectors. `seed`,
  `public_key`, `message`, `signature` are RFC values copied directly into
  the generator.
- [RFC 8032 §5.1.2 / §5.1.5 / §5.1.6 / §5.1.7](https://www.rfc-editor.org/rfc/rfc8032#section-5.1)
  — encoding, seed expansion, deterministic `r` derivation, verification
  equation.
- Chalkias, Garillot, Nikolaenko,
  ["Taming the Many EdDSAs"](https://eprint.iacr.org/2020/1244) —
  Algorithm 2 (strict variant), Table 5 (the eight small-order public-key
  encodings recorded as numeric conformance data in the generator). See
  [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md).
- [`algorithms/01-ED25519_PUREEDDSA_RAW_RS64_CANONICAL.md`](../../algorithms/01-ED25519_PUREEDDSA_RAW_RS64_CANONICAL.md)
  — slot specification.

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the
Docker build and run commands. On a steady-state branch,
re-running the rebuilder MUST reproduce the bytes of every
fixture in this directory bit-identically; a non-empty diff
is either a generator defect or an intended spec change that
has propagated through the generator.

## How to use

For each fixture below, the `*.binpb` is a serialized
`SignedYamlArtifact` ready for `Verify(form = TRANSCRIPTION_FORM_PROTOBUF, …)`.
The matching `*.yaml` carries the same `(payload, alg, signature)`
content under the YAML form. Per-fixture `expected.txt` files name
both the expected verifier-state outcome and the verification-key /
seed inputs used to construct the fixture (where applicable).

For configured-key failure fixtures, the verifier MUST be invoked
with the specific public key documented in `expected.txt` —
`KeyResolutionFailure` is an invocation-error category that
depends on caller-supplied key material, not on artifact bytes.

## Fixtures

### Happy-path vectors (RFC 8032 §7.1)

| File | RFC 8032 vector | Expected outcome |
| --- | --- | --- |
| `rfc8032-vec1-empty-message.binpb` / `.yaml` | Test 1 (empty message) | `Verified` |
| `rfc8032-vec2-one-octet.binpb` | Test 2 (one-byte message `0x72`) | `Verified` |

These vectors are taken verbatim from
[RFC 8032 §7.1](https://www.rfc-editor.org/rfc/rfc8032#section-7.1).
Implementations passing pure Ed25519 against those vectors will
produce the correct 64-octet `signature` wire bytes; the Test 1
YAML-form fixture pins the same `(R, S)` re-encoded under
[Base64 Requirements](../../base64-requirements.md). Test 2 ships
in protobuf form only — the one-byte signed payload `0x72` cannot
precede the constrained YAML marker without inserting a line
terminator that would change the signed bytes; `rfc8032-vec2-one-octet.expected.txt`
documents the same constraint and the top-level
[conformance README](../README.md) lists it under "Compromise (Ed25519)".

### Canonical-encoding rejection (artifact bytes)

| File | Targets | Expected outcome |
| --- | --- | --- |
| `noncanonical-R.binpb` | non-canonical `R` (encoded integer ≥ `p`) | `MalformedAttemptedSigned` |
| `noncanonical-S-equals-L.binpb` | `S = L` (boundary, just over) | `MalformedAttemptedSigned` |
| `noncanonical-S-equals-L-plus-1.binpb` | `S = L + 1` | `MalformedAttemptedSigned` |

These bytes are in the artifact, so the rejection is artifact-side
(`MalformedAttemptedSigned`), not configured-key-side.

### Configured-key rejection (key material — `KeyResolutionFailure`)

| File | Key supplied | Expected outcome |
| --- | --- | --- |
| `configured-key-small-order.txt` | one of the 8 small-order points (each encoded in its own line) | `KeyResolutionFailure` — see [Verification API](../../verification-api.md) |

These are not full artifacts — they're public-key encodings the
verifier would receive via `config.public_key_handle`. The
implementation MUST reject them at key resolution, not at artifact
parsing.

### Stable re-signing

| File | Targets | Expected outcome |
| --- | --- | --- |
| `stable-resign.txt` | A signer that signs the same `(seed, payload)` twice MUST produce byte-identical `signature` octets. The file pins the expected `(R, S)` for the RFC 8032 §7.1 Test 1 seed+message and an additional copy of the same. | Both invocations produce the documented signature octets verbatim. |

### `algorithm_parameters` rejection

| File | Targets | Expected outcome |
| --- | --- | --- |
| `algorithm-parameters-present.expected.txt` | A `SignRequest` / `VerifyRequest` carrying a non-empty `algorithm_parameters` value (one byte `0x00`). The algorithm defines NO parameters. | Signer: `SignerInvocationError(InvalidAlgorithmParameters)`. Verifier: `InvocationError(InvalidAlgorithmParameters)`. |

This case exercises the request-shape boundary, not the artifact —
the artifact content itself is the RFC 8032 §7.1 Test 1 vector;
what differs is the surrounding API call's `algorithm_parameters`
field. Only the `.expected.txt` sidecar ships (it documents the
request shape and expected outcomes); no `.binpb` is emitted because
the deviation is in the API call, not in the on-wire artifact.
