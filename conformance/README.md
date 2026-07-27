# Conformance Tests

Static fixtures for `YamlSigil.v1alpha1` conformance testing. When you
edit Markdown under `conformance/`, follow the documentation **style
guide** in the repository-root [`AGENTS.md`](../AGENTS.md).

Each
subdirectory groups fixtures by the spec surface they exercise; each
subdirectory's own `README.md` describes the file inventory, expected
outcomes, and how to drive each fixture through an implementation.

These are reference inputs and expected verdicts. They do not depend on
any particular implementation or test harness — a conforming
implementation should produce the documented outcome for every fixture.

A negative `VerifyFromPreVerify` test is deliberately not included here;
`VerifyFromPreVerify` is an in-process / same-instance concept rather than
a data-side fixture concern. See [README](../README.md)'s "Known Deficiencies"
for the current tracking of that IDL deployment constraint.

Each per-directory README names the canonical upstream source so an
auditor can replace or extend the shipped fixtures with full
external-source coverage. Where a fixture is deliberately incomplete
or locally-generated, a `#### Compromise` block in the relevant
subsection below names what's missing and what additional coverage
the auditor should run. See also [`AGENTS.md`](./AGENTS.md) for the
process of updating these directories.

The applicable redistribution notices are collected in
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) and must remain with
distributed copies of the fixtures and generator.

## General Notes

- `.binpb` is used for protobuf-wire-format messages (raw serialized
  bytes; no text wrapper). Where the protobuf is `SignedYamlArtifact`,
  that's stated in the per-directory README.
- `.yaml` is used for full YAML-form artifacts (payload stream +
  constrained marker + signature document, per
  [Artifact Decomposition](../artifact-decomposition.md)).
- `.txt` is used for base64 / scalar inputs that aren't a full artifact.
- Where a fixture pins specific cryptographic values (RFC 8032 §7.1,
  NIST CAVP / ACVP), the file body either contains the exact value or
  references the canonical publication.
- Where a fixture's expected outcome is one of the verifier states
  ([Verification API](../verification-api.md)), the per-directory
  README spells out the expected state.

## Conformance Test Data

### YAML Decomposition (`yaml-decomposition/`)

- **YAML decomposition** — markerless carrier, LF and CRLF markers,
  UTF-8 and BOM preconditions, empty payload, extra marker inside
  carrier, no-marker artifact, marker at EOF.

See [yaml-decomposition/README.md](./yaml-decomposition/README.md).

### Protobuf conformance (`protobuf-conformance/`)

- **Protobuf conformance** — duplicate outer `payload`, duplicate
  outer `signature`, unknown outer fields, inner strict / permissive
  duplicate handling, present-empty outer `signature` submessage.

See [protobuf-conformance/README.md](./protobuf-conformance/README.md).

### YAML signature-document conformance (`yaml-signature-conformance/`)

- **YAML signature-document conformance** — duplicate `schema`,
  `alg`, `keyid`, `signature` mapping keys; unknown mapping key.
  The YAML-form symmetric set to the inner-`YamlSigilSignature`
  cases in `protobuf-conformance/`, driving the
  [Verification API](../verification-api.md) "Conformance Profiles"
  rules.

See [yaml-signature-conformance/README.md](./yaml-signature-conformance/README.md).

### Schema alignment (`schema-alignment/`)

- **Schema alignment** — YAML `alg` strings, protobuf enum values,
  unknown-string behavior, protobuf unknown-integer behavior.

See [schema-alignment/README.md](./schema-alignment/README.md).

### `keyid` Conformance (`key-id/`)

- **`keyid`** — absent, present-empty, exactly 1024 UTF-8 octets,
  oversized, multibyte (proves octet-count enforcement beyond JSON
  Schema's code-point `maxLength` approximation), and line-break
  rejection.

See [key-id/README.md](./key-id/README.md).

### Base64 Conformance (`base64/`)

- **Base64** — invalid alphabet, padding present, `len mod 4 == 1`,
  whitespace, non-zero trailing bits, empty decoded bytes, one valid
  64-octet signature.

See [base64/README.md](./base64/README.md).

### Ed25519 Conformance (`alg-ed25519/`)

- **Ed25519** — RFC 8032 §7.1 vectors, canonical rejection,
  configured-key `KeyResolutionFailure`, stable re-signing,
  `algorithm_parameters` rejection.

See [alg-ed25519/README.md](./alg-ed25519/README.md).

#### Compromise (Ed25519)

- **RFC 8032 §7.1 Test 2 ships only the protobuf form.** Test 2
  specifies the full payload as the single byte `0x72`. The YAML
  form cannot represent it (the constrained marker profile requires
  the byte before the marker to be a line terminator). The protobuf
  form has no such constraint; `verification-api.md`'s
  metadata-extraction table now spells out that YAML-envelope
  payload rules do NOT apply to the protobuf form, so this is a
  spec-normative carve-out rather than a per-fixture compromise.
  See also the protobuf-conformance fixture
  `binary-payload-no-yaml-fit.binpb` for a minimal example of the
  same carve-out and `transcoding.md`'s round-trip table for the
  asymmetric-transcoding consequence.
- **No corpus of "implementation drift" fixtures.** The Ed25519
  fixture set covers canonical-encoding rejection and small-order
  configured-key rejection (the strict-variant rules). It does NOT
  attempt to cover every disagreement between popular Ed25519
  implementations enumerated in ["Taming the Many
  EdDSAs"](https://eprint.iacr.org/2020/1244); auditors targeting
  cross-library interop SHOULD additionally run that paper's full
  test suite against an implementation.

### ECDSA Conformance (`alg-ecdsa/`)

- **ECDSA** — CAVP / ACVP vectors, high-S and low-S acceptance,
  invalid component ranges, invalid public keys, deterministic
  two-nonce instability, `algorithm_parameters` rejection.

See [alg-ecdsa/README.md](./alg-ecdsa/README.md).

#### Compromise (ECDSA)

- **Happy-path is NIST-anchored; the rest are locally-generated.**
  The `acvp-fips186-5-p256-sha256-tc131.binpb` fixture (and its
  `.expected.txt`) is sourced from the NIST ACVP-Server's FIPS 186-5
  ECDSA SigGen AFT vectors at a pinned commit hash — see
  [`rebuild-rs/vendor/acvp/README.md`](./rebuild-rs/vendor/acvp/README.md).
  The rebuilder's test suite replays **every** P-256 / SHA-256 AFT
  case in that vendored file through our hand-rolled signer and
  asserts byte-equality with the published `(R, S)`.
- **High-S / low-S, range-rejection, two-nonce-instability,
  bad-key, and the YAML-envelope happy-path remain locally-generated.**
  These cases either require shapes ACVP doesn't supply (two distinct
  nonces over the same `(d, message)`, deliberately-malformed
  artifact bytes, bad-key encodings) or constrain the payload to
  YAML-printable ASCII (the ACVP messages are random binary). They
  prove *structural* conformance to the algorithm spec but are NOT a
  substitute for the broader official NIST corpus. Implementations
  targeting full FIPS conformance SHOULD additionally run against the
  complete
  [CAVP digital-signature vectors](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/digital-signatures)
  and / or the [ACVP test definitions](https://pages.nist.gov/ACVP/).

## Rebuilding all fixtures (Docker)

All generator source lives under [`rebuild-rs/`](./rebuild-rs/) with
pinned dependencies in `Cargo.toml` + `Cargo.lock`. The Dockerfile
produces a self-contained container that runs as UID `1000` and
writes regenerated fixtures into a mounted `conformance/` tree.

**Build (run from the repository root):**

```sh
docker build -t yamlsigil-conformance-rebuild-rs conformance/rebuild-rs
```

**Rebuild every fixture in place (run from `conformance/`):**

```sh
cd conformance
docker run --rm \
    -v "$(pwd):/work" \
    yamlsigil-conformance-rebuild-rs
```

**Running without Docker (local `cargo`):**

```sh
cd conformance/rebuild-rs
CONFORMANCE_ROOT="$(realpath ..)" cargo run --release --locked
```

The container entrypoint iterates each subdirectory under `/work`
and regenerates its fixtures. On a steady-state branch, re-running
MUST produce bit-identical output for everything currently shipped;
a non-empty `git diff` after running is either a generator defect or
an intended spec change that's now propagated into the fixtures. On
a branch that is updating the spec, the fixture diff IS part of the
spec change — the generator is authoritative either way; never edit
a fixture by hand.

If the host's effective UID isn't `1000`, pre-`chown` `conformance/`
or pass `--user "$(id -u):$(id -g)"` to `docker run`.

See [`rebuild-rs/README.md`](./rebuild-rs/README.md) and
[`AGENTS.md`](./AGENTS.md) for the source layout and maintenance
contract.
