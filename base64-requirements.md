# Base64 Requirements

This document is the normative profile for base64 encoding of the YAML-form
`signature` field in `YamlSigilSignature.v1alpha1`. Alphabet, padding, and
bit semantics are defined by [RFC 4648](https://www.rfc-editor.org/rfc/rfc4648)
section 5; this document only selects the profile conforming implementations
MUST use. The applicable source attribution, copying conditions, and warranty
disclaimer are recorded in
[`THIRD_PARTY_NOTICES.md`](./THIRD_PARTY_NOTICES.md).

The protobuf form carries raw signature octets on `YamlSigilSignature.signature`
and is not base64-encoded on the wire.

## Profile

Conforming implementations MUST use the following profile when encoding or
decoding the YAML-form `signature` scalar:

| Parameter | Requirement |
| --- | --- |
| Alphabet | URL-safe (RFC 4648 section 5) |
| Padding | None |
| Trailing bits | None — reject encodings with non-zero unused bits in the final partial quantum |

## Where this profile applies

- **Verification** — decode the YAML `signature` scalar during signature-metadata
  extraction. Decode failure (including trailing-bits violation) maps to
  `MalformedAttemptedSigned`. Note: empty decoded octets are **not** a base64
  decode failure under this profile — the empty string is a valid base64
  encoding of zero bytes. The non-empty `signature` rule is content-layer and
  is enforced by Verification before runtime algorithm-support classification.
  Algorithm-specific length checks follow the selected algorithm's rules (see
  [Verification API](./verification-api.md) Structural Rules By Form). Neither
  rule belongs to this decoder.
- **Signing and transcoding** — encode raw signature octets when emitting the
  YAML-form signature carrier. Canonical carrier emission MUST use this profile;
  see [Transcoding](./transcoding.md) for scalar transport rules (for example,
  no line wrapping inside the YAML scalar).
- **JSON Schema** — the `signature` string pattern in
  [`schema/YamlSigilSignature.v1alpha1.schema.json`](./schema/YamlSigilSignature.v1alpha1.schema.json)
  reflects URL-safe unpadded shape only. Trailing-bits rejection is enforced at
  decode, not fully expressible in JSON Schema.
