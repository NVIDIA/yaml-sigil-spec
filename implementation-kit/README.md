# Language Implementation Kit

> [!WARNING]
> This directory is non-normative. The specification, API companions,
> protobuf IDL, JSON Schema, and conformance fixture documentation are
> authoritative. If this kit disagrees with an authoritative source, follow
> the authoritative source.

This source-only kit helps start an independent implementation of
`YamlSigil.v1alpha1` in any language. It provides protobuf generation templates
for Go, Python, and TypeScript, a reusable implementation prompt, and a
machine-readable fixture index. It does not define another API, wire format,
schema, or conformance-runner protocol.

## Start here

Point an agent at this kit:

```text
I want to build an implementation of yaml-sigil in <LANGUAGE>. Use this kit
and the prompt from here:
https://github.com/NVIDIA/yaml-sigil-spec/tree/main/implementation-kit
```

The rest of this document and
[`implementation-prompt.md`](./implementation-prompt.md) describe what that
work involves. Read them together with [AGENTS.md](./AGENTS.md), the entry
point for agents with a local copy. All three provide complementary guidance
for the same implementation task.

## Contents

| File | Purpose |
| --- | --- |
| [`AGENTS.md`](./AGENTS.md) | Agent entry point and guidance for using or maintaining the kit. |
| [`implementation-prompt.md`](./implementation-prompt.md) | Reusable prompt for a language implementation. |
| [`buf.gen.go.yaml`](./buf.gen.go.yaml) | Go protobuf message generation. |
| [`buf.gen.python.yaml`](./buf.gen.python.yaml) | Python protobuf message and type-stub generation. |
| [`buf.gen.typescript.yaml`](./buf.gen.typescript.yaml) | TypeScript protobuf message generation. |
| [`conformance-manifest.json`](./conformance-manifest.json) | Non-normative fixture discovery and expectation index. |

## Track your inputs

An implementation may draw on three NVIDIA published repositories:

- The authoritative
  [`yaml-sigil-spec`](https://github.com/NVIDIA/yaml-sigil-spec)
  specification.
- [`yaml-sigil-traits`](https://github.com/NVIDIA/yaml-sigil-traits), a Rust
  declaration of the API.
- The [`yaml-sigil-rs`](https://github.com/NVIDIA/yaml-sigil-rs) reference
  implementation.

It may also draw upon others; these are just the three we know about at
time of writing.

Following `main` for the spec is fine while getting started (some
corrections and clarifications land after a published version label).

### Ratcheting down the inputs

Once results need to be reproducible either for reconciling conformance fixture
failures, publishing, or making a conformance claim, record the exact commit IDs
for all used inputs (short commit hashes are fine for legibility). Move off of
floating refs and into exact commits to describe or diagnose problems or to
make a statement about the implementation referencing them.

**The specification's commit is authoritative if the repositories disagree.**

### Local copy

Cloning [`yaml-sigil-spec`](https://github.com/NVIDIA/yaml-sigil-spec) locally
is worth doing at the start. The specification documents, conformance fixtures,
and templates are all faster to search and read from disk than over the network.
The clone does not need to be tracked in the implementation's own history; if
tracking it is useful, a submodule works anywhere `git` does. Use better
dependency tooling instead wherever the target ecosystem provides it.

### Reference materials

The Rust implementations `yaml-sigil-rs` and `yaml-sigil-traits`, from the
perspective of implementing in a new language, are just references for what
worked well in Rust. They may or may not be the right design principles for
your language and/or implementation.

## Start from the interface

The specification's `proto/` tree declares the operations, messages, enums, and
error categories an implementation exposes. That IDL is the source for the API
surface.

`yaml-sigil-traits` declares the same surface in Rust. It is useful directly if
the target is Rust, and as a cross-check otherwise. Its name refers to the Rust
construct used to express it, so express the surface with whatever the target
language uses for an interface contract — interfaces, protocols, abstract base
classes, concepts, comptime interfaces, an explicit vtable struct — rather than
reproducing traits in a language that has none.

It also carries synchronous and asynchronous forms of the operations, which Rust
expresses as paired declarations. Other ecosystems draw that line differently or
not at all: much Go code is synchronous, JavaScript is asynchronous throughout,
and some languages express both through one signature. Provide the forms the
target language expects rather than reproducing the pairing.

## Generate protobuf messages

Any protobuf tooling that can compile the `proto/` tree will do. The examples
here use [`buf`](https://github.com/bufbuild/buf), which handles module and
plugin wiring that `protoc` leaves to the caller, so it is an easier starting
point. Nothing about the specification requires it; use whatever protobuf
tooling the target ecosystem already relies on.

The templates provided here are
[`buf.gen.yaml` v2 configuration files](https://buf.build/docs/configuration/v2/buf-gen-yaml/)
covering three languages. Plugins for many more are listed in the
[protobuf plugin directory](https://buf.build/plugins/protobuf).

The templates use Buf's
[remote plugins](https://buf.build/docs/bsr/remote-plugins/usage/). Their
`remote: buf.build/...` entries run generators on the Buf Schema Registry,
so you do not need to install `protoc` or the language-specific generator
executables locally. When a suitable off-the-shelf plugin is available and
remote generation fits your project, this avoids extra protobuf toolchain
setup. You still need Buf, network access for generation, and the
[generated-code runtime dependencies](#add-generated-code-runtimes).
Use your ecosystem's tooling when it calls for a different setup; the
[local-generator alternatives](#substitute-local-generators) remain available.

Each template reads the `proto/` tree from the specification repository over
Git, so generation needs no checkout of the specification itself:

```shell
kit_output_dir="$(mktemp -d)"
buf generate --template implementation-kit/buf.gen.go.yaml \
  --output "$kit_output_dir"
```

Use `buf.gen.python.yaml` or `buf.gen.typescript.yaml` in the same command for
the other languages. Send output to a disposable directory or one owned by the
downstream project — where generated protobuf stubs belong differs by language
and project. Do not add generated output to this specification repository.

These example `buf.gen.*.yaml` files may or may not be right for your development
environment. They serve as an example only and starting place.

The templates track `main`. Replace `branch: main` with `tag: <release>` or
`ref: <commit>` once the inputs need to be reproducible.

The templates pin these public Buf plugins and revision `1`:

| Language | Plugins |
| --- | --- |
| Go | [`protocolbuffers/go:v1.36.12`](https://github.com/bufbuild/plugins/tree/main/plugins/protocolbuffers/go/v1.36.12). |
| Python | [`protocolbuffers/python:v36.0`](https://github.com/bufbuild/plugins/tree/main/plugins/protocolbuffers/python/v36.0) and [`protocolbuffers/pyi:v36.0`](https://github.com/bufbuild/plugins/tree/main/plugins/protocolbuffers/pyi/v36.0). |
| TypeScript | [`bufbuild/es:v2.14.0`](https://github.com/bufbuild/plugins/tree/main/plugins/bufbuild/es/v2.14.0). |

The Go template uses `paths=source_relative`. Its managed-mode
`go_package_prefix` carries the sentinel
`example.invalid/replace-with-your-module`, which is not a publishable import
path. Copy the template into the implementation's own tree and set the real Go
module path in that copy, leaving the kit's copy alone.

Each template excludes `SigningService`, `TranscriptionService`, and
`VerificationService` by fully qualified type name. The request and response
messages remain generated, including `VerifyFromPreVerifyRequest` and
`VerifyFromPreVerifyResponse`. Treat them as in-process data types, not as
transport bindings. Generated output must contain no service descriptors,
gRPC stubs, or Connect stubs.

## Plan around the known deficiencies

The specification's [Known Deficiencies](../README.md#known-deficiencies) are
the places where it does not expect an implementation to do what a careful
reader might otherwise attempt. Read them before designing around a conformance
profile.

**Strict inner-protobuf conformance is not reachable with stock tooling.**
Generated protobuf code cannot reject unknown fields and duplicate known
singular fields, and no widely used protobuf implementation offers this. An
implementation in that position advertises `Permissive`, which the specification
treats as the honest answer rather than a shortfall. Do not spend effort
engineering toward `Strict` or `SignatureStrict` with a stock decoder; advertise
what the decoder actually does.

**The two forms spell algorithm names differently**, and the YAML form rejects
the protobuf-prefixed spelling. The
[schema-alignment suite](../conformance/schema-alignment/README.md) covers this
with fixtures. Map between the spellings rather than reusing a generated enum
value name as a YAML `alg`.

Follow the specification and the linked fixture documentation for the exact
rules in both cases. This kit is non-normative and deliberately does not restate
them.

> [!IMPORTANT]
> The pre-verification helpers — `CanPreVerify`, `PreVerify`, and
> `VerifyFromPreVerify` — are required by the specification, but
> `VerifyFromPreVerify` is itself listed under Known Deficiencies: it exists in
> the IDL for generated-code consistency, not as a deployment surface. The whole
> family is proposed for removal in `v1alpha2`
> ([yaml-sigil-spec#72](https://github.com/NVIDIA/yaml-sigil-spec/issues/72)).
>
> They are worth implementing where artifacts are large enough that parsing once
> and reusing prepared state is a real saving. Most workloads never reach that,
> and for those the helpers add surface without buying anything.
>
> That makes this a decision to take deliberately with whoever commissioned the
> implementation, rather than by default in either direction. Omitting them is a
> deviation from the current specification, and must be recorded as one.
>
> If they are implemented, `VerifyFromPreVerify` stays in-process on the same
> verifier instance, over opaque state that instance produced. Do not register
> it over gRPC, Connect, HTTP, IPC, or another public transport.

## Substitute local generators

Remote generation sends the protobuf image to the selected public Buf plugin.
When local generation is required, keep the output, options, and
`exclude_types` entries unchanged, remove `revision`, and replace the plugin
selector:

| Language | Local selector |
| --- | --- |
| Go | Replace `remote` with `local: protoc-gen-go`. |
| Python | Replace the two `remote` entries with `protoc_builtin: python` and `protoc_builtin: pyi`, and set `protoc_path` on each entry. |
| TypeScript | Replace `remote` with `local: protoc-gen-es`. |

Pin the local generator versions through the downstream project's toolchain.
The local versions must implement the options used by the corresponding
template.

## Add generated-code runtimes

Generated source needs a language runtime that the generator itself does not
vendor:

| Language | Downstream runtime dependency |
| --- | --- |
| Go | `google.golang.org/protobuf`. |
| Python | The `protobuf` distribution that satisfies the generated runtime-version check. |
| TypeScript | `@bufbuild/protobuf`. |

Pin and review those dependencies under the downstream project's dependency
policy. The message-only templates do not require a gRPC or Connect runtime.

Generated protobuf types implement serialization, field access, and descriptor
metadata. They do not implement YamlSigil semantics. In particular, they do
not perform Artifact Decomposition, YAML signature-carrier safety checks,
conformance-profile handling, base64 validation, algorithm validation,
cryptographic verification, verified-payload isolation, or reader-side trust
policy.

## Structure for what comes later

If the goal is a general-purpose library or SDK, a few boundaries are cheap to
draw at the start and expensive to introduce afterward, because adding them
later changes an API other people already depend on. For a demo, a spike, or an
implementation with one known caller, none of this needs deciding now.

**Keep generated types out of the public API.** Exchange wire bytes, or the
implementation's own types, at the boundary consumers see. Generated code then
stays an internal detail, and the protobuf library, its version, or the
generation options can change without a breaking release. Exposing generated
types directly forfeits that.

**Reach signing through an abstraction rather than an in-process key.** A
signer that accepts a key object hardwires the assumption that the private key
is available in memory. Defining signing as an operation that takes a handle and
returns signature octets leaves room for an HSM, a KMS, or a remote signing
service later without reshaping the API. The same applies to verification and
public key sources.

**Separate specification logic from I/O and platform glue.** Decomposition,
transcoding, profile handling, and verification need no filesystem, network, or
clock. Keeping them apart from the code that does makes the core portable —
including to WebAssembly — and much easier to exercise against the conformance
fixtures.

Resource limits are worth treating as caller-visible policy rather than
constants. The specification sets no maximum artifact size, so any bound is an
implementation decision; expressing it as a parameter with a documented default
lets callers raise, lower, or disable it.

## Leave room in the crypto layer

Requirements here vary widely between projects: a hardware token, a KMS, a
validated module, a pure-software library picked for footprint, or a backend
chosen for throughput. Treating the cryptographic backend as something a caller
supplies, rather than something the library fixes, keeps those choices open
without forcing a decision up front.

Offering more than one level of integration is one way to do that. Those levels
form a gradient of risk acceptance: the more control a caller takes over the
cryptography, the less the implementation can promise about conformance, and
the more of that guarantee the caller assumes.

- A backend the implementation has qualified against its own suite. The
  implementation can stand behind conformance here.
- A caller-supplied backend the implementation has not qualified, accepted
  deliberately by a caller who takes on the conformance risk along with it.
- A direct implementation of the signing and verification interfaces, where
  establishing conformance becomes entirely the caller's job.

Make that boundary visible in the API and its documentation, so a caller
choosing a lower level knows what it costs them. An unqualified backend that
looks the same as a qualified one moves risk onto someone who never agreed to
take it.

If backends are qualified, binding a key handle to a single algorithm and public
key — and interleaving cross-key checks in the qualification suite — catches
handle-reuse and caching mistakes that otherwise surface as intermittent
verification failures.

Whichever backend is used, check it against the algorithm profiles —
[Ed25519](../algorithms/01-ED25519_PUREEDDSA_RAW_RS64_CANONICAL.md) and
[ECDSA P-256](../algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md) — rather than
assuming the library agrees. Signature encoding, which S-values are accepted,
public key encoding, and whether an API hashes the message or expects a prehash
are all places where common libraries default to something the profiles do not
permit. The conformance fixtures catch these, but the failures are much easier
to read when the assumptions have been checked first.

Keep error values content-free. Report categories, and byte counts where they
are useful, but do not carry payload bytes, key material, or malformed input
into diagnostics, logs, or error strings.

## Ship a runnable example

Include a lightweight signing and verification example in the implementation,
with two copy-and-paste walkthroughs in its `README.md`. Take the offline flow
of [`yaml-sigil-rs/examples/github-keys`](https://github.com/NVIDIA/yaml-sigil-rs/tree/main/examples/github-keys)
as a behavioral reference. Leave out GitHub identity discovery, login, and key
registration. Select keys locally and use the implementation's public APIs.
For key selection and trust semantics, consult the specification's
[Verification API](../verification-api.md) and
[signature-document guidance](../README.md#the-signature-document).

Use whatever runnable form fits the language ecosystem, including an example
binary, script, notebook, or small application. Ecosystem conventions take
precedence over the command-line shape below. An SSH-agent example can sign
with a selected agent identity, verify against agent public identities, and
verify with an explicit OpenSSH public-key file. Explicit-key verification
needs neither an agent nor identity lookup. Where an SSH agent is not a
natural fit, use an ecosystem-native temporary key source and demonstrate
equivalent signing and public-key verification. Keep verification offline even
when the artifact contains a `keyid` URL.

The following shell blocks illustrate the desired flow, not an executable
provided by this kit. Replace `<build-command>` and `<demo>` with actual
commands in the implementation's README, adapt flags to its interface, list
prerequisites, and test the resulting instructions. For a non-CLI example,
provide equally concrete setup and execution steps. Document expected success
output and how failures appear; a CLI should exit unsuccessfully when
verification does not succeed. Run both walkthroughs before delivery.

### Self-consistency test

Build or prepare the example, create an ephemeral Ed25519 key, sign a small
YAML file, and verify it through both local key-selection paths. This Bash
pattern requires OpenSSH tools and uses a temporary directory and an isolated
agent. The subshell cleans up its agent and files on exit, so it can be rerun
without replacing existing keys or changing identities in the user's agent.

```shell
<build-command>
(
  set -eu
  demo_dir="$(mktemp -d)"
  trap 'rm -rf "$demo_dir"' EXIT
  demo_agent_env="$(ssh-agent -s)"
  eval "$demo_agent_env"
  trap 'ssh-agent -k >/dev/null || true; rm -rf "$demo_dir"' EXIT
  ssh-keygen -t ed25519 -f "$demo_dir/key" -N ''
  ssh-add "$demo_dir/key"
  demo_fingerprint="$(ssh-keygen -E sha256 -lf "$demo_dir/key.pub" | awk '{print $2}')"

  printf 'service: demo\nport: 8080\n' > "$demo_dir/unsigned.yaml"
  <demo> sign \
    --input "$demo_dir/unsigned.yaml" \
    --output "$demo_dir/signed.yaml" \
    --key-fingerprint "$demo_fingerprint"
  <demo> verify --input "$demo_dir/signed.yaml"
  <demo> verify \
    --input "$demo_dir/signed.yaml" \
    --public-key "$demo_dir/key.pub"
)
```

Both verification calls should succeed. Include a negative check that changes
the signed payload without signing again and confirms verification failure.
Passing this walkthrough checks the implementation against itself; the next
walkthrough checks an artifact produced by another implementation.

### External validation

Use the published
[`ddurst-nvidia.pub-key`](https://github.com/NVIDIA/yaml-sigil-rs/blob/main/examples/github-keys/fixtures/ddurst-nvidia.pub-key)
and
[`signed.yaml`](https://github.com/NVIDIA/yaml-sigil-rs/blob/main/examples/github-keys/fixtures/signed.yaml)
fixtures. Download both from the same exact `yaml-sigil-rs` commit recorded
under [Track your inputs](#track-your-inputs). The example below pins a commit
containing the pair. Record the chosen commit in the implementation's README
and test metadata, and preserve the downloaded bytes.

Make this walkthrough independently runnable, including its build step.
It needs `curl` for the downloads; verification afterward is local and uses
only the supplied public-key file. The account name identifies the fixture's
source, not an identity the example needs to discover or authenticate.

```shell
<build-command>
(
  set -eu
  demo_dir="$(mktemp -d)"
  trap 'rm -rf "$demo_dir"' EXIT
  reference_commit=deb53b7c554b89437c980a729a614dd46e5624e6
  fixture_base="https://raw.githubusercontent.com/NVIDIA/yaml-sigil-rs/$reference_commit/examples/github-keys/fixtures"
  curl --fail --silent --show-error --location \
    "$fixture_base/ddurst-nvidia.pub-key" --output "$demo_dir/reference.pub"
  curl --fail --silent --show-error --location \
    "$fixture_base/signed.yaml" --output "$demo_dir/reference.yaml"
  <demo> verify \
    --input "$demo_dir/reference.yaml" \
    --public-key "$demo_dir/reference.pub"
)
```

Expect successful verification with the explicit key. Consult the fixture's
[documentation](https://github.com/NVIDIA/yaml-sigil-rs/tree/main/examples/github-keys#fixtures)
at the selected commit for provenance and expected behavior. This example
complements the specification's conformance suites; it does not replace them.

## Use the conformance manifest

[`conformance-manifest.json`](./conformance-manifest.json) groups all fixture
assets into stable cases. Paths are relative to the repository root. Each case
names its operation, form and profile context, and typed expectations or
property assertions.

The manifest is an exhaustive discovery index for the specification commit
that contains it. It is not a new universal runner protocol. Follow the linked
suite `README.md` files and text sidecars for invocation inputs, exact
expectations, and provenance. Those sources win if the manifest disagrees.

## Cross-test against the reference implementation

The conformance fixtures are the contract, but they are a fixed set and cannot
cover every input an implementation will meet. Integration tests that run the
same inputs through `yaml-sigil-rs` and through the new implementation, then
compare the results, find disagreements the fixtures do not reach. They are
also a good way to dial in conformance while an implementation is still taking
shape, because a mismatch points at a specific operation instead of a failing
suite.

Either form of the reference works. Drive it natively as a subprocess or a
linked library, or drive its WebAssembly crate from JavaScript or TypeScript.
Compare operation outcomes, verifier states, error categories, and returned
bytes. Generated or randomized inputs earn their keep here, since the shipped
fixtures are fixed and cannot vary themselves.

Where the two disagree, the specification decides which one is wrong. Neither
implementation is authoritative, and a difference is not automatically a bug in
the new one.

### WebAssembly Reference Implementation

`yaml-sigil-rs` offers a WebAssembly crate that can be built from source and
driven from JavaScript or TypeScript. It exposes compose, decompose, sign, and
verify, and reports stable status and error codes rather than throwing, which
makes it a convenient thing to compare against when a fixture outcome is
ambiguous. Use it if it helps; ignore it if it does not.

Build it from source:

```shell
git clone https://github.com/NVIDIA/yaml-sigil-rs.git
cd yaml-sigil-rs
rustup target add wasm32-unknown-unknown
cargo build -p yaml-sigil-wasm --target wasm32-unknown-unknown
cargo build -p yaml-sigil-wasm --target wasm32-unknown-unknown --release
```

The modules land at
`target/wasm32-unknown-unknown/debug/yaml_sigil_wasm.wasm` and
`target/wasm32-unknown-unknown/release/yaml_sigil_wasm.wasm`. The debug module
is far larger and slower; prefer the release module unless stepping through the
boundary itself.

Neither module can be instantiated as it stands. Both carry unresolved
[`wasm-bindgen`](https://wasm-bindgen.github.io/wasm-bindgen) imports, so run
`wasm-bindgen` over the chosen artifact to emit the JavaScript bindings a host
needs to load it:

```shell
cargo install wasm-bindgen-cli
wasm-bindgen --target web --out-dir pkg \
  target/wasm32-unknown-unknown/release/yaml_sigil_wasm.wasm
```

Install the CLI version that matches the `wasm-bindgen` crate version in the
workspace `Cargo.lock`. A mismatch fails at binding time. Pass
`--version <version>` to `cargo install` when the default is not the matching
one.

That writes `yaml_sigil_wasm.js`, the processed `yaml_sigil_wasm_bg.wasm`, and
TypeScript declarations in `yaml_sigil_wasm.d.ts`. Choose the `--target` that
matches the host: `web` for browsers and native ES module loaders, `nodejs` for
CommonJS, `bundler` for webpack or similar, `deno` for Deno.

## Separate compatibility from security assurance

A conforming implementation is not necessarily a secure one. Passing the
conformance fixtures establishes compatibility, nothing more. A security review
is required before production use or any production-readiness claim, scoped to
the target language and ecosystem and resolved under the downstream project's
security policy.
