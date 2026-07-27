# Schemas

This directory contains schema artifacts for `YamlSigil.v1alpha1`.

- [`YamlSigilSignature.v1alpha1.schema.json`](./YamlSigilSignature.v1alpha1.schema.json)
  defines the YAML-form signature document shape for
  `YamlSigilSignature.v1alpha1`. This is the shape Verification parses
  out of the YAML signature-carrier bytes that the
  [Transcription API](../transcription-api.md) delivers; Transcription
  itself treats the carrier as opaque bytes.

The JSON Schema document is written against the IETF JSON Schema draft:

- [JSON Schema, `draft-ietf-jsonschema-json-schema-00`](https://datatracker.ietf.org/doc/draft-ietf-jsonschema-json-schema/)

## JSON Schema interim validation formalism

The artifact in this directory is the **working enumeration of the
YAML-form `YamlSigilSignature.v1alpha1` shape**, written in JSON
Schema because that's the most widely understood notation for the
constraints today. The repo [README](../README.md)'s Implementation
Note treats this as the spec's interim validation formalism. JSON
Schema is in use today; the broader alignment story (driving the
`.proto` and the YAML-form schema from one source, with
`protovalidate` as the eventual mechanization path) is named under
Known Deficiencies.

JSON Schema's fit for a YAML-rooted format is imperfect:

- It is JSON-rooted formalism. The YAML mapping it validates is
  what an implementation's YAML parser produced; JSON Schema does
  not see the original YAML bytes.
- This file represents the `Strict` and `SignatureStrict` unknown-key
  posture. `Permissive` decoders apply its declared constraints to known
  fields but accept unknown mapping keys as specified by the
  [Verification API](../verification-api.md).
- The `keyid` 1024 UTF-8-octet bound is not fully expressible:
  JSON Schema's `maxLength` counts Unicode code points, so the
  schema's `maxLength: 1024` is a loose approximation; the strict
  octet bound is enforced at the decoder layer.
- The base64 trailing-bits rule (see
  [Base64 Requirements](../base64-requirements.md)) is enforced at
  decode, not in the schema regex.
- The `signature` length is not constrained here: the `pattern`
  validates the URL-safe base64 alphabet and grouping, not a
  character count. The decoded length is algorithm-specific (a fixed
  64 octets / 86 base64url characters for both `v1alpha1`
  algorithms) and is enforced at the verification stage, per
  verification-api.md's metadata-extraction rules.

This file remains the reference shape any future bespoke
YAML-subset validator MUST agree with; switching mechanisms would
be a "swap the validator, keep the shape" exercise.

## Recommended editing workflow

When the signature-document schema needs to change, **start by editing
the protobuf representation** —
[`v1alpha1.YamlSigilSignature`](../proto/yaml_sigil/v1alpha1/yaml_sigil.proto)
— and **then replicate the change here** in
[`YamlSigilSignature.v1alpha1.schema.json`](./YamlSigilSignature.v1alpha1.schema.json).
The two are two reifications of the same logical schema and MUST stay
aligned (see the alignment notes below). For new `Algorithm` enum
entries, update the protobuf enum, the `alg` wire-string mapping in
[README.md](../README.md), and this file's `algorithm` enum together.

## Alignment with the protobuf schema

The protobuf `v1alpha1.YamlSigilSignature` definition (in
[`proto/yaml_sigil/v1alpha1/yaml_sigil.proto`](../proto/yaml_sigil/v1alpha1/yaml_sigil.proto))
and the YAML `YamlSigilSignature.v1alpha1` definition here are two
reifications of the same logical schema. **They MUST be kept aligned:
a change to one MUST be accompanied by a matching change to the
other.** A formal alignment process — automated checks, cross-validation
tooling, conformance tests — cannot be created until both
representations exist by hand first. That is why both are written by
hand now. Mechanizing the alignment is future work; the artifacts have
to exist before any process can validate them against each other.

**`alg` spelling.** Protobuf `Algorithm` enum constants use an
`ALGORITHM_` prefix; this JSON Schema and the YAML `alg` scalar use the
unprefixed wire strings. The mapping table in [README.md](../README.md)
("The Signature Document") is authoritative. `ALGORITHM_UNSPECIFIED`
(protobuf zero) is not listed here because it is not a valid YAML-form
`alg` value.
