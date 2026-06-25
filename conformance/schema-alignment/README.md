# Schema Alignment Fixtures

Validates that YAML `alg` strings and protobuf `Algorithm` enum
integers map to the same algorithm, that the
`ALGORITHM_UNSPECIFIED` / unknown-string / unknown-integer cases are
rejected consistently across forms, and that the YAML wire form does
NOT accept the protobuf-prefixed (`ALGORITHM_…`) spelling.

See [README](../../README.md)'s "The Signature Document" section for
the authoritative mapping table.

## Upstream sources

- [`README.md`](../../README.md) "The Signature Document" — canonical names +
  slot assignment + protobuf-prefix rule
- [`proto/.../yaml_sigil.proto`](../../proto/yaml_sigil/v1alpha1/yaml_sigil.proto)
  — `Algorithm` enum
- [`schema/YamlSigilSignature.v1alpha1.schema.json`](../../schema/YamlSigilSignature.v1alpha1.schema.json)
  — YAML form's enum + `$comment`
- [`verification-api.md`](../../verification-api.md) "Algorithm Policy"

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the
Docker build and run commands. On a steady-state branch,
re-running the rebuilder MUST reproduce the bytes of every
fixture in this directory bit-identically; a non-empty diff
is either a generator defect or an intended spec change that
has propagated through the generator.

## How to use

For each YAML fixture, parse the document and confirm the
`alg` string matches the JSON Schema's `algorithm` enum (and that the
implementation accepts or rejects per the expected column). For each
protobuf fixture, decode the inner `YamlSigilSignature.alg` varint
and confirm the implementation accepts or rejects per the expected
column.

## YAML fixtures (`*.yaml`)

| File | `alg` content | Expected verifier outcome |
| --- | --- | --- |
| `yaml-alg-ed25519.yaml` | `ED25519_PUREEDDSA_RAW_RS64_CANONICAL` | proceeds to verification (`Verified` / `SignedButFailedVerification` / etc. per crypto) |
| `yaml-alg-ecdsa.yaml` | `ECDSA_SECP256R1_SHA256_RAW_RS64` | proceeds to verification |
| `yaml-alg-unknown-string.yaml` | `FOO_BAR_BAZ` (not in the allowlist) | `MalformedAttemptedSigned` (metadata-extraction failure: schema-unknown string) |
| `yaml-alg-prefixed-rejected.yaml` | `ALGORITHM_ED25519_PUREEDDSA_RAW_RS64_CANONICAL` (the protobuf-prefixed spelling, which is NOT valid in YAML form) | `MalformedAttemptedSigned` (the YAML wire form rejects the prefix per [README](../../README.md)) |
| `yaml-alg-unspecified-rejected.yaml` | `ALGORITHM_UNSPECIFIED` (the protobuf zero-value name, which is NOT a valid YAML `alg`) | `MalformedAttemptedSigned` |

## Protobuf fixtures (`*.binpb`)

| File | Inner `alg` varint | Expected verifier outcome |
| --- | --- | --- |
| `proto-alg-ed25519.binpb` | `1` (`ALGORITHM_ED25519_PUREEDDSA_RAW_RS64_CANONICAL`) | proceeds to verification |
| `proto-alg-ecdsa.binpb` | `2` (`ALGORITHM_ECDSA_SECP256R1_SHA256_RAW_RS64`) | proceeds to verification |
| `proto-alg-unspecified.binpb` | `0` (`ALGORITHM_UNSPECIFIED`) | `MalformedAttemptedSigned` (runtime classification rejects the zero value) |
| `proto-alg-unknown-integer.binpb` | `42` (not in the enum) | `MalformedAttemptedSigned` (structural failure: schema-unknown enum integer) |

## Note on YAML rejection vs protobuf rejection

YAML unknown-string failures are caught at **metadata extraction**
(YAML parses; the string-to-enum mapping fails). Protobuf unknown-enum
failures are caught at the **structural** stage (the wire integer is
not in the closed enum). Both surface as `MalformedAttemptedSigned`,
but via different verifier-stage paths — see the Algorithm Policy
in [Verification API](../../verification-api.md).
