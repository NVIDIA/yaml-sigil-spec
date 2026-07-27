# YAML Decomposition Fixtures

Byte-level decomposition fixtures driving the YAML-form
[Artifact Decomposition](../../artifact-decomposition.md) algorithm and
the [Transcription API](../../transcription-api.md) `Decompose(form = YAML)`
contract.

## Upstream sources

These fixtures are derived from this repository alone — the
constrained marker profile and the decomposition algorithm are
self-contained. The relevant authority documents are:

- [`artifact-decomposition.md`](../../artifact-decomposition.md) — byte-level algorithm + marker profile
- [`README.md`](../../README.md) — artifact-layout examples
- [`base64-requirements.md`](../../base64-requirements.md) — signature scalar encoding

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the
Docker build and run commands. On a steady-state branch,
re-running the rebuilder MUST reproduce the bytes of every
fixture in this directory bit-identically; a non-empty diff
is either a generator defect or an intended spec change that
has propagated through the generator.

## How to use these fixtures

1. Read the artifact file (`*.yaml`) as raw bytes — do NOT pass through
   a YAML parser before invoking Decompose; this stage is byte-level.
2. Invoke `Decompose(form = TRANSCRIPTION_FORM_YAML)` on the artifact
   bytes.
3. Compare the returned `DecomposeOutcome` and (where `Ok`) the
   `payload` / `signature_carrier` byte ranges against the per-fixture
   expectations below.

## Fixtures

| File | Outcome | Verifier-state mapping | Notes |
| --- | --- | --- | --- |
| `signed-single-lf.yaml` | `Ok` | proceeds to verification | One-document payload + LF marker (`---\n`) + carrier. The signature octets are a placeholder (all-zero); these fixtures exercise byte-level decomposition only and do NOT exercise cryptographic verification. |
| `signed-single-crlf.yaml` | `Ok` | proceeds to verification | Same artifact shape as `signed-single-lf` but every line terminator is CRLF (`\r\n`) and the marker is `---\r\n`. |
| `signed-multi.yaml` | `Ok` | proceeds to verification | Multi-document payload (two YAML docs) + LF marker + carrier. Exercises `M = max(S)` — an earlier `---` inside the payload range is NOT the signing marker. |
| `empty-payload.yaml` | `Ok` (payload empty) | proceeds to verification | Artifact starts immediately with `---\n` at offset 0 → `payload_range = [0, 0)`. |
| `no-marker.yaml` | `Unsigned` | `Unsigned` | YAML stream with no constrained `---` marker anywhere. `|S| = 0`. |
| `extra-marker-inside-carrier.yaml` | `Ok` at Decompose; `MalformedAttemptedSigned` at Verification | `MalformedAttemptedSigned` | Tests `M = max(S)` selection. Two constrained markers are present; an implementation that incorrectly selected the FIRST marker would return a carrier that parses as a valid `YamlSigilSignature.v1alpha1`. The correct algorithm selects the LAST marker, yielding a carrier (`extra: trailer\n`) that fails Verification's metadata-extraction stage because it lacks the required `schema` / `alg` / `signature` fields. |
| `marker-at-eof-empty-body.yaml` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Marker present, but `signature_carrier_range` is empty (no body after the marker through EOF). Step 6 of the algorithm: `T = |A|` → `MalformedAttemptedSigned`. |
| `invalid-utf8-no-marker.yaml` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Invalid UTF-8 fails the encoding precondition before marker scanning. It MUST NOT be reported as `Unsigned`. |
| `invalid-utf8-before-marker.yaml` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | Invalid UTF-8 before an otherwise valid marker fails before marker selection. |
| `bom-signed.yaml` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | UTF-8 BOM octets `EF BB BF` at offset 0 fail before signed artifact processing. |
| `bom-no-marker.yaml` | `MalformedAttemptedSigned` | `MalformedAttemptedSigned` | UTF-8 BOM octets `EF BB BF` at offset 0 fail before no-marker handling. It MUST NOT be reported as `Unsigned`. |

Step 7 has no direct fixture because correct `M = max(S)` selection makes
its failure condition unreachable. `extra-marker-inside-carrier.yaml`
instead checks that implementations select the last marker.

## Byte-level reference

The constrained marker octets are:

| Form | Octets | Text |
| --- | --- | --- |
| LF | `2D 2D 2D 0A` | `---\n` |
| CRLF | `2D 2D 2D 0D 0A` | `---\r\n` |

See [Artifact Decomposition](../../artifact-decomposition.md) for the
full algorithm.
