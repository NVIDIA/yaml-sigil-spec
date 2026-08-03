# Conformance directory maintenance

This file describes how to update fixtures under `conformance/`. When
you edit Markdown here (including per-subdirectory `README.md` files),
follow the documentation **style guide** in the repository-root
[`AGENTS.md`](../AGENTS.md), including its GitHub Flavored Markdown
dialect target.

All
generator source lives under [`rebuild-rs/`](./rebuild-rs/), a Rust
crate with dependencies pinned in `Cargo.toml` + `Cargo.lock`. The
repository policy is: **the `rebuild-rs` tree is authoritative for
fixture content**. If a fixture and its generator disagree, the
generator wins; re-run the rebuild container (or the local `cargo`
flow in `rebuild-rs/README.md`) to regenerate the file.

## Audit principle

An auditor reading this repository SHOULD be able to validate every
shipped fixture against its upstream source without trusting this
repository's commit history. Concretely:

1. The fixture's expected outcome is named in the per-directory
   `README.md`.
2. The fixture's input is regenerable from canonical sources cited
   in the same `README.md` (RFC 8032, FIPS / NIST CAVP / ACVP,
   *Standards for Efficient Cryptography 1 (SEC 1)*, etc.) using the
   matching generator module under
   [`rebuild-rs/src/`](./rebuild-rs/src/).
3. External generator dependencies are minimal and exact-pinned in
   `rebuild-rs/Cargo.toml`, grouped by role:
   - Crypto primitives (used by the hand-rolled implementations
     against cited upstream specs): `sha2`, `num-bigint`,
     `num-integer`, `num-traits`.
   - Vendored ACVP test-vector ingestion (JSON parsing of the
     pinned NIST ACVP-Server snapshot under `rebuild-rs/vendor/`):
     `serde`, `serde_json`.
   - Output isolation uses the repository-local
     `yamlsigil-pinned-dir` crate. It pins each output directory before
     replacing a fixture or vendor file.

   No wholesale crypto library is trusted for fixture content;
   ECDSA point arithmetic and Ed25519 vector handling are
   hand-rolled against the cited upstream specs.
4. Re-running the generator MUST produce bit-identical output for
   every shipped fixture, **modulo intentional spec changes that
   have been propagated through the generator in the same change**.
   In a steady-state branch, diff-clean regeneration is the
   conformance contract for the *fixtures themselves*; in a branch
   that is updating the spec, the same `cargo run` produces the new
   bytes and the fixture diff IS the spec change. Either way, the
   generator is authoritative — never edit a fixture by hand.

If you find a fixture that you cannot reproduce from the documented
upstream + generator, treat it as a defect to file against this
repository, not as authoritative data.

## Source layout

```text
conformance/
├── README.md            (top-level fixture index + Docker build/run)
├── AGENTS.md            (this file)
├── rebuild-rs/          (Rust generator source)
│   ├── Dockerfile       (UID 1000, rust:1.95.0-trixie)
│   ├── Cargo.toml
│   ├── Cargo.lock       (pinned deps)
│   ├── README.md
│   ├── .cargo/
│   │   └── config.toml        (cargo xtask alias)
│   ├── src/
│   │   ├── main.rs            (entry point)
│   │   ├── wire.rs            (protobuf helpers)
│   │   ├── b64.rs             (URL-safe unpadded base64)
│   │   ├── util.rs            (write helpers + hex)
│   │   ├── p256.rs            (hand-rolled P-256)
│   │   ├── acvp.rs            (NIST ACVP-Server JSON parser; used by alg_ecdsa.rs)
│   │   ├── yaml_decomposition.rs
│   │   ├── protobuf_conformance.rs
│   │   ├── yaml_signature_conformance.rs
│   │   ├── schema_alignment.rs
│   │   ├── transcoding.rs
│   │   ├── verification_runtime.rs
│   │   ├── key_id.rs
│   │   ├── base64_gen.rs
│   │   ├── alg_ed25519.rs
│   │   └── alg_ecdsa.rs
│   ├── pinned-dir/            (repository-local pinned-directory writes)
│   ├── vendor/                (pinned upstream snapshots)
│   │   └── acvp/              (NIST ACVP-Server ECDSA SigGen vectors)
│   └── xtask/                 (developer-only commands; workspace member,
│                               excluded from default-members so a bare
│                               `cargo build` builds only the rebuilder)
└── <subdir>/            (one per spec surface; fixture data only)
    ├── README.md        (inventory, expected outcomes, upstream sources)
    └── <fixture files>
```

The per-subdirectory READMEs document **inventory and expected
outcomes**; the generator modules contain the **construction logic**.
The two halves are intentionally separated so updating one without
the other surfaces as a diff.

## How to update a subdirectory

For any spec change that touches conformance behavior:

1. Identify the affected subdirectory (or subdirectories — many
   changes touch more than one).
2. Update the corresponding generator module under
   `rebuild-rs/src/` if the change alters input bytes, expected
   outcomes, or upstream-source references.
3. Re-run the rebuild (Docker or local `cargo` — see
   `conformance/README.md`). Confirm the new output matches the
   intent of the spec change.
4. Update the subdirectory's `README.md` if the fixture inventory,
   expected outcomes, or upstream sources changed.
5. If the spec change introduces a new fixture surface entirely
   (e.g., a new algorithm slot): add a new module under
   `rebuild-rs/src/`, register it in `main.rs`'s `SUBDIRS` const,
   create the new subdirectory with its `README.md`, and add a
   top-level entry in [`conformance/README.md`](./README.md).
6. Run the mechanical-sanity checks named in the top-level
   [`AGENTS.md`](../AGENTS.md) (`buf lint`, `buf format`,
   `jq empty`, `rumdl check`, link sweep). Conformance fixtures are downstream of
   the IDL and schema; a fixture cannot be valid if the upstream
   artifacts don't parse.
7. Run the Rust-toolchain checks from inside `rebuild-rs/`:
   `cargo fmt --all --check`,
   `cargo clippy --workspace --all-targets`, and
   `cargo test --workspace`. All three MUST be clean before landing the
   change.
8. If `Cargo.toml`, `Cargo.lock`, a container base image, or an installed
   container package changes, audit the resulting build and runtime
   dependency graph. The Docker build automatically collects registry-crate
   license files and preserves Debian package notices. Verify that collection,
   and update the repository-root `THIRD_PARTY_NOTICES.md` when the change
   adds checked-in or derived third-party material or requires a
   repository-level explanation.

## What requires a conformance update

The repository root [`AGENTS.md`](../AGENTS.md) names this rule
normatively. Briefly, any change in the following list MUST be
followed by a conformance pass in the affected subdirectory:

| Spec change | Affected subdirectory |
| --- | --- |
| Constrained marker profile, decomposition algorithm, payload-range rules | `yaml-decomposition/` |
| Outer `SignedYamlArtifact` wire layout, malformed protobuf wire rules, `OuterConformance` enum values, Conformance Profile rules as they manifest on the protobuf `YamlSigilSignature` decode | `protobuf-conformance/` |
| YAML signature-carrier safety rules, including byte and parser-resource bounds, duplicate known keys, unknown keys, and tag handling | `yaml-signature-conformance/` |
| Runtime algorithm support and cryptographic result mapping | `verification-runtime/` |
| Canonical YAML carrier and YAML ↔ protobuf round-trip rules | `transcoding/` |
| `Algorithm` enum membership, YAML `alg` string mapping, schema-unknown handling | `schema-alignment/` |
| `keyid` bounds, structural rules, JSON Schema vs decoder-level enforcement | `key-id/` |
| `base64-requirements.md` (alphabet, padding, trailing-bits rule) | `base64/` |
| `algorithms/01-ED25519_*` content (signing rules, canonical-encoding rules, key admissibility, parameters) | `alg-ed25519/` |
| `algorithms/02-ECDSA_*` content (same scope as Ed25519) | `alg-ecdsa/` |

A change touching the top-level glossary, the abstract Artifact
contract, or the five verifier states will typically require updates
in multiple subdirectories — start with the most affected one and
work outward.

## Per-fixture-file conventions

- `<fixture>.binpb` — raw protobuf wire bytes for `SignedYamlArtifact`.
- `<fixture>.yaml` — full YAML-form artifact (payload + marker +
  signature document).
- `<fixture>.txt` — base64 scalar, raw hex, or sidecar plain-text
  documentation.
- `<fixture>.expected.txt` — per-fixture sidecar describing the
  expected verifier outcome and the configured-key / nonce inputs
  used when generating the fixture. Auditors reading this file
  should be able to reproduce the verification call exactly.

Fixture subdirectories do NOT contain any code. Generator source
lives only under `rebuild-rs/`.

## Rust-source conventions

Every Rust source file under `rebuild-rs/src/` MUST:

1. **Cite the upstream document for every behavior or magic number
   it contains.** Citations live in rustdoc comments (`//!` for
   module-level, `///` for items) and SHOULD:
   - Link to the upstream as closely as possible. Easier for RFCs
     (deep-linked HTML at `https://www.rfc-editor.org/rfc/rfcN#section-X.Y`)
     than for NIST PDFs; cite the section / subsection in either case.
   - Quote only the upstream text needed to establish the behavior or
     constant. Use a Markdown blockquote (`>`) inside the rustdoc when
     quoting. Two or three paragraphs is the cap; prefer a tighter excerpt
     and link over reproducing a full table or section.
   - State explicitly that **these are the exact numbers / methods
     being used** in the implementation that follows.
   - If the citation includes a fenced code block, use ` ```text `
     (or `​```plaintext`) so `cargo test` does not try to compile it
     as a doctest. Bare ` ``` ` MUST be avoided.
2. **Carry reasonable unit tests.** Test files live alongside the
   module in a `#[cfg(test)] mod tests { ... }` block at the bottom
   of the file. The bar is: cover the primitives the module exposes
   (encoder / decoder pairs, arithmetic identities, known-answer
   vectors). Test data SHOULD come from the same upstream citation
   used in the rustdoc.
3. **Pass `cargo fmt`, `cargo clippy`, and `cargo test`.** All three
   MUST be clean (no `-D warnings` override needed; clippy at its
   default-warn level MUST emit no findings) before the change can
   land.

## Why generators, not check-in-only?

A generator-anchored approach buys three things:

1. **Audit replay.** An auditor can re-run the generator and diff the
   output against the shipped fixtures. If anything differs, either
   the generator is wrong or the fixture has been tampered with.
   Both are findings worth surfacing.
2. **Upstream-source freshness.** When an upstream (RFC, NIST, or
   *Standards for Efficient Cryptography 1 (SEC 1)*) publishes a new version
   that affects vectors or rules, the generator is where the change lands
   first; the fixtures are downstream artifacts that re-derive automatically.
3. **Cross-implementation honesty.** Locally-generated fixtures
   (e.g. the ECDSA happy-path with hand-rolled signing) are flagged
   as locally-generated in the `README.md` and the generator carries
   the parameters used. There's no "magic numbers" anywhere — every
   value either comes from an upstream citation or from a generator
   step that's reproducible from documented inputs.

## Edit-ownership note

The `conformance/` tree is open for editing by any agent or human
working on this repository. The constraint is the audit-replay property
above: changes MUST keep the generator-fixture pair consistent and MUST
keep the upstream-source citation accurate.
