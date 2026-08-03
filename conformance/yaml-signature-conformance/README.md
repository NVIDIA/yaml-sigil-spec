# YAML Signature-Document Conformance Fixtures

YAML-form fixtures driving the [Verification API](../../verification-api.md)
required `schema` identity and "Conformance Profiles" rules. The profile cases
cover the YAML signature-carrier byte limit, document count, mapping root,
declared field types, universal duplicate-known-key rejection, and unknown-field
behavior in the YAML signature-document mapping. The protobuf-form symmetric
profile cases live under
[`../protobuf-conformance/`](../protobuf-conformance/).

## Upstream sources

- [YAML 1.2.2 §6.7.2](https://yaml.org/spec/1.2.2/#672-mapping-key) —
  mapping-key syntax.
- [YAML 1.2.2 §9.1](https://yaml.org/spec/1.2.2/#91-documents) — document
  markers and explicit documents.
- [`verification-api.md`](../../verification-api.md) — "Structural Rules By
  Form," "YAML Signature-Carrier Safety," and "Conformance Profiles."
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
   metadata-extraction stage under the conformance profile the fixture targets.
3. If the implementation exposes `PreVerify`, compare its outcome against the
   per-fixture expectations.
4. Compare the verifier state against the per-fixture expectations.

## Fixtures

Every fixture below carries a valid `payload` and constrained marker. Except
where a fixture targets the `signature` field's YAML type, each signature
scalar that reaches field extraction is a canonical 86-character, URL-safe
unpadded base64 encoding of 64 octets. The `duplicate-signature.yaml` values
are distinct but independently valid.

| File | Targets | `Strict` outcome | `SignatureStrict` outcome | `Permissive` outcome |
| --- | --- | --- | --- | --- |
| `valid-baseline.yaml` | Sanity baseline: each mapping key appears exactly once. | Decode reaches the verification stage. | Same. | Same. |
| `wrong-schema.yaml` | `schema` declares `YamlSigilSignature.v2alpha1`. `PreVerify` returns `MetadataParseFailure`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `missing-schema.yaml` | Required `schema` key is absent. `PreVerify` returns `MetadataParseFailure`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `duplicate-schema.yaml` | `schema` key appears twice with matching values. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `duplicate-alg.yaml` | `alg` key appears twice with different values. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `duplicate-keyid.yaml` | `keyid` key appears twice with different values. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `duplicate-signature.yaml` | `signature` key appears twice with different base64 strings. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `unknown-key.yaml` | An extra mapping key (`bogus`) not declared in the schema. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Accept; the unknown key is dropped at parse. |
| `oversized-carrier.yaml` | Markerless carrier exceeds 16,384 octets while retaining a valid mapping after a comment. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `document-end-at-eof.yaml` | One mapping document ends with explicit `...` and no additional content. | Decode reaches the verification stage. | Same. | Same. |
| `document-end-with-second-document.yaml` | Explicit `...` is followed by a second document whose `---` marker carries a comment and therefore is not a constrained YamlSigil marker. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `non-mapping-root.yaml` | The signature document root is a sequence containing one mapping. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `non-string-schema.yaml` | `schema` has the explicit standard tag `!!int`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `non-string-alg.yaml` | `alg` has the explicit standard tag `!!bool`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `non-string-keyid.yaml` | `keyid` has the explicit standard tag `!!int`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |
| `non-string-signature.yaml` | `signature` has the explicit standard tag `!!int`. | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` |

## Notes

Duplicate-known-key rejection and the carrier byte limit apply before
profile-specific unknown-field handling. An implementation MUST inspect raw
mapping-key occurrences before a decoder collapses them into an object.
`PreVerify` reports every safety failure in this table as
`MetadataParseFailure`.

`document-end-with-second-document.yaml` deliberately uses
`--- # second YAML document`. YAML recognizes that line as a document start,
but Artifact Decomposition does not recognize it as the exact constrained
`---\n` marker. The second document therefore remains in the carrier and tests
the single-document-through-EOF rule rather than last-marker selection.

Nesting, constructed-node, and alias-expansion limits are mandatory, but their
numeric values and counters are implementation-defined because common parser
APIs expose different controls. Implementations test those limits locally and
document them; this portable fixture directory does not prescribe one
library-specific counter model.
