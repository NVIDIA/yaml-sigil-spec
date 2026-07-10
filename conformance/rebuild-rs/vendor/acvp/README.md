# Vendored ACVP test vectors

This directory tracks a single file from the NIST
[Automated Cryptographic Validation Protocol (ACVP)](https://pages.nist.gov/ACVP/)
server's reference test-vector tree.

| Field | Value |
| --- | --- |
| Upstream repo | <https://github.com/usnistgov/ACVP-Server> |
| Commit | `15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0` |
| Upstream path | `gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json` |
| Browse on GitHub | <https://github.com/usnistgov/ACVP-Server/blob/15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0/gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json> |
| Vendored as | `vendor/acvp/ECDSA-SigGen-FIPS186-5.json` (2330483 bytes) |
| Pinned by | `xtask/src/main.rs` `DEFAULT_COMMIT` |

## What this is

A NIST ACVP "AFT" (Algorithm Functional Test) vector set
for ECDSA signature generation under FIPS 186-5. Each test
group pins `(d, Q, k, message, r, s)` — i.e. the private key,
public key, ephemeral nonce, message, and expected signature
components — so the rebuilder can replay the sign and assert
byte-equality against the published `(r, s)`. The file
covers multiple curve / hash combinations; the rebuilder
filters for `curve = P-256` and `hashAlg = SHA2-256`.

The National Institute of Standards and Technology is explicitly
acknowledged as the source of this test data. The local file name
was changed; its contents were not modified. The NIST notice that
governs this snapshot is reproduced in
[`THIRD_PARTY_NOTICES.md`](../../../../THIRD_PARTY_NOTICES.md) and
must remain with distributions of this vendored file.

## Manual verification

To confirm the vendored bytes by hand, fetch the same file at
the pinned commit and compare SHA-256 hashes:

```sh
# Compute the hash of the upstream file at the pinned commit.
curl -sL 'https://raw.githubusercontent.com/usnistgov/ACVP-Server/15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0/gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json' | sha256sum

# Compare against the hash of the vendored copy.
sha256sum vendor/acvp/ECDSA-SigGen-FIPS186-5.json
```

The two outputs MUST match. If they don't, the vendored file
has drifted from its upstream pin and that diff is itself a
finding to surface.

## Refreshing

To bump the pin to a newer commit, edit `DEFAULT_COMMIT` in
`xtask/src/main.rs` and run:

```sh
cargo xtask update-acvp
```

Or pass an explicit commit hash (without bumping the default):

```sh
cargo xtask update-acvp --commit <hash>
```

The xtask rewrites both the JSON file and this README. This
file is regenerated on every run; do not edit it by hand.
