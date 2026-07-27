# Verification API

This document defines the `YamlSigil.v1alpha1` Verification API contract. The
concrete service, messages, enums, fields, and field numbers live in
[`verification.proto`](./proto/yaml_sigil/v1alpha1/verification.proto).

Verification consumes artifact bytes, validates the selected form, interprets
the unauthenticated signature document, and returns verified payload bytes only
when cryptographic verification succeeds.

## API Model

| Input | Contract |
| --- | --- |
| `input_bytes` | YAML-form artifact bytes or serialized `SignedYamlArtifact` bytes, according to `form`. |
| `form` | Required `Form`. `FORM_UNSPECIFIED` is invalid. |
| `config.public_key_handle` | Opaque public-key reference or material. Resolution is deployment-specific. |
| `config.algorithm_parameters` | Required exactly when the artifact algorithm defines verifier parameters. Surplus parameters are invalid. |
| `config.trust_policy` | Opaque trust-policy reference or material. Interpretation is deployment-specific. |
| `include_parser_observations` | Optional opt-in for implementation-specific decode observations. It MUST NOT affect the verifier state, verified payload bytes, or cryptographic verification. |

Raw YAML `alg` text and raw protobuf enum integers are boundary inputs only.
YAML strings that do not map to the closed `Algorithm` enum and protobuf
integers outside the enum fail before the API exposes a typed `alg`.

## Verifier States

A well-formed invocation returns exactly one state. Implementations MUST
preserve all five distinctions.

| State | Meaning | Payload returned |
| --- | --- | --- |
| `Verified` | Artifact is structurally valid, signature metadata is usable, and cryptographic verification succeeds. | Exact verified payload bytes. |
| `Unsigned` | YAML form contains no constrained marker. Protobuf form does not produce this state. | None. |
| `MalformedAttemptedSigned` | Artifact carries a signing attempt but fails structural validation, metadata validation, the protobuf `..._UNSPECIFIED` zero value for `alg`, schema-unknown `alg`, empty signature, or another pre-crypto rule. | None. |
| `SignedButAlgorithmUnsupported` | Artifact is structurally valid and uses a valid schema-defined algorithm that this verifier does not implement. | None. |
| `SignedButFailedVerification` | Artifact is structurally valid, the verifier implements the algorithm, and cryptographic verification was attempted but failed. | None. |

`Verified` means the returned payload bytes were signed. It is not a YAML parse
claim or application-schema claim.

## Invocation Errors

Invocation errors describe invalid caller inputs, not artifact properties. They
MUST be distinguishable from verifier states.

| Category | Meaning |
| --- | --- |
| `InvalidOrUnsupportedForm` | IDL request-shape failure for `FORM_UNSPECIFIED` or unsupported artifact form. In gRPC, this is `INVALID_ARGUMENT`. It is not a verifier state, `PreVerifyOutcome`, or `CanPreVerify` false result. |
| `InvalidAlgorithmParameters` | Required algorithm parameters are missing, malformed, out of bounds, or surplus. |
| `KeyResolutionFailure` | The verifier cannot obtain or use the configured key before cryptographic verification. |
| `TrustPolicyConfigurationError` | The supplied trust policy is malformed or internally inconsistent. |
| `InvalidPreVerifyResult` | `VerifyFromPreVerify` received a result that is not an `Ok` result from the same verifier instance and profile. |

Implementations MAY define additional subcategories, but callers MUST NOT need
to parse human text to distinguish the baseline categories above.

## Verification Stages

The verifier pipeline is:

1. Invocation validation.
2. Form-specific structural separation through the Transcription API.
3. Signature metadata extraction.
4. Runtime algorithm and parameter checks.
5. Cryptographic verification.

Implementations MAY combine stages internally, but the externally visible state
and invocation-error distinctions MUST match this model.

| Stage | Responsibility | Result surface |
| --- | --- | --- |
| Invocation validation | Validate caller-provided form, key, trust policy, and parameters known before artifact processing. | `InvocationError`. |
| Structural separation | Decompose YAML or protobuf form into `(payload_bytes, signature_carrier_bytes)`. | `Unsigned` or structural `MalformedAttemptedSigned`. |
| Metadata extraction | Decode the signature carrier into typed `alg`, optional `keyid`, and signature octets. | Metadata `MalformedAttemptedSigned`. |
| Runtime checks | Classify algorithm support, validate algorithm parameters, and enforce non-empty signature. | `InvocationError`, `MalformedAttemptedSigned`, or `SignedButAlgorithmUnsupported`. |
| Cryptographic verification | Verify signature over the payload bytes with the configured key and policy. | `Verified` or `SignedButFailedVerification`. |

## Structural Rules By Form

YAML-form Verification calls `Decompose(form = YAML)`, which runs
[Artifact Decomposition](./artifact-decomposition.md). The returned
signature-carrier bytes are parsed as a YAML mapping matching
`YamlSigilSignature.v1alpha1`.

Protobuf-form Verification calls
`Decompose(form = PROTOBUF, outer_conformance = ...)`. The returned
signature-carrier bytes are decoded as `YamlSigilSignature` under the
verifier's advertised conformance profile.

After Transcription succeeds, metadata extraction MUST enforce:

| Check | Applies to | Failure mapping |
| --- | --- | --- |
| `payload` is valid UTF-8 and does not begin with `EF BB BF`. | YAML form only. | `MalformedAttemptedSigned`. |
| YAML `schema` equals `YamlSigilSignature.v1alpha1`; protobuf schema identity is the message type. | Both. | `MalformedAttemptedSigned`. |
| `alg` maps to a defined `Algorithm` enum value. | Both. | Invalid or schema-unknown values are `MalformedAttemptedSigned`. |
| Present `keyid` is non-empty, at most 1024 UTF-8 octets, and contains no `U+000A` or `U+000D`. | Both. | `MalformedAttemptedSigned`. |
| YAML `signature` base64-decodes per [Base64 Requirements](./base64-requirements.md); protobuf `signature` carries raw bytes. | YAML decode only. | `MalformedAttemptedSigned`. |

The non-empty and algorithm-specific signature length checks run after metadata
extraction, before or during cryptographic verification.

## Algorithm Policy

[README](./README.md) defines the closed `alg` allowlist and canonical names.
Verification classifies encountered algorithm values as follows.

| Value class | Verifier behavior |
| --- | --- |
| Protobuf zero value `ALGORITHM_UNSPECIFIED`. | `MalformedAttemptedSigned`. |
| Schema-unknown protobuf enum integer or YAML `alg` string. | `MalformedAttemptedSigned`. |
| Schema-defined algorithm not implemented by this verifier. | `SignedButAlgorithmUnsupported`. |
| Schema-defined algorithm implemented by this verifier. | Attempt cryptographic verification. |

Algorithm parameters are caller-supplied verifier inputs. Missing, malformed,
out-of-bounds, or surplus parameters return `InvalidAlgorithmParameters`.
Well-formed parameters that do not match what the signer used cause
`SignedButFailedVerification`.

## Output Contract

The verifier returns verified payload bytes and verifier metadata only.

- `verified_payload_bytes` is populated only for `Verified`.
- Returned bytes are the exact payload bytes covered by the signature.
- The API MUST NOT return raw artifact bytes, parsed YAML payload trees, or
  deserialized protobuf payload messages.
- Artifact-derived diagnostics MUST remain separate from verifier-derived
  metadata.
- `parser_observations` are verifier-derived decode observations, populated
  only when requested through `include_parser_observations`. They are
  non-normative and MUST NOT be parsed for conformance.

## Conformance Profiles

Verification advertises exactly one conformance profile for the inner
signature-document decode. The same profile applies to all supported forms.

| Condition | Protobuf manifestation | YAML manifestation |
| --- | --- | --- |
| Unknown field. | A tag whose field number is not declared in `YamlSigilSignature`. | A mapping key other than `schema`, `alg`, `keyid`, or `signature`. |
| Duplicate known singular field. | Multiple wire occurrences of the same non-`repeated` field. | Multiple entries for the same mapping key. |

| Profile | Inner signature-document rule | Expected protobuf outer conformance |
| --- | --- | --- |
| `Strict` | Reject unknown fields and duplicate known singular fields. | `OUTER_CONFORMANCE_STRICT`. |
| `Permissive` | Accept per the form's default decode semantics. | Caller-determined. |
| `SignatureStrict` | Reject unknown fields and duplicate known singular fields. | `OUTER_CONFORMANCE_SIGNATURE_STRICT`. |

Only these profiles are conforming. A verifier supporting both wire forms MUST
advertise one profile for both. Advertising different profiles per form is
non-conforming.

The advertised profile is a ceiling on permissiveness. A verifier MAY behave
stricter than its advertised profile on any axis. It MUST NOT advertise a
stricter profile than it actually enforces.

Strict inner-protobuf behavior is not available from many stock protobuf
decoders. Implementations that cannot reject unknown fields and duplicate
known singular fields MUST advertise `Permissive` or add a wire-level
validation layer before advertising `Strict` or `SignatureStrict`.

## Pre-Verification Helpers

`Verify` is required. Helpers are optional and MUST be advertised when
provided. `VerifyFromPreVerify` is an in-process and same-instance concept,
not a public RPC deployment surface. Invalid or unsupported `form` values fail
at request validation, before a helper result exists.

| Helper | Contract |
| --- | --- |
| `CanPreVerify` | Boolean summary of `PreVerify`. `allow_unsigned` controls whether unsigned YAML counts as true. |
| `PreVerify` | Runs structural processing and unauthenticated signature metadata extraction. It performs no cryptographic verification and does not classify runtime algorithm support. |
| `VerifyFromPreVerify` | Runs only the verification stage using an opaque successful result produced by the same verifier instance and profile. |

`PreVerifyOutcome` values:

| Outcome | Meaning |
| --- | --- |
| `Ok` | Structural and metadata extraction succeeded. |
| `Unsigned` | YAML form has no signing attempt. |
| `StructuralFailure` | Form-specific structural processing rejected the input. |
| `MetadataParseFailure` | YAML signature-document parsing, schema matching, required-field extraction, or base64 decoding failed. |

`PreVerifyResult` values are unverified. For any input where `PreVerify`
returns `Ok`, `Verify` and same-instance `VerifyFromPreVerify` MUST produce
the same verifier outcome under the same configuration.

## Reader-Side Rule

Readers downstream of verification MUST parse only `verified_payload_bytes`
from a `Verified` result. They MUST NOT re-open the original artifact, parse
trailing artifact spans, or fetch content the verifier did not return.

## Cross-Form Rules

- YAML-form and protobuf-form verifiers expose the same state model.
- Equivalent verdicts require equivalent trust policy, keys, algorithm support,
  algorithm parameters, and compatible conformance profile.
- `Verified` with zero payload bytes is valid and is not `Unsigned`.
