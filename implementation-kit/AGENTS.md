# Implementation kit

This directory is a non-normative kit for building a `YamlSigil.v1alpha1`
implementation in a language other than Rust. Two different tasks bring
an agent here, and they need different things.

- Asked to **build an implementation**, follow "Build an
  implementation" below.
- Asked to **change the kit itself**, follow "Maintaining the kit"
  below.

## Build an implementation

Read [`README.md`](./README.md) first, then
[`implementation-prompt.md`](./implementation-prompt.md). The prompt is
the working instruction set. The README explains the reasoning behind it
and the traps that generated code does not solve on its own, so read
both before writing code.

The specification is authoritative and sits in the parent directory of
this one. Start from [`../README.md`](../README.md), which indexes the
API documents, algorithm profiles, hard rules, verifier states, and
known deficiencies, and read the documents it points at for the
surfaces being implemented. This kit is not authoritative, and neither
is `yaml-sigil-rs`. Report a conflict rather than quietly following the
kit.

Work in this order.

1. Take the target language and ecosystem from the user's request and
   substitute them for the prompt's angle-bracketed placeholders as you
   follow it. Ask the user for anything the request did not supply.
2. Generate protobuf message and enum types with the matching
   `buf.gen.*.yaml` template, or an equivalent generator for a language
   the templates do not cover. The templates read `proto/` over Git, so
   no checkout of the specification is required for this step.
3. Express the API surface declared by the specification's `proto/` IDL
   using whatever the target language uses for interface contracts, and
   implement against it.
4. Build a conformance harness from
   [`conformance-manifest.json`](./conformance-manifest.json) and the
   fixture documentation it points at.
5. Cross-test against `yaml-sigil-rs` where that helps.

Two decisions MUST go back to the user rather than being made silently.

- **Pre-verification helpers.** The specification requires
  `CanPreVerify`, `PreVerify`, and `VerifyFromPreVerify`, but they are
  proposed for removal and pay off only for large artifacts. Ask whether
  they are wanted, and record both the answer and the fact that omitting
  them is a deviation.
- **Advertised conformance profile.** Stock protobuf decoders reach only
  `Permissive`. Advertise what the decoder actually does, and confirm
  the target profile before building toward it.

Nothing in this kit needs to be edited to use it. Leave this repository
unchanged, and write generated code and implementation source into the
implementation's own tree or a disposable directory. Finish by reporting
the branch and commit hash used for each input repository, along with
the items the prompt's closing paragraph lists.

## Maintaining the kit

When you edit Markdown here, follow the documentation **style guide** in
the repository-root [`AGENTS.md`](../AGENTS.md), including its GitHub
Flavored Markdown dialect target.

### Do not restate normative rules

The kit MUST NOT paraphrase normative rules. Where a reader needs an
exact rule, link to the authoritative document instead. Paraphrase
drifts out of agreement with the specification as the specification
changes, and the kit's own header tells readers to follow the
authoritative source on conflict — a kit that restates rules sets up
that conflict rather than avoiding it.

This applies with equal force to material that looks like helpful
detail: octet counts, accepted signature encodings, profile rules, and
field-handling requirements all belong in the specification, not here.

### Known deficiencies and future changes

Where the specification records a
[Known Deficiency](../README.md#known-deficiencies), the kit MUST NOT
leave an implementer trying to build something the specification does
not expect to be buildable. Say what is unreachable, say what the
honest alternative is, and link the entry.

Where something is required today but expected to change, state the
current requirement, link the tracking issue, and route the decision to
a human rather than deciding on the reader's behalf. Do not soften a
current requirement into "optional."

### How the kit is used

The kit is used in place rather than vendored. A developer points an
agent at
`https://github.com/NVIDIA/yaml-sigil-spec/tree/main/implementation-kit`
and the agent reads the kit from this repository. Repository-relative
links are therefore correct throughout, and resolve both in a clone and
on GitHub.

The bootstrap prompt appears twice — near the top of
[`README.md`](./README.md) and under "DIY Implementations" in the
repository-root [`README.md`](../README.md). Keep both copies in
agreement, and update them if this directory ever moves.

### Audience

Readers range from someone building a throwaway demo to someone building
a first-class SDK. Do not gatekeep, and do not classify the reader's
approach. Present optional tooling as available rather than as a blessed
or discouraged path, and qualify structural advice by when it actually
applies.

Rust is the only NVIDIA provided implementation at time of writing, but
the kit's structural guidance is offered, not required. Nothing about
how `yaml-sigil-rs` is arranged constrains a conforming implementation,
and the kit should not read as though it does.

### Maintenance triggers

The kit links outward heavily, so a major change elsewhere in the
repository can silently break it. Re-check [`README.md`](./README.md)
and [`implementation-prompt.md`](./implementation-prompt.md) whenever
any of the following change:

- The "Known Deficiencies" list in the repository-root
  [`README.md`](../README.md), or its anchor. The kit directs
  implementers there before they design around a conformance profile,
  and calls out the unreachable-`Strict` entry by name.
- The algorithm profile documents under
  [`../algorithms/`](../algorithms/), including their filenames. The kit
  links both by path.
- The [`../conformance/schema-alignment/`](../conformance/schema-alignment/)
  suite, which the kit cites for the YAML/protobuf algorithm-spelling
  split.
- The `proto/` tree, the Buf plugin pins in the `buf.gen.*.yaml`
  templates, or the generated-code runtime dependencies. See
  "Coordinated Buf upgrades" in the repository-root
  [`AGENTS.md`](../AGENTS.md).
- [`conformance-manifest.json`](./conformance-manifest.json), whenever
  fixtures are added, removed, or renamed.

### Generation templates

The `buf.gen.*.yaml` templates use a `git_repo` input pointing at this
repository's public URL with `subdir: proto`, so a move or rename of
`proto/` breaks generation for downstream users even though in-repo
builds still pass. After changing a template, verify it from a
directory holding no checkout, which is how downstream users run it.
Start the following from the repository root.

```shell
kit="$PWD/implementation-kit"
cd "$(mktemp -d)"
buf generate --template "$kit/buf.gen.go.yaml" --output .
```

The templates exclude `SigningService`, `TranscriptionService`, and
`VerificationService` by fully qualified type name. Generated output
MUST contain no service descriptors, gRPC stubs, or Connect stubs.
