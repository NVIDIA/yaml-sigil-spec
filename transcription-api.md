# Transcription API

This document defines the `YamlSigil.v1alpha1` Transcription API contract. The
concrete service, messages, enums, fields, and field numbers live in
[`transcription.proto`](./proto/yaml_sigil/v1alpha1/transcription.proto).

Transcription is a bytes-only envelope layer. It composes an abstract Artifact
`(payload_bytes, signature_carrier_bytes)` into YAML or protobuf form, and
decomposes either form back into those two byte strings. It MUST NOT parse the
signature carrier.

## API Model

### Compose

| Input | Contract |
| --- | --- |
| `payload` | Required opaque payload bytes. Compose carries them verbatim. |
| `signature_carrier` | Required opaque signature-carrier bytes. Compose carries them verbatim at the form-specific location. |
| `form` | Required `TranscriptionForm`. The transcriber emits exactly that form or refuses. |

### Decompose

| Input | Contract |
| --- | --- |
| `artifact` | Required envelope bytes. |
| `form` | Required `TranscriptionForm`. The transcriber decomposes exactly that form or refuses. |
| `outer_conformance` | Required when `form` is `TRANSCRIPTION_FORM_PROTOBUF`; otherwise it MUST be `OUTER_CONFORMANCE_UNSPECIFIED`. |

A successful `Decompose` returns the abstract Artifact. Inner
`YamlSigilSignature` interpretation belongs to the Verification API.

## Capabilities

A transcriber MUST advertise the typed values it accepts:

| Capability | Rule |
| --- | --- |
| `supported_forms` | MUST contain only supported `TranscriptionForm` values and MUST NOT contain `TRANSCRIPTION_FORM_UNSPECIFIED`. |
| `supported_outer_conformances` | MUST contain only supported `OuterConformance` values and MUST NOT contain `OUTER_CONFORMANCE_UNSPECIFIED`. It MAY be empty for YAML-only transcribers. |
| `emits_canonical_yaml_envelope` | If `true`, YAML Compose MUST use the LF marker form `---\n`. If `false`, YAML Compose MAY use either constrained marker form. |

## Validation And Outcomes

Request-shape failures return `TranscriberInvocationError` and stop before
byte processing.

| Check | Category |
| --- | --- |
| `form` is unspecified, schema-unknown, or absent from `supported_forms`. | `InvalidOrUnsupportedForm`. |
| Protobuf Decompose receives an unsupported or unspecified `outer_conformance`, or non-protobuf Decompose receives a non-unspecified `outer_conformance`. | `InvalidOrUnsupportedOuterConformance`. |

Compose failures after request-shape validation return `TranscriberError`.

| Check | Category |
| --- | --- |
| YAML Compose receives payload bytes that are invalid UTF-8, begin with a BOM, or are non-empty without a trailing `0A` or `0D 0A`. | `InvalidPayloadBytes`. |
| YAML Compose receives a signature carrier containing a constrained marker at a line-start position. | `InvalidSignatureCarrier`. |

Decompose returns a structural outcome.

| Outcome | Meaning | Verifier-state mapping |
| --- | --- | --- |
| `Ok` | Decomposition succeeded and returned `payload` plus `signature_carrier`. | Proceed to Verification. |
| `Unsigned` | YAML form has no constrained marker. Protobuf form does not produce this outcome. | `Unsigned`. |
| `MalformedAttemptedSigned` | Envelope-level structural validity failed. | `MalformedAttemptedSigned`. |

An empty `signature_carrier` is accepted at the Compose boundary because the
carrier is opaque. In YAML form it later fails structural Decompose as
`MalformedAttemptedSigned`. In protobuf form it can decompose as `Ok` and then
fail Verification's signature metadata checks.

## YAML Profile

<p align="center">
  <img
    src="./images/yaml-artifact-transcription-diagram.png"
    alt="YAML transcription byte ranges.">
</p>

YAML-form Compose writes:

```text
payload bytes || constrained marker || signature carrier bytes
```

The constrained marker is `---\n` or `---\r\n` at a line-start position.
`signature_carrier` is markerless at the API boundary. Compose inserts the
marker, and Decompose strips it.

YAML Compose MUST reject a `signature_carrier` containing a constrained marker
at offset `0` or immediately after an `0A` octet. It returns
`InvalidSignatureCarrier`. This envelope check does not parse the carrier.
The carrier-wide check is not redundant with field-specific rules such as the
`keyid` line-break constraint. It applies to all carrier bytes, including bytes
for signature-document fields added later, without requiring Transcription to
parse them.

YAML Decompose MUST run the byte-level algorithm in
[Artifact Decomposition](./artifact-decomposition.md). That document is
authoritative for marker recognition, selected ranges, UTF-8 and BOM
preconditions, empty-body handling, and outcome mapping.

Transcription does not canonicalize the YAML signature carrier. Field order,
quoting, and base64 formatting are outside this layer.

## Protobuf Profile

Protobuf-form Compose serializes `SignedYamlArtifact` with:

- `payload` set to the payload bytes.
- The outer `signature` submessage body set to the signature-carrier bytes.

Protobuf Decompose inspects only the outer `SignedYamlArtifact` wire shape. It
locates the `payload` bytes and the length-delimited body of the outer
`signature` submessage, and returns both verbatim. It MUST NOT decode the
inner `YamlSigilSignature` at this layer.

Outer-envelope conformance applies only to protobuf Decompose:

| Mode | Outer `SignedYamlArtifact` behavior |
| --- | --- |
| `Strict` | Reject unknown outer fields, duplicate outer `signature`, and duplicate outer `payload`. |
| `SignatureStrict` | Reject duplicate outer `signature`. Accept other unknown outer fields and duplicate outer `payload` per protobuf default semantics. |

Any outer-envelope rejection maps to `MalformedAttemptedSigned`.

## Output Contract

| Operation | Successful output |
| --- | --- |
| Compose YAML | Payload bytes, constrained marker, and signature-carrier bytes through EOF. |
| Compose protobuf | Serialized `SignedYamlArtifact`. |
| Decompose | `(payload_bytes, signature_carrier_bytes)` as opaque bytes. |

Compose MUST NOT return modified payload bytes. Line-terminator append is a
Signing API concern.

## Rules

- Transcription touches the envelope only.
- Payload and signature-carrier bytes are byte-preserved across Compose and
  Decompose.
- Transcription MUST NOT read `alg`, `keyid`, or signature octets. The YAML
  carrier marker check is an envelope operation.
- YAML and protobuf are independent envelope forms for the same abstract
  Artifact.
