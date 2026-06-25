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
| `duplicate-schema.yaml` | `schema` key appears twice with matching values. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Last-wins. |
| `duplicate-alg.yaml` | `alg` key appears twice with **different** values; an attacker is the model. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Last-wins (the second `alg` value is what extraction sees). |
| `duplicate-keyid.yaml` | `keyid` key appears twice with different values. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Last-wins. |
| `duplicate-signature.yaml` | `signature` key appears twice with different base64 strings. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Last-wins. |
| `unknown-key.yaml` | An extra mapping key (`bogus`) not declared in the schema. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Accept; the unknown key is dropped at parse. |

`Permissive`-only behavior is what an unhardened YAML parser will
produce by default — YAML 1.2.2 §6.7.2 SAYS the parser "should warn"
but doesn't require rejection. `Strict` and `SignatureStrict` are
the modes that need to *reject*; the "Permissive outcome" column
captures what an unhardened parser would do.

**Implementations whose default YAML parser is stricter than
Permissive are still conforming.** Per the ceiling-reading paragraph
in [Verification API](../../verification-api.md)'s "Conformance
Profiles", the advertised profile is the loosest decode posture
the verifier guarantees; behaving strictly stricter than that on
any axis is over-delivery, not a conformance failure. A
`Permissive`-advertising verifier whose YAML library rejects
duplicate mapping keys at parse time (the structural default of
most Rust YAML libraries, for instance) MAY return
`MalformedAttemptedSigned` on the four `duplicate-*.yaml` fixtures
without violating its `Permissive` advertisement. Drivers SHOULD
record the over-delivery axis(es) in their own audit trail so
callers know what they actually get.

## Notes

A YAML parser whose API exposes raw mapping-key occurrences can
implement Strict / SignatureStrict by rejecting on the second occurrence.
A parser that collapses to last-wins before exposing the mapping to
the caller MUST disclose that limitation with the advertised
profile (see verification-api.md "Conformance Profiles") and SHOULD
surface the discarded duplicate through the `parser_observations`
channel when the caller opts in via `include_parser_observations`.
