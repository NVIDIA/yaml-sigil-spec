# YAML Signature-Document Conformance Fixtures

YAML-form fixtures driving the [Verification API](../../verification-api.md)
"Conformance Profiles" section — specifically, the
**duplicate-known-singular-field** and **unknown-field** rules as they
manifest in the YAML signature-document mapping (the protobuf-form
symmetric cases live under [`../protobuf-conformance/`](../protobuf-conformance/)).

## Upstream sources

- [YAML 1.2.2 §6.7.2](https://yaml.org/spec/1.2.2/#672-mapping-key) —
  mapping-key duplicate handling ("should warn" guidance).
- [`verification-api.md`](../../verification-api.md) — "Conformance Profiles":
  Strict / Permissive / SignatureStrict definitions.
- [`schema/YamlSigilSignature.v1alpha1.schema.json`](../../schema/YamlSigilSignature.v1alpha1.schema.json)
  — `additionalProperties: false` is the JSON-Schema-side expression of the
  "unknown field" rule.
- [`README.md`](../../README.md) — "The Signature Document" section names the
  same profile vocabulary.

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the Docker
build and run commands. On a steady-state branch, re-running the
rebuilder MUST reproduce the bytes of every fixture in this
directory bit-identically; a non-empty diff is either a generator
defect or an intended spec change that has propagated through the
generator.

## How to use these fixtures

1. Read the `.yaml` file as raw bytes.
2. Invoke `Decompose(form = TRANSCRIPTION_FORM_YAML)` from the
   [Transcription API](../../transcription-api.md) and then feed the
   recovered `signature_carrier_bytes` through Verification's
   metadata-extraction stage under the conformance profile the
   fixture targets.
3. Compare the verifier state against the per-fixture expectations.

## Fixtures

Every fixture below carries a valid `payload`, a valid constrained
marker, and a `YamlSigilSignature.v1alpha1` mapping in which one
otherwise-valid mapping key has either been duplicated (the
duplicate cases) or replaced with a key not in the closed schema
(the unknown case). The signature octets are the canonical 86-char
placeholder (URL-safe-unpadded base64 of 64 zero bytes), so any
rejection prior to the cryptographic stage is unambiguously
attributable to the conformance rule under test.

| File | Targets | `Strict` outcome | `SignatureStrict` outcome | `Permissive` outcome |
| --- | --- | --- | --- | --- |
| `valid-baseline.yaml` | Sanity baseline: each mapping key appears exactly once. | Decode reaches the verification stage. | Same. | Same. |
| `duplicate-schema.yaml` | `schema` key appears twice with matching values. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | The decoder may reject or accept using its documented effective value. |
| `duplicate-alg.yaml` | `alg` key appears twice with **different** values; an attacker is the model. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | The decoder may reject or accept using its documented effective value. |
| `duplicate-keyid.yaml` | `keyid` key appears twice with different values. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | The decoder may reject or accept using its documented effective value. |
| `duplicate-signature.yaml` | `signature` key appears twice with different base64 strings. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | The decoder may reject or accept using its documented effective value. |
| `unknown-key.yaml` | An extra mapping key (`bogus`) not declared in the schema. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Accept; the unknown key is dropped at parse. |

`Permissive` does not prescribe one duplicate-key outcome. An implementation
advertising `Permissive` MUST document in human-readable implementation
documentation whether its YAML decoder rejects duplicate known mapping keys
and, if it accepts them, the exact rule used to select each effective field
value. Naming the parser library or relying on source code alone does not
satisfy this requirement.

The `Permissive` column records both permitted outcomes. A decoder that rejects
duplicates returns `MalformedAttemptedSigned`. A decoder that accepts them
continues with its documented effective field values. Conformance drivers
SHOULD compare the observed behavior with the implementation's prose
documentation.

## Notes

A YAML parser whose API exposes raw mapping-key occurrences can
implement Strict / SignatureStrict by rejecting on the second occurrence.
A parser that accepts duplicates before exposing the mapping to the caller MUST
document its exact effective-value rule and SHOULD surface discarded duplicate
occurrences through the `parser_observations` channel when the caller opts in
via `include_parser_observations`.
