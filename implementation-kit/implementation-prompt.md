# Reusable Language Implementation Prompt

Substitute the target's details for every angle-bracketed placeholder as you
follow this prompt. This file does not need to be edited.

---

Implement `YamlSigil.v1alpha1` in `<TARGET_LANGUAGE>` for
`<TARGET_ECOSYSTEM>`. Produce source, tests, and developer documentation.

Use these repositories:

- Authoritative specification:
  [`yaml-sigil-spec`](https://github.com/NVIDIA/yaml-sigil-spec).
- Rust declaration of the API:
  [`yaml-sigil-traits`](https://github.com/NVIDIA/yaml-sigil-traits).
- Reference implementation:
  [`yaml-sigil-rs`](https://github.com/NVIDIA/yaml-sigil-rs).

Stop if any placeholder remains. State the branch and commit hash used for each
repository, and record them in the implementation's developer documentation and
test metadata. Do not copy repository-local branches, private review context,
credentials, internal URLs, or untracked files.

Clone `yaml-sigil-spec` locally before starting. Its specification documents,
conformance fixtures, and generation templates are all faster to search and
read from disk than over the network. The clone does not need to be tracked in
the implementation's own history; if tracking it is useful, a submodule works
anywhere `git` does. Use better dependency tooling instead wherever
`<TARGET_ECOSYSTEM>` provides it.

Treat the specification as authoritative. The API surface comes from its
`proto/` IDL; `yaml-sigil-traits` declares that same surface in Rust and is a
cross-check, not a second source. Use `yaml-sigil-rs` as a behavioral reference
only; how it is organized carries no requirement. Neither repository overrides
the specification. Report a conflict instead of silently following one.

Express the surface using whatever `<TARGET_LANGUAGE>` uses for interface
contracts — interfaces, protocols, abstract base classes, concepts, comptime
interfaces, or an explicit vtable struct — rather than reproducing Rust traits
in a language that has none. `yaml-sigil-traits` declares synchronous and
asynchronous forms of the operations as paired declarations; provide whichever
forms `<TARGET_LANGUAGE>` expects rather than reproducing the pairing.

Generate protobuf message and enum types from the specification's `proto/`
tree with whatever protobuf tooling suits `<TARGET_ECOSYSTEM>`.
`implementation-kit/` carries message-only `buf.gen.yaml` v2 templates for Go,
Python, and TypeScript; use the matching one, or follow the same shape —
messages and enums only, services excluded — with a generator appropriate to
another target language. Pin the template's remote plugin version and revision,
or substitute an equivalently pinned local generator as documented by the kit.
For Go, copy the template into this implementation's own tree and set
`go_package_prefix` there to its real module path. Add and pin the required
generated-code runtime dependency.
Do not generate or hand-write gRPC, Connect, HTTP, IPC, or other transport
stubs for `SigningService`, `TranscriptionService`, or `VerificationService`.

> [!IMPORTANT]
> The specification requires the pre-verification helpers — `CanPreVerify`,
> `PreVerify`, and `VerifyFromPreVerify` — but they are proposed for removal in
> `v1alpha2`
> ([yaml-sigil-spec#72](https://github.com/NVIDIA/yaml-sigil-spec/issues/72)).
> They are worth implementing where artifacts are large enough that parsing once
> and reusing prepared state is a real saving, and add surface without buying
> anything otherwise. Stop and ask whether they are wanted before implementing
> them, and record the answer. Do not quietly omit them: against the current
> specification that is a deviation, and it must be reported as one.
>
> If they are implemented, `VerifyFromPreVerify` remains an in-process,
> same-verifier-instance operation over opaque pre-verification state, and must
> not be registered over gRPC, Connect, HTTP, IPC, or another public transport.
> Generated `VerifyFromPreVerifyRequest` and `VerifyFromPreVerifyResponse`
> messages are data types only.

Implement the Signing API, Transcription API, Verification API, Artifact
Decomposition, YAML and protobuf forms, transcoding rules, conformance
profiles, base64 profile, `keyid` constraints, algorithm profiles, parser
resource bounds, and verified-payload isolation exactly as defined by the
specification. Generated protobuf types provide serialization only.
They do not implement any YamlSigil semantic or security rule.

Read the specification's
[Known Deficiencies](https://github.com/NVIDIA/yaml-sigil-spec#known-deficiencies)
before designing around a conformance profile. Strict inner-protobuf
conformance is not reachable with stock protobuf tooling, and an implementation
that cannot reject unknown fields and duplicate known singular fields advertises
`Permissive` as the honest answer. Advertise the profile actually implemented
rather than the one intended, and do not spend effort engineering toward
`Strict` or `SignatureStrict` with a stock decoder.

Algorithm names are spelled differently in the YAML and protobuf forms, so map
between the two rather than reusing a generated enum value name as a YAML `alg`.
Check the chosen cryptographic library against the specification's algorithm
profiles rather than assuming its defaults agree, particularly around signature
encoding, accepted S-values, public key encoding, and prehashing.

Keep the cryptographic backend caller-supplied where practical, so a hardware
token, KMS, validated module, or a library chosen for footprint or throughput
can be used without reshaping the API. Where callers can substitute their own
backend, state what that costs them: the further cryptography moves out of the
implementation's control, the less the implementation can promise about
conformance, and the more of that guarantee the caller accepts.

Use the specification's
`implementation-kit/conformance-manifest.json` to discover fixture cases.
Follow every linked suite `README.md` and supporting sidecar for invocation
material, precise expected behavior, and provenance. The normative fixture
documentation and fixture bytes override the non-normative manifest if they
disagree. Do not reinterpret the manifest as a new universal test-runner
protocol.

Build a functional conformance harness appropriate to `<TARGET_LANGUAGE>`.
Exercise every manifest case in each stated form and profile, including
operation outcomes, invocation errors, verifier states, returned-byte rules,
and property assertions. Add language-specific negative tests for parser
depth, constructed-node, alias-expansion, memory, input-size, and time bounds
where the portable fixtures intentionally cannot prescribe a library-specific
counter.

A conforming implementation is not necessarily a secure one. Passing
conformance establishes functional compatibility, nothing more. A security
review scoped to `<TARGET_LANGUAGE>` and `<TARGET_ECOSYSTEM>` is required before
production use or any production-readiness claim. Do not describe conformance
success alone as a security audit, certification, or approval.

Finish by reporting the branch and commit hash used for each input repository,
generated-code tool and runtime versions, test commands, fixture coverage,
known deviations, and security-assessment status. Do not claim completion while
a normative behavior or fixture case remains unimplemented.

---
