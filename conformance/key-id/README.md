# `keyid` Conformance Fixtures

Exercises the `keyid` structural bounds (optional; when present, 1..1024
UTF-8 octets) pinned in [README](../../README.md)'s "The Signature
Document" section.

## Upstream sources

- [`README.md`](../../README.md) "The Signature Document" — keyid bounds
- [`proto/.../yaml_sigil.proto`](../../proto/yaml_sigil/v1alpha1/yaml_sigil.proto) — `optional string keyid = 2`
- [`schema/YamlSigilSignature.v1alpha1.schema.json`](../../schema/YamlSigilSignature.v1alpha1.schema.json) — `maxLength: 1024`
- [`signing-api.md`](../../signing-api.md) — `InvalidKeyid` invocation-error category
- The multibyte test uses U+1F600 (4 UTF-8 octets per code point); see
  [Unicode](https://www.unicode.org/charts/) /
  [RFC 3629](https://www.rfc-editor.org/rfc/rfc3629) for the UTF-8 encoding
  rules.

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the
Docker build and run commands. On a steady-state branch,
re-running the rebuilder MUST reproduce the bytes of every
fixture in this directory bit-identically; a non-empty diff
is either a generator defect or an intended spec change that
has propagated through the generator.

## How to use

For each fixture, decode the artifact (YAML or protobuf), inspect the
`keyid` field, and confirm the implementation produces the expected
outcome:

- **Verifier path** — implementation runs `Verify(form, …)` and returns
  either a successful state (if all other rules pass) or
  `MalformedAttemptedSigned` (if `keyid` is invalid).
- **Signer path** — a signer presented with the equivalent
  `SignRequest.keyid` would return `InvalidKeyid` for the
  oversized / present-empty cases.

## Fixtures

| File | `keyid` value | Bound check | Expected verifier outcome |
| --- | --- | --- | --- |
| `keyid-absent.yaml` / `keyid-absent.binpb` | (field omitted) | OK | proceeds to verification |
| `keyid-present-empty.yaml` / `keyid-present-empty.binpb` | empty string | violates min (0 octets) | `MalformedAttemptedSigned` |
| `keyid-1024-ascii.yaml` / `keyid-1024-ascii.binpb` | 1024 ASCII `a` characters (= 1024 UTF-8 octets) | exactly at the boundary | proceeds to verification |
| `keyid-1025-ascii.yaml` / `keyid-1025-ascii.binpb` | 1025 ASCII `a` characters (= 1025 octets) | one over | `MalformedAttemptedSigned` |
| `keyid-multibyte-under.yaml` / `keyid-multibyte-under.binpb` | 256 × U+1F600 emoji (= 1024 UTF-8 octets, 256 code points) | exactly at the octet boundary | proceeds to verification |
| `keyid-multibyte-over.yaml` / `keyid-multibyte-over.binpb` | 257 × U+1F600 emoji (= 1028 UTF-8 octets, 257 code points) | over the octet boundary by 4; the code-point count (257) is well under JSON Schema's `maxLength: 1024`. **An implementation that only checks code points would erroneously accept this.** | `MalformedAttemptedSigned` |

## Why the multibyte fixtures matter

`maxLength: 1024` in JSON Schema 2020-12 is defined to count Unicode
code points (or UTF-16 code units depending on validator). For a
1024-emoji `keyid` (each emoji = 1 code point, 4 UTF-8 octets):

- Code-point count: 1024 → JSON Schema sees this as exactly at the
  limit (valid).
- Octet count: 4096 → the spec's actual bound (rejected).

The schema is documented as a loose approximation; the implementation's
decoder MUST enforce the strict octet rule. The
`keyid-multibyte-over.yaml` fixture (with 257 emoji = 1028 octets but
257 code points) is the smoking gun: any code-point-only
implementation will accept it; any octet-counting implementation will
reject it.
