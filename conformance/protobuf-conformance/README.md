# Protobuf Conformance Fixtures

Wire-format fixtures driving the [Transcription API](../../transcription-api.md)
`Decompose(form = PROTOBUF, outer_conformance = …)` outer-envelope
conformance rules and the [Verification API](../../verification-api.md)
inner-`YamlSigilSignature` conformance profiles.

## Upstream sources

- [protobuf wire-format encoding](https://protobuf.dev/programming-guides/encoding/)
  — varint, tag, length-delimited rules
- [`proto/yaml_sigil/v1alpha1/yaml_sigil.proto`](../../proto/yaml_sigil/v1alpha1/yaml_sigil.proto)
  — `SignedYamlArtifact` message shape
- [`transcription-api.md`](../../transcription-api.md) — `OuterConformance`
  enum + `DecomposeOutcome` semantics
- [`verification-api.md`](../../verification-api.md) — Conformance Profiles
  (Strict / Permissive / SignatureStrict); the same profile vocabulary covers
  the YAML-form symmetric cases in
  [`yaml-signature-conformance/`](../yaml-signature-conformance/)

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the
Docker build and run commands. On a steady-state branch,
re-running the rebuilder MUST reproduce the bytes of every
fixture in this directory bit-identically; a non-empty diff
is either a generator defect or an intended spec change that
has propagated through the generator.

## Wire-format reminder

`SignedYamlArtifact` fields:

| Field | Number | Wire type | Tag byte |
| --- | --- | --- | --- |
| `payload` (bytes) | 1 | 2 (length-delimited) | `0x0A` |
| `signature` (`YamlSigilSignature`) | 2 | 2 (length-delimited) | `0x12` |

`YamlSigilSignature` fields:

| Field | Number | Wire type | Tag byte |
| --- | --- | --- | --- |
| `alg` (enum, varint) | 1 | 0 (varint) | `0x08` |
| `keyid` (string) | 2 | 2 (length-delimited) | `0x12` |
| `signature` (bytes) | 3 | 2 (length-delimited) | `0x1A` |

## How to use these fixtures

1. Read the `.binpb` file as raw bytes.
2. Invoke `Decompose(form = TRANSCRIPTION_FORM_PROTOBUF,
   outer_conformance = X)` where `X` is the
   [`OuterConformance`](../../proto/yaml_sigil/v1alpha1/transcription.proto)
   value the fixture targets (`STRICT` or `SIGNATURE_STRICT`; see
   the "Column meaning" note below for what the third column
   represents).
3. Compare the returned `DecomposeOutcome` (and any subsequent
   `Verifier` state) against the per-fixture expectations.

## Column meaning

The three outcome columns below are **paired profile states**, not
three separate values of a single enum. Each column names a
`(OuterConformance, ConformanceProfile)` pairing the spec recognises
as conforming:

| Column | Outer `OuterConformance` (`transcription.proto`) | Inner `ConformanceProfile` (`verification.proto`) | Verifier role |
| --- | --- | --- | --- |
| `Strict` | `OUTER_CONFORMANCE_STRICT` | `CONFORMANCE_PROFILE_STRICT` | Hardened both outer and inner. |
| `SignatureStrict` | `OUTER_CONFORMANCE_SIGNATURE_STRICT` | `CONFORMANCE_PROFILE_SIGNATURE_STRICT` | Hardened inner; outer permissive except for duplicate `signature`. |
| `Permissive` | Caller-determined (Transcription has no Permissive outer mode) | `CONFORMANCE_PROFILE_PERMISSIVE` | Verifier accepts whatever an unhardened parser produces. |

A verifier MUST advertise exactly one `ConformanceProfile` on
`VerifierCapabilities.conformance_profile`; advertising
`CONFORMANCE_PROFILE_UNSPECIFIED` is non-conforming for any verifier
that supports a wire form. Implementations that don't reject
duplicates SHOULD advertise `Permissive` (which is conforming and
matches what their parser does by default), not `Unspecified`.

## Fixtures

| File | Targets | `Strict` pairing outcome | `SignatureStrict` pairing outcome | `Permissive` pairing outcome |
| --- | --- | --- | --- | --- |
| `valid-baseline.binpb` | Sanity baseline: one payload, one signature submessage, no extras. | `Ok` | `Ok` | `Ok` |
| `duplicate-outer-payload.binpb` | Outer `payload` field appears twice. | `MalformedAttemptedSigned` (outer `Strict` rejects duplicate outer `payload`) | `Ok`, with `payload` = last wire occurrence (outer `SignatureStrict` accepts duplicate outer `payload` per standard protobuf last-wins) | `Ok`, last-wins |
| `duplicate-outer-signature.binpb` | Outer `signature` submessage appears twice. | `MalformedAttemptedSigned` (outer `Strict` rejects) | `MalformedAttemptedSigned` (outer `SignatureStrict` mandates rejection — this is the load-bearing case the profile is named for) | `Ok`, parsers merge submessages by default (see "Notes on duplicate-outer-`signature` under Permissive" below — `Permissive` implementations SHOULD log this) |
| `unknown-outer-field.binpb` | An unknown tag (field number `5`, varint) is present in the outer envelope. | `MalformedAttemptedSigned` | `Ok` (outer `SignatureStrict` only hardens the signature submessage, not the outer wrapper) | `Ok` |
| `inner-strict-duplicate-alg.binpb` | Inner `YamlSigilSignature` has two `alg` varint occurrences. | `MalformedAttemptedSigned` (inner `Strict` rejects) | `MalformedAttemptedSigned` (inner `SignatureStrict` rejects) | `Ok`, last-wins on inner scalar duplicates (inner `Permissive`) |
| `present-empty-outer-signature.binpb` | Outer `signature` submessage is present but its length-delimited body is zero-length. | `Ok` at Decompose with empty `signature_carrier`; later `MalformedAttemptedSigned` at Verification's verification stage (non-empty `signature` rule) | same as `Strict` | same as `Strict` |
| `binary-payload-no-yaml-fit.binpb` | Payload is a single octet `0x72` — a valid protobuf-form payload (the protobuf `bytes` carries arbitrary octets) that violates the YAML envelope's structural rules (non-empty, no trailing `0A` / `0D 0A`). Exercises the protobuf-vs-YAML payload-byte carve-out documented in `verification-api.md`'s metadata-extraction table and `transcoding.md`'s round-trip table. | `Ok` at Decompose; verifier MUST NOT raise YAML-envelope payload checks. Result is whatever the signed `(payload, signature, key)` triple verifies to — `MalformedAttemptedSigned` is the wrong category here. | same as `Strict` | same as `Strict` |
| `invalid-field-zero.binpb` | A complete baseline is followed by a length-delimited field with field number zero. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `out-of-range-field-number.binpb` | A complete baseline is followed by field number `2^29`, the first value outside protobuf's field-number range. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `overflowing-tag-varint.binpb` | A complete baseline is followed by a ten-octet tag varint whose final payload overflows `uint64`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `oversized-length.binpb` | An unknown length-delimited field declares `2^32 + 5` octets but supplies one octet. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `invalid-wire-type-6.binpb` | A complete baseline is followed by an unknown field whose tag declares wire type `6`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `invalid-wire-type-7.binpb` | A complete baseline is followed by an unknown field whose tag declares wire type `7`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |

The six malformed-wire fixtures reject before outer conformance checks.
Implementations MUST consume and validate the entire envelope through EOF;
successfully extracting the known fields does not permit ignoring a malformed
trailing record.

`Permissive`-only behavior is on the wire by default in any
protobuf parser. `Strict` and `SignatureStrict` are the pairings that
need to *reject*; the `Permissive` column captures what an unhardened
parser would do, which is the baseline implementations must override
when they advertise the harder pairings.

**Stricter-than-advertised behavior is conforming.** Per the
ceiling-reading paragraph in
[Verification API](../../verification-api.md)'s "Conformance
Profiles", a verifier MAY behave strictly stricter than its
advertised profile on any axis without being non-conforming. A
`Permissive`-advertising verifier whose protobuf decoder happens
to reject some duplicate-known-singular-field case is over-delivery,
not a conformance failure. (Few protobuf libraries do this by
default; the symmetric Rust-YAML case is more common, see
`yaml-signature-conformance/README.md` for that one.)

## Notes on duplicate-outer-`signature` under Permissive

A standard protobuf parser merges duplicate submessage fields by
concatenating their inner-field occurrences into one decoded
submessage. Under Permissive, this means an attacker can split an
inner `YamlSigilSignature` across two outer `signature` occurrences in
ways that no single occurrence carried complete metadata. This is the
attack vector `SignatureStrict` exists to close.
