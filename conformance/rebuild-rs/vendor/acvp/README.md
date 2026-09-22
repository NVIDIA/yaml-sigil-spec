# Vendored ACVP test vectors

This directory contains a pinned snapshot from the NIST
[Automated Cryptographic Validation Protocol (ACVP)](https://pages.nist.gov/ACVP/)
server's reference test vectors.

| Field | Value |
| --- | --- |
| Upstream repo | <https://github.com/usnistgov/ACVP-Server> |
| Commit | `15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0` |
| Upstream path | `gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json` |
| Browse on GitHub | <https://github.com/usnistgov/ACVP-Server/blob/15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0/gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json> |
| Vendored as | `vendor/acvp/ECDSA-SigGen-FIPS186-5.json` (2330483 bytes) |
| Pinned by | `xtask/src/main.rs` `DEFAULT_COMMIT` |

## Vector set

The snapshot contains Algorithm Functional Test (AFT) vectors for ECDSA
signature generation under FIPS 186-5. The values `(d, Q, k, message, r, s)`
identify the private key, public key, nonce, message, and expected signature
components. The rebuilder replays signing and requires byte equality with the
published `(r, s)`. It selects `curve = P-256` and `hashAlg = SHA2-256` groups
without a `conformance` tag from the snapshot's curve and hash combinations.

## Resource limits

Candidate preflight and the native rebuilder accept at most 3 MiB of encoded
JSON. The rebuilder also limits the corpus to 512 test groups, 64 cases per
group, and 4,096 cases in total. The selected P-256 / SHA2-256 replay accepts
at most eight groups and 256 cases. Scalar-like hex fields have a
160-character limit; messages have a 4,096-character limit; randomized-hashing
values have a 256-character limit.

The rebuilder reads one anchored no-follow snapshot and validates these limits
before retaining collections. It then deserializes the same bytes for replay.
A refresh outside these limits requires explicit review and a coordinated
limit change. `cargo xtask ci` tests every exact boundary and one value beyond
it.

The National Institute of Standards and Technology is explicitly
acknowledged as the source of this test data. The local file name
was changed; its contents were not modified. The NIST notice that
governs this snapshot is reproduced in
[`THIRD_PARTY_NOTICES.md`](../../../../THIRD_PARTY_NOTICES.md) and
must remain with distributions of this vendored file.

## Manual verification

From `conformance/rebuild-rs`, fetch the upstream file at the pinned commit
and compare its SHA-256 hash with the local snapshot:

```shell
# Compute the hash of the upstream file at the pinned commit.
curl -sL 'https://raw.githubusercontent.com/usnistgov/ACVP-Server/15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0/gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json' | sha256sum

# Compare against the hash of the vendored copy.
sha256sum vendor/acvp/ECDSA-SigGen-FIPS186-5.json
```

The two hashes MUST match. Investigate any mismatch before using the snapshot.

## Refreshing

To select a different default pin, edit `DEFAULT_COMMIT` in
`xtask/src/main.rs` and run:

```shell
cargo xtask update-acvp
```

To refresh from a full 40-character lowercase hexadecimal commit without
changing the default, pass it explicitly:

```shell
cargo xtask update-acvp --commit <40-character-lowercase-commit>
```

The updater accepts only an HTTP 200 response over HTTPS, follows at
most five HTTPS redirects, and uses the platform certificate verifier.
It honors supported HTTP and HTTPS proxy and `NO_PROXY` environment
settings, does not retry, requests identity encoding, and bounds
response headers, timeouts, and the 3 MiB response body before
replacing either pinned file.

The updater rewrites the JSON snapshot and this README. Edit the README
template in `xtask/src/main.rs`; do not edit the generated file by hand.
