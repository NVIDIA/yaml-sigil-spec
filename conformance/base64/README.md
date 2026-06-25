# Base64 Conformance Fixtures

Driving the [Base64 Requirements](../../base64-requirements.md) profile
for the YAML-form `signature` scalar: URL-safe alphabet, no padding,
no non-zero trailing bits.

## Upstream sources

- [RFC 4648 §5](https://www.rfc-editor.org/rfc/rfc4648#section-5) — URL-safe base64 alphabet
- [`base64-requirements.md`](../../base64-requirements.md) — the strict
  YamlSigil profile (no padding, no trailing bits)

## How to regenerate

Fixtures are produced by [`conformance/rebuild-rs`](../rebuild-rs/).
See the top-level [conformance README](../README.md) for the
Docker build and run commands. On a steady-state branch,
re-running the rebuilder MUST reproduce the bytes of every
fixture in this directory bit-identically; a non-empty diff
is either a generator defect or an intended spec change that
has propagated through the generator.

## How to use

For each fixture, run it through the YAML-form base64 decoder under
the [Base64 Requirements](../../base64-requirements.md) profile.
Compare against the expected outcome below.

For "valid" fixtures, the decoder MUST succeed and produce the
expected decoded byte count. For "invalid" fixtures, the decoder MUST
fail — which maps to `MalformedAttemptedSigned` at the verifier-state
layer.

## Fixtures

| File | Content | Expected decoder outcome | Notes |
| --- | --- | --- | --- |
| `valid-64-octet.txt` | 86-character URL-safe unpadded encoding of 64 zero-bytes | success; 64 decoded bytes | The canonical happy-path for an Ed25519 / ECDSA 64-octet signature. |
| `empty.txt` | (zero bytes) | success; 0 decoded bytes (but the content-layer non-empty `signature` rule then rejects) | Note: empty decoded octets are NOT a base64 decode failure under this profile. The verifier's verification stage rejects them. |
| `invalid-alphabet-plus.txt` | contains a `+` (standard-alphabet char, not URL-safe) | decode failure | The URL-safe profile uses `-` and `_`, not `+` and `/`. |
| `invalid-alphabet-slash.txt` | contains a `/` (standard-alphabet char, not URL-safe) | decode failure | Same as above. |
| `padding-present.txt` | ends with `=` | decode failure | The profile is unpadded; `=` is rejected. |
| `length-mod-4-eq-1.txt` | 85 characters (length mod 4 == 1) | decode failure | No valid base64 encoding has `len % 4 == 1`. |
| `whitespace-internal.txt` | valid 86-char encoding with a space inserted mid-string | decode failure | Strict decoders MUST reject internal whitespace. |
| `nonzero-trailing-bits.txt` | 86-character encoding whose final partial quantum has non-zero unused bits | decode failure | The load-bearing security check — see [Base64 Requirements](../../base64-requirements.md). Without this rejection, signature encodings become malleable. |

## Trailing-bits derivation (`nonzero-trailing-bits.txt`)

The 64-byte signature encodes to 86 base64 characters. The last two
characters (`AA` in the all-zero case) encode the final 12 bits, of
which 8 are the actual signature octet and 4 are padding bits that
MUST be zero. Replacing the final `A` with `B` flips one of those
padding bits from `0` to `1` — the decoded 64-byte value is identical,
but the encoded string differs. A strict decoder MUST reject the `B`
variant.
