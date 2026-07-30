# Transcoding

This document defines transcoding between YAML and protobuf concrete forms of
a `YamlSigil.v1alpha1` artifact. Transcoding preserves payload bytes and the
effective decoded `YamlSigilSignature`; it does not prove signature validity.

Transcoding uses two layers:

- Transcription decomposes and composes the outer envelope.
- Verification metadata extraction interprets and re-emits the signature
  carrier.

It introduces no new RPC methods.

## YAML To Protobuf

A conforming YAML to protobuf transcoder MUST:

| Step | Action | Failure |
| --- | --- | --- |
| 1. Decompose YAML. | Call `Decompose(form = YAML)` and obtain `(payload_bytes, signature_carrier_bytes)`. | Propagate `Unsigned` or `MalformedAttemptedSigned`. |
| 2. Parse carrier. | Recover typed `YamlSigilSignature` under the verifier's advertised profile. | Parse, schema, base64, duplicate, unknown-field, or invalid-`alg` failure maps to `MalformedAttemptedSigned`. |
| 3. Emit carrier as protobuf. | Serialize the typed `YamlSigilSignature` as the body of the outer `signature` submessage. | Serializer failure is an implementation failure. |
| 4. Compose protobuf. | Call `Compose(form = PROTOBUF)` with original payload bytes and emitted carrier bytes. | Propagate Compose failure. |

## Protobuf To YAML

A conforming protobuf to YAML transcoder MUST:

| Step | Action | Failure |
| --- | --- | --- |
| 1. Decompose protobuf. | Call `Decompose(form = PROTOBUF, outer_conformance = ...)` and obtain `(payload_bytes, signature_carrier_bytes)`. | Propagate `MalformedAttemptedSigned`. |
| 2. Parse carrier. | Recover typed `YamlSigilSignature` under the verifier's advertised profile. | Inner-signature conformance or invalid-`alg` failure maps to `MalformedAttemptedSigned`. |
| 3. Emit carrier as YAML. | Emit markerless canonical YAML carrier bytes. | Non-conforming output if the emitted carrier cannot form exactly one signature document through EOF. |
| 4. Compose YAML. | Call `Compose(form = YAML)` with original payload bytes and emitted carrier bytes. | Propagate Compose failure, including `InvalidPayloadBytes` when protobuf payload bytes do not satisfy the YAML envelope. |

## Canonical YAML Carrier

Signing and protobuf-to-YAML transcoding MUST use this carrier profile.
Verification does not require byte-identical carrier formatting because the
carrier is not signed.

| Item | Requirement |
| --- | --- |
| Field order | `schema`, `alg`, `keyid` if present, `signature`. |
| Field separator | One field per line, each terminated by `\n`. |
| Key/value syntax | `key: value\n` with one space after `:`. |
| YAML features | No anchors, aliases, custom tags, comments, flow form, or block scalars. |
| String values | Emit `schema`, `alg`, and `signature` as plain scalars. Emit `keyid` as a double-quoted scalar with required YAML escapes. |
| `alg` value | Canonical name from [README](./README.md). |
| Signature encoding | Base64 per [Base64 Requirements](./base64-requirements.md), no line wrapping. |
| Envelope safety | No constrained marker at a carrier line-start position. |
| EOF | Exactly one trailing `\n` after the last field and no additional octets. |

The constrained marker is not part of the carrier. Compose owns the marker.

## Round-Trip Properties

| Property | Result |
| --- | --- |
| Payload bytes preserved across YAML to protobuf to YAML. | Yes. |
| Payload bytes preserved across protobuf to YAML to protobuf. | Yes, when the protobuf payload satisfies the YAML envelope. Otherwise protobuf to YAML fails. |
| Effective decoded `YamlSigilSignature` fields preserved. | Yes, except unknown fields and protobuf duplicate occurrences discarded or merged under a `Permissive` decoder's documented semantics. Duplicate known YAML mapping keys are invalid. |
| Signature octets preserved when the round trip completes. | Yes. |
| Full artifact byte hash preserved. | Not guaranteed. |
| Signature validity preserved when the round trip completes. | Yes, under equivalent verifier configuration. |

Equivalent verifier configuration means equivalent trust policy, key material,
algorithm support, algorithm parameters, and compatible conformance profile.

## Commitments

Transcoding depends on these normative commitments:

| Commitment | Source |
| --- | --- |
| YAML-form UTF-8 and BOM rules. | [README](./README.md). |
| Constrained marker profile and EOF rule. | [Artifact Decomposition](./artifact-decomposition.md). |
| `YamlSigilSignature` schema shape. | `.proto` and JSON Schema. |
| Closed `alg` allowlist. | [README](./README.md). |
| YAML `signature` base64 profile. | [Base64 Requirements](./base64-requirements.md). |
| Verifier states. | [Verification API](./verification-api.md). |
