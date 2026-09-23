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
- Developer-only ACVP refresh transport: `ureq` uses Rustls with the platform
  certificate verifier.

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
cargo xtask update-acvp [--commit <40-character-lowercase-commit>]
```

The optional commit must be a full 40-character lowercase hexadecimal Git
commit ID. The updater accepts only an HTTP 200 response over HTTPS, follows at
most five HTTPS redirects, uses the platform certificate verifier, and honors
supported `ALL_PROXY`, `HTTPS_PROXY`, `HTTP_PROXY`, and `NO_PROXY` environment
settings. It does not retry, requests identity encoding, and bounds response
headers, each network phase, the complete request, and the response body before
replacing either pinned file.

The xtask rewrites both the data file and the vendor `README.md` so the pin is
always self-describing. Downloads and replay are limited to a 3 MiB encoded
snapshot. Group, case, selected-replay, and decoded-field limits are documented
in that generated vendor README and exercised by exact-boundary and
limit-plus-one tests.

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
cargo xtask check
```

This includes repository Markdown, Protobuf, and JSON Schema checks, followed
by formatting, compilation, linting, tests, unused-dependency detection,
dependency policy, and audit checks for this locked Rust workspace. `ci` is an
alias for `check`.

Select checks with `cargo xtask check --only=fmt,clippy,test` or omit checks
with `--exclude=audit`. Selections run in registry order and stop on failure.
Use `--all-features`, `--features`, or `--no-default-features` to select Cargo
features for compilation, tests, and coverage; with no feature option, these
commands enable all features. Cargo Deny includes all features under the
repository's `deny.toml` policy even when compilation selects fewer features.

## Coverage

From the repository root or this workspace, generate a fresh HTML report:

```shell
cargo xtask coverage
cargo xtask coverage --engine tarpaulin
cargo xtask coverage --open
cargo xtask coverage-open --engine tarpaulin
```

Install the selected engine with `cargo install --locked cargo-llvm-cov` or
`cargo install --locked cargo-tarpaulin`. LLVM coverage also requires
`rustup component add llvm-tools-preview`. Both opening commands regenerate
the report before opening it. If no browser opener is installed, use the
printed report path.

Reports live beneath this workspace at
`target/coverage/llvm-cov/html/index.html` and
`target/coverage/tarpaulin/tarpaulin-report.html`. Tarpaulin builds in
`target/coverage/tarpaulin/build` so its cleanup cannot remove ordinary Cargo
builds. Coverage accepts the same feature options as `check`.

## Build and verify the local image

From the repository root or this workspace:

```shell
cargo xtask image
cargo xtask image --engine docker
cargo xtask image --engine buildah --tag yamlsigil-conformance-rebuild-rs
```

`auto` selects a usable Docker daemon first, then Buildah. An explicit engine
must work; a build failure does not switch engines. The command builds this
Dockerfile with the repository root as its context, runs the generator with
network access disabled, and compares every generated fixture byte with the
repository. It keeps the image and removes its temporary container and fixture
tree on success or failure. The default tag is
`yamlsigil-conformance-rebuild-rs`. Builds are local and never push an image.

The runtime image includes the pinned ACVP snapshot at `/src/vendor/acvp`,
where the generator reads it. The manual Docker and native rebuild commands
remain available above and in [`../README.md`](../README.md).

The full command and maintenance contract is in
[`xtask/AGENTS.md`](xtask/AGENTS.md).
