# Signing API

This document defines the `YamlSigil.v1alpha1` Signing API contract. The
concrete service, messages, enums, fields, and field numbers live in
[`signing.proto`](./proto/yaml_sigil/v1alpha1/signing.proto).

The API signs payload bytes and emits exactly one artifact in the requested
output form. A deployment may expose the API over gRPC, in-process calls, or
another binding, but the validation and output contract remains the same.

The deployment authorizes the signing key for its intended purpose and places
any required application context in the payload.

## Inputs

| Input | Contract |
| --- | --- |
| Private key or key handle | Required. Key loading, storage, and memory handling are deployment concerns. |
| Payload stream bytes | Required. These are the bytes to sign, subject to the output-form preconditions below. |
| `alg` | Required `Algorithm` value from the closed allowlist in [README](./README.md). The signer accepts it or refuses. |
| `algorithm_parameters` | Required exactly when the selected algorithm defines parameters. Surplus parameters are invalid. |
| `output_form` | Required `OutputForm`. The signer emits exactly that form or refuses. |
| `keyid` | Optional lookup hint. When present, it MUST be non-empty, at most 1024 UTF-8 octets, and contain no `U+000A` or `U+000D`. The signer copies it verbatim and MUST NOT derive it from private-key secret material. |

## Capabilities

A signer MUST advertise the typed values it accepts before signing:

| Capability | Rule |
| --- | --- |
| `supported_output_forms` | MUST contain only supported `OutputForm` values and MUST NOT contain `OUTPUT_FORM_UNSPECIFIED`. |
| `supported_algorithms` | MUST contain only implemented `Algorithm` values and MUST NOT contain `ALGORITHM_UNSPECIFIED`. |
| `best_effort_yaml_validation` | If `true`, the signer MAY return `YAMLValidationFailure`. If `false`, it MUST NOT return that category. |

## Validation And Errors

Request-shape failures return `SignerInvocationError` and stop before payload
processing.

| Check | Category |
| --- | --- |
| `alg` is unspecified, schema-unknown, absent from `supported_algorithms`, or unsupported. | `InvalidOrUnsupportedAlgorithm`. |
| Algorithm parameters are missing, malformed, out of bounds, or surplus. | `InvalidAlgorithmParameters`. |
| `output_form` is unspecified, schema-unknown, or absent from `supported_output_forms`. | `InvalidOrUnsupportedOutputForm`. |
| Present `keyid` is empty, longer than 1024 UTF-8 octets, or contains `U+000A` or `U+000D`. | `InvalidKeyid`. |

Failures after request-shape validation return `SignerError`.

| Check | Category |
| --- | --- |
| YAML output is selected and payload bytes violate YAML-envelope preconditions. | `InvalidPayloadBytes`. |
| YAML output is selected, a non-empty payload lacks a line terminator, and the signer refuses to append one. | `PayloadLineTerminatorRefusal`. |
| The private-key operation fails. | `KeyOperationFailure`. |
| Advertised best-effort YAML 1.2 validation rejects the payload. | `YAMLValidationFailure`. |

## Payload Preconditions

For YAML-form output:

- Payload bytes MUST be valid UTF-8.
- Payload bytes MUST NOT begin with the UTF-8 BOM octets `EF BB BF`.
- Payload bytes MAY be empty.
- A non-empty payload MUST end with `0A` or `0D 0A`, or the signer MUST either
  refuse with `PayloadLineTerminatorRefusal` or append one `0A`.
- If the signer appends `0A`, it signs the modified bytes and returns those
  modified bytes to the caller.
- No other payload modification is permitted.

For protobuf-form output:

- Payload bytes MAY be any sequence of octets, including the empty sequence.
- The signer MUST NOT modify the payload bytes.
- The YAML-form UTF-8, BOM, and line-terminator checks do not run.

## Processing And Output

The signer MUST:

1. Validate request shape.
2. Validate payload preconditions for the selected output form.
3. Apply best-effort YAML validation only when the signer advertises that
   discipline for the selected output.
4. Sign the final payload byte string exactly.
5. Build `YamlSigilSignature`.
6. Emit the requested artifact form.

The signature input is the final payload byte string. The signer MUST NOT
canonicalize YAML, normalize line endings, re-encode strings, trim whitespace,
add framing, or pre-hash unless the selected algorithm requires it.

| Output form | Required artifact |
| --- | --- |
| YAML | Final payload bytes, followed by `---\n`, followed by one YAML `YamlSigilSignature.v1alpha1` signature document through EOF. The carrier MUST follow [Canonical YAML Carrier](./transcoding.md#canonical-yaml-carrier). |
| Protobuf | Serialized `SignedYamlArtifact` with `payload` set to the final payload bytes and `signature` set to `YamlSigilSignature`. |

A successful response returns the serialized artifact. If the signer appended a
line terminator, it also returns the modified payload bytes. It MUST NOT return
private key material, intermediate hash state, parsed YAML trees, or additional
artifact bytes.

## Rules

- Empty payload signing is valid.
- The signature document is unsigned input. Authenticated data belongs in the
  payload stream.
- `keyid` is a lookup hint, not proof of signer identity.
- YAML and protobuf outputs are independent forms over the same signed payload
  bytes.
- Randomized algorithms MAY produce different signature octets across calls.
