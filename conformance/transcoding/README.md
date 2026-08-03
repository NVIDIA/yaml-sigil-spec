# Transcoding Conformance Fixtures

These paired fixtures exercise YAML to protobuf and protobuf to YAML
[Transcoding](../../transcoding.md) for `signature` string values whose
base64url text resembles YAML's null, Boolean, or integer forms. They test
logical field and signature-octet preservation without prescribing a scalar
presentation for emitted YAML.

## Sources

- [`transcoding.md`](../../transcoding.md) defines the cross-form steps,
  canonical YAML carrier profile, and round-trip properties.
- [`verification-api.md`](../../verification-api.md) requires declared YAML
  signature-document values to be strings during metadata extraction.
- [`base64-requirements.md`](../../base64-requirements.md) defines the
  URL-safe unpadded encoding of YAML-form `signature` values.
- [`yaml_sigil.proto`](../../proto/yaml_sigil/v1alpha1/yaml_sigil.proto)
  defines the protobuf `SignedYamlArtifact` and `YamlSigilSignature` fields.
- [YAML 1.2.2 §10.3.2](https://yaml.org/spec/1.2.2/#1032-tag-resolution)
  defines Core Schema plain-scalar resolution for empty, null, Boolean, and
  numeric forms.

The fixture strings and corresponding signature octets are locally
constructed from the base64url alphabet. They are not copied test vectors.
The matching generator module proves each byte string encodes to the value
listed below.

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/). See the
top-level [conformance README](../README.md) for the Docker build and run
commands. On a steady-state branch, re-running the rebuilder MUST reproduce
every fixture in this directory bit-identically.

Each stem has one `.yaml` file and one `.binpb` file. The generator writes
each member as an independent regular file. The YAML member supplies one
permitted input presentation; it does not specify the presentation a
conforming transcoder must emit.

## Fixtures

Every pair uses the payload bytes `payload: example` followed by LF, algorithm
slot `1`, and no `keyid`.

| Stem | `signature` string value | Signature octets in hex | Plain Core Schema resolution risk |
| --- | --- | --- | --- |
| `empty` | Empty string. | Empty. | Null. |
| `boolean-like-true` | `true` | `b6bb9e` | Boolean. |
| `null-like-null` | `null` | `9ee965` | Null. |
| `numeric-looking-1234` | `1234` | `d76df8` | Integer. |

## Required checks

For each fixture pair, a conforming implementation performs these checks:

1. Transcode the `.binpb` artifact to YAML. Parse the emitted signature
   document and confirm that `signature` is a YAML string equal to the table's
   string value.
2. Transcode that emitted YAML back to protobuf. Confirm that the payload,
   `alg`, absent `keyid`, and signature octets equal the input's effective
   `SignedYamlArtifact` fields.
3. Transcode the `.yaml` artifact to protobuf. Confirm that its effective
   fields equal those in the paired `.binpb` artifact.
4. Transcode the resulting protobuf back to YAML and confirm the same parsed
   string value and effective fields.

Do not compare emitted YAML bytes with the checked-in `.yaml` member. The
carrier profile permits multiple scalar presentations that preserve the same
string value.

These fixtures carry empty or three-octet signatures, so neither algorithm in
`v1alpha1` accepts them for successful cryptographic verification. Transcoding
interprets and preserves the typed signature carrier; it does not prove
signature validity.
