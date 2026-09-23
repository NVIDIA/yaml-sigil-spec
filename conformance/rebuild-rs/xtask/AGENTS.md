# Development task contract

This crate is a member of `conformance/rebuild-rs`, shares its committed
`Cargo.lock`, and has `publish = false`. The root Cargo alias selects this
manifest; the nested alias selects package `xtask`. Both use `--locked`.
The default workspace member remains the fixture generator.

Use Clap derive types for parsing and retain its `CommandFactory::debug_assert`
test. Keep `main.rs` limited to parsing, diagnostics, and exit status. Reusable
behavior belongs in the library. Keep command construction separate from
execution and pass operating-system argument vectors rather than shell strings.
Document public library behavior and errors. Preserve the generator's Linux
and mounted `/proc` requirements; do not add unsupported platform promises.

## Checks

`cargo xtask check` runs these registered checks in order:

1. `markdown`.
2. `buf-build`.
3. `buf-lint`.
4. `buf-fmt`.
5. `schema`.
6. `fmt`.
7. `check`.
8. `clippy`.
9. `test`.
10. `machete`.
11. `deny`.
12. `audit`.

`ci` is a visible alias over the same parser and execution path. With no
selector, run every check and stop on the first failure. Accept either
`--only=<csv>` or `--exclude=<csv>`, reject empty and unknown selections,
deduplicate names, and run selected checks in registry order.

Share feature options with coverage. With no explicit option, enable all
features. Forward `--features` and `--no-default-features` with Cargo semantics,
including their combination. `--all-features` conflicts with both. Preserve
locked workspace and target flags; formatting receives no feature flags.
Cargo Deny always uses `--all-features`, matching the repository's `deny.toml`
policy. Its checks include all features even when compilation, tests, or
coverage select fewer features.

`--candidate-root PATH` keeps the running validator in its original checkout.
Pin the candidate repository and rebuild workspace without following symlinks,
require a regular Cargo manifest, and keep both directory handles alive until
all selected checks finish. `candidate-preflight --candidate-root PATH` keeps
its bounded no-follow ACVP read before candidate execution.

Only probe prerequisites for selected operations. Distinguish missing programs
from programs that launch unsuccessfully, retain useful stderr, and include an
installation command when available. Keep Buf's `BUF` override, minimum CLI
version, and `buf-toolchain` guidance in `src/ci.rs`.

Keep workflow, script, and documentation callers aligned when commands change.
The xtask must not inspect or parse CI provider declarations. Hosted candidate
policy checks precede the terminal credential-free Rust phase; preserve that
boundary when replacing direct commands with selected xtask calls.

## Coverage

`coverage` defaults to LLVM and accepts `--engine llvm-cov|tarpaulin`.
`coverage --open` and `coverage-open` generate a fresh report, require that
report to exist, then open it. Keep reports beneath the rebuild workspace:

- LLVM HTML index: `target/coverage/llvm-cov/html/index.html`.
- Tarpaulin HTML: `target/coverage/tarpaulin/tarpaulin-report.html`.

Keep Tarpaulin builds in `target/coverage/tarpaulin/build`; its cleanup must
not remove ordinary Cargo outputs.

Install the selected engine with `cargo install --locked cargo-llvm-cov` or
`cargo install --locked cargo-tarpaulin`. LLVM also needs
`rustup component add llvm-tools-preview`. Report the absolute path if a
browser opener is unavailable; never install a desktop utility automatically.

## Local image

`image --engine auto|docker|buildah [--tag TAG]` owns one image with no owned
image prerequisites. `auto` probes Docker's daemon first and Buildah's runtime
second. Select one usable engine before building; never hide a build failure
by changing engines.

- Build file: `conformance/rebuild-rs/Dockerfile`.
- Build context: repository root.
- Default local tag: `yamlsigil-conformance-rebuild-rs`.
- Runtime data: pinned ACVP files at `/src/vendor/acvp`.
- Smoke operation: run `/usr/local/bin/rebuild_all` without network access,
  mounting a new fixture directory at `/work`, then compare its complete file
  inventory and every fixture byte with the repository.
- Cleanup: remove temporary containers and the smoke directory on success or
  failure; retain the local image. Never push an image.

The temporary fixture directory permits the image's UID 1000 to write under a
private host parent. Do not mount the source checkout writable for smoke tests.
Preserve the image's repository, Cargo dependency, Rust, and Debian notices.
Link to official [Docker](https://docs.docker.com/engine/install/) and
[Buildah](https://github.com/containers/buildah/blob/main/install.md)
instructions for installation and runtime troubleshooting.

## Maintenance and omissions

Keep `update-acvp [--commit HASH]`, its exact lowercase commit validation,
bounded HTTPS transport, proxy behavior, and anchored file replacement. Its
README template and checked-in output must match; do not fetch new data for an
unrelated documentation edit.

Profiling is omitted because the repository maintains a deterministic fixture
generator without a performance workload. MCP evaluation is omitted because
there is no MCP server.

Prefer Rust for typed, Cargo-aware orchestration. Keep the existing Python
binder, reporter, and their fixtures at their pre-checkout or checkout-free
trust boundaries; they must not compile candidate-controlled Rust. Keep the
shell helpers that materialize candidates and verify source pins. Inspect and
propose a migration, and obtain user approval, before replacing mature helpers
or changing their callers.

## Validation

Run `cargo xtask check` from the repository root before handoff. While
iterating, use a focused check selection. Retain tests for parser invariants,
selection order, feature forwarding, command construction, prerequisite errors,
report freshness, image-engine selection and cleanup, and all existing
candidate and ACVP security boundaries. Use fakes for external-tool failure
paths, then run representative local coverage and image commands when their
prerequisites are available. Keep all executable outputs local and ephemeral.
