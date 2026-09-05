# conformance/rebuild-rs

Generator source for `conformance/` fixtures. External Cargo dependencies are
minimal and exact-pinned in `Cargo.toml`, grouped by role:

- Crypto primitives (used by the hand-rolled implementations against
  cited upstream specs): `sha2`, `num-bigint`, `num-integer`,
  `num-traits`.
- Vendored ACVP test-vector ingestion (JSON parsing of the pinned NIST
  ACVP-Server snapshot under `vendor/`): `serde`, `serde_json`.
- Input and output isolation: the repository-local `yamlsigil-pinned-dir`
  crate anchors no-follow ACVP reads and pins output directories before
  replacing fixture or vendor files.

The full transitive graph is locked in `Cargo.lock`. The Docker image
defined here packages everything needed to rebuild every fixture in
`conformance/` bit-identically.

The generator incorporates identified standards and test-vector material that
is not relicensed under Apache-2.0. Its source attribution, copying conditions,
warranty disclaimers, and patent/IP caveats are recorded in
[`../../THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md).

See [`../README.md`](../README.md) for the Docker build and run
instructions. The repository root is the build context so the image can
include `LICENSE` and `THIRD_PARTY_NOTICES.md`.

The image carries those two files, license files collected automatically from
the locked Cargo dependencies, and the exact pinned Rust standard-library
notice bundle under `/usr/share/doc/yamlsigil-conformance-rebuild/`. Notices
installed by Debian packages remain under
`/usr/share/doc/<package>/copyright`; the Dockerfile does not copy or maintain
them separately.

See [`../AGENTS.md`](../AGENTS.md) for the maintenance contract,
including the rustdoc-citation, testing, and `cargo fmt` / `clippy` /
`test` requirements that every source file here MUST satisfy.

## Layout

```text
rebuild-rs/
├── Dockerfile
├── Cargo.toml
├── Cargo.lock
├── README.md           (this file)
├── .cargo/config.toml  (cargo xtask alias)
├── src/
│   ├── main.rs               (entry point — runs every subdirectory's generator)
│   ├── wire.rs               (protobuf wire-format helpers)
│   ├── b64.rs                (URL-safe unpadded base64)
│   ├── util.rs               (write helpers + hex)
│   ├── p256.rs               (hand-rolled P-256 ECDSA over num-bigint)
│   ├── acvp.rs               (NIST ACVP-Server JSON parser; used by alg_ecdsa.rs)
│   ├── yaml_decomposition.rs
│   ├── protobuf_conformance.rs
│   ├── yaml_signature_conformance.rs
│   ├── schema_alignment.rs
│   ├── transcoding.rs
│   ├── verification_runtime.rs
│   ├── key_id.rs
│   ├── base64_gen.rs
│   ├── alg_ed25519.rs        (RFC 8032 §7.1 vectors verbatim)
│   └── alg_ecdsa.rs          (uses sha2 for SHA-256, p256.rs for the curve)
├── pinned-dir/         (repository-local pinned-directory write helper)
├── xtask/              (developer-only tasks; see below)
└── vendor/
    └── acvp/           (vendored NIST ACVP-Server vectors)
```

### Vendored upstream data

The `vendor/` tree carries pinned snapshots of external test-vector
files. Each subdirectory has its own `README.md` describing the
upstream origin, pinned commit hash, and manual-verification
commands. Currently:

- [`vendor/acvp/README.md`](./vendor/acvp/README.md) — NIST
  ACVP-Server ECDSA SigGen FIPS 186-5 test vectors.

To refresh a vendored file, use the corresponding xtask subcommand:

```sh
cargo xtask update-acvp [--commit <hash>]
```

The xtask rewrites both the data file and the vendor `README.md` so
the pin is always self-describing. Downloads and replay are limited to a
3 MiB encoded snapshot. Group, case, selected-replay, and decoded-field limits
are documented in that generated vendor README and exercised by
exact-boundary and limit-plus-one tests.

## Running locally without Docker

The native flow requires Linux and a mounted `/proc`. Use the Docker workflow
from [`../README.md`](../README.md) on other host operating systems.

```sh
cd conformance/rebuild-rs
CONFORMANCE_ROOT="$(realpath ..)" cargo run --release --locked
```

`CONFORMANCE_ROOT` defaults to `/work` (the Docker mount point); set
it explicitly when running outside the container.

## Local validation

Run the complete validation sequence from the repository root:

```shell
cargo xtask ci
```

This includes repository Markdown, Protobuf, and JSON Schema checks, followed
by formatting, linting, tests, and a dependency audit for this locked Rust
workspace.
