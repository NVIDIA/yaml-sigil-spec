# `keyid` Conformance Fixtures

Exercises the `keyid` constraints pinned in
[README](../../README.md)'s "The Signature Document" section.

## Upstream sources

- [`README.md`](../../README.md) "The Signature Document" — `keyid`
  constraints
- [`proto/.../yaml_sigil.proto`](../../proto/yaml_sigil/v1alpha1/yaml_sigil.proto) — `optional string keyid = 2`
- [`schema/YamlSigilSignature.v1alpha1.schema.json`](../../schema/YamlSigilSignature.v1alpha1.schema.json)
  — `maxLength: 1024` and the CR/LF exclusion
- [`signing-api.md`](../../signing-api.md) — `InvalidKeyid` invocation-error category
- The multibyte test uses U+1F600, which is four UTF-8 octets per code point
  under [RFC 3629 section 3](https://www.rfc-editor.org/rfc/rfc3629#section-3).

The RFC excerpt and encoding table in the generator are third-party standards
material, not material relicensed under Apache-2.0. Their source, copying
conditions, disclaimer, and intellectual-property caveat are recorded in
[`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md).

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
  oversized, present-empty, and line-break cases.
- **Compose path** — a transcriber presented with
  `keyid-marker-injection.carrier.txt` returns
  `InvalidSignatureCarrier`.

## Fixtures

| File | `keyid` value | Constraint | Expected verifier outcome |
| --- | --- | --- | --- |
| `keyid-absent.yaml` / `keyid-absent.binpb` | (field omitted) | OK | proceeds to verification |
| `keyid-present-empty.yaml` / `keyid-present-empty.binpb` | empty string | violates min (0 octets) | `MalformedAttemptedSigned` |
| `keyid-1024-ascii.yaml` / `keyid-1024-ascii.binpb` | 1024 ASCII `a` characters (= 1024 UTF-8 octets) | exactly at the boundary | proceeds to verification |
| `keyid-1025-ascii.yaml` / `keyid-1025-ascii.binpb` | 1025 ASCII `a` characters (= 1025 octets) | one over | `MalformedAttemptedSigned` |
| `keyid-multibyte-under.yaml` / `keyid-multibyte-under.binpb` | 256 × U+1F600 emoji (= 1024 UTF-8 octets, 256 code points) | exactly at the octet boundary | proceeds to verification |
| `keyid-multibyte-over.yaml` / `keyid-multibyte-over.binpb` | 257 × U+1F600 emoji (= 1028 UTF-8 octets, 257 code points) | over the octet boundary by 4; the code-point count (257) is well under JSON Schema's `maxLength: 1024`. **An implementation that only checks code points would erroneously accept this.** | `MalformedAttemptedSigned` |
| `keyid-line-break.yaml` / `keyid-line-break.binpb` | `kid`, `U+000A`, `suffix` | line break prohibited | `MalformedAttemptedSigned` |
| `keyid-marker-injection.carrier.txt` | Markerless carrier with a constrained marker inside a single-quoted `keyid` | Compose envelope check | `InvalidSignatureCarrier` |

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
