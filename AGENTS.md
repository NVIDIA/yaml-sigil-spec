# AGENTS.md

## Agent Documentation Standards

Project-local skills exist under `.agents/skills/` and should remain
discoverable by agents working in this repository. Maintain those skills
according to the
[Agent Skills specification](https://agentskills.io/specification), and
maintain this file according to the
[AGENTS.md standard](https://agents.md/). Keep both portable across compatible
agent clients, without assumptions about user-specific paths or session state.

Use `.agents/skills/update-non-normative/SKILL.md` when updating or reviewing
non-normative companion material after specification changes, including
diagrams, schema-adjacent documentation, conformance indexes, examples, and
cross-references.

## Repository scope

This repository is a **specification**: the normative deliverables are
Markdown and protobuf schema, not application source code. The day-to-day
work is reading, writing, and refining those specification documents.

## Document hierarchy

The repository contains **normative** documents at two levels of normativity.
Treat them differently.

### Working specification

- **[README](./README.md)** — the working specification for `YamlSigil.v1alpha1`.
  Contains the glossary, compact artifact examples, format-level rules,
  verifier-state summary, reference index, and the open items still to be
  decided, alongside the normative **Hard Rules Overview** (UTF-8 only, YAML
  1.2 scoped per role, form-appropriate
  structural separation before crypto via the Transcription API — Artifact
  Decomposition for YAML form, `SignedYamlArtifact` outer-envelope decode
  for protobuf form) and a brief enumeration of the Verification API states
  (full definitions live in the companion specification). The README is
  the canonical source for terminology and the format-level rules; for
  the byte-level decomposition algorithm, the Transcription API envelope
  contract, transcoding round-trips, and the verifier API contract,
  consult the hard reference specifications below.

### Hard technical reference specifications

- **[Artifact Decomposition](./artifact-decomposition.md)** — normative
  byte-level YAML implementation reference for the
  [Transcription API](./transcription-api.md)'s `Decompose` operation.
  Covers encoding requirements, the constrained marker profile, the
  decomposition algorithm, failure modes, and the verifier-API state
  mapping. When in doubt about YAML boundary detection, encoding, the
  marker rule, or YAML decomposition outcomes, this document is
  authoritative.

- **[Base64 Requirements](./base64-requirements.md)** — normative
  profile for base64 encoding of the YAML-form `signature` field in
  `YamlSigilSignature.v1alpha1` (URL-safe alphabet, no padding, no
  non-zero trailing bits in the final partial quantum). When in doubt
  about YAML `signature` base64 encode/decode policy, this document is
  authoritative.

- **`proto/yaml_sigil/v1alpha1/yaml_sigil.proto`**
  — normative artifact wire schema for the protobuf form. Defines the
  `SignedYamlArtifact` and `YamlSigilSignature` messages and the
  `Algorithm` enum in package
  `yaml_sigil.v1alpha1`. The strict signature-document
  schema in protobuf form is `v1alpha1.YamlSigilSignature` here. When
  in doubt about the wire shape, message types, field numbers, or
  enum values, the `.proto` file is authoritative. The schema is not
  embedded elsewhere in the repo (this is a deliberate
  single-source-of-truth choice; so [README](./README.md) and
  the API design documents link to this file rather than duplicating its
  contents).

  The `proto/` tree is governed by `proto/buf.yaml` (buf v2 format)
  using the full `STANDARD` lint rule set with no exceptions.
  `buf lint proto` and `buf format proto -d` MUST pass on every
  change.

- **`schema/YamlSigilSignature.v1alpha1.schema.json`** — the working
  enumeration of the YAML-form strict signature-document shape for
  `YamlSigilSignature.v1alpha1`, written against the IETF JSON Schema
  draft. Normative for the schema *shape* (must stay aligned with the
  protobuf `v1alpha1.YamlSigilSignature` above). **JSON Schema is the
  spec's interim validation formalism** for the YAML form; the
  reasoning is in [README](./README.md)'s Implementation Note and in
  [`schema/README.md`](./schema/README.md). Mechanizing the alignment
  between the `.proto` and the YAML-form schema (the natural path is
  `protovalidate`) is tracked under Known Deficiencies.

  **Alignment of the two schemas.** The protobuf `v1alpha1.YamlSigilSignature`
  and the YAML `YamlSigilSignature.v1alpha1` JSON Schema are two
  reifications of the same logical schema. **They MUST be kept
  aligned**: a change to one MUST be accompanied by a matching change
  to the other. A formal alignment process (automated checks,
  cross-validation tooling, conformance tests) cannot exist until both
  representations are written by hand first — that is why both are
  written by hand now. Mechanizing the alignment is future work; the
  artifacts have to exist before any process can validate them against
  each other. Edits SHOULD start with the `.proto` representation and
  replicate into the JSON Schema; see
  [`schema/README.md`](./schema/README.md). For `alg`, the canonical
  names and the protobuf-prefix convention are pinned in
  [README](./README.md)'s "The Signature Document" section.

- **`proto/yaml_sigil/v1alpha1/transcription.proto`**
  — authoritative IDL for the Transcription API surface. Defines
  `TranscriptionService`, request and response messages,
  `TranscriptionForm`, `OuterConformance`, decomposition outcomes, and
  transcriber error enums. When in doubt about the exact transcription
  method shape, field numbers, enum values, or implementor-facing field
  contract, this file is authoritative. The service definition is an API
  contract and code-generation surface; it does not require every
  conforming implementation to expose public gRPC.

- **[Transcription API](./transcription-api.md)** — normative
  specification of the Transcription API design model: the abstract
  Artifact, bytes-only Compose/Decompose contracts, YAML and protobuf
  profiles, outer-envelope conformance behavior, structural failure
  mapping, and explicitly named anti-patterns. Exact method, message,
  enum, field, and service-contract shapes live in `transcription.proto`.
  Inner-`YamlSigilSignature` interpretation and inner conformance live
  in `verification-api.md`, not here.

- **[Transcoding](./transcoding.md)** — normative specification of
  YAML ↔ protobuf round-trip behavior as a two-layer orchestration over
  the Transcription API (envelope) and the Verification API
  (signature-carrier interpretation). Covers the round-trip step tables,
  canonical YAML carrier emission, round-trip properties, verification
  path independence, and the open v1 commitment items that block
  byte-identical interop.

- **`proto/yaml_sigil/v1alpha1/signing.proto`**
  — authoritative IDL for the Signing API surface. Defines
  `SigningService`, request and response messages, signer capability
  fields, `OutputForm`, and signer error enums. When in doubt about
  the exact signing method shape, field numbers, enum values, or
  implementor-facing field contract, this file is authoritative.
  The service definition is an API contract and code-generation
  surface; it does not require every conforming implementation to
  expose public gRPC.

- **`proto/yaml_sigil/v1alpha1/verification.proto`**
  — authoritative IDL for the Verification API surface. Defines
  `VerificationService`, request and response messages, verifier
  states, pre-verification result types, invocation-error enums, and
  metadata fields. When in doubt about the exact verification method
  shape, field numbers, enum values, or implementor-facing field
  contract, this file is authoritative. The service definition is an
  API contract and code-generation surface; it does not require every
  conforming implementation to expose public gRPC. In particular,
  `VerifyFromPreVerify` is an in-process/same-instance API concept,
  not a public RPC deployment surface.

- **[Signing API](./signing-api.md)** — normative specification of the Signing
  API design model: signer inputs, validation phases, signer-side
  errors, output contract, best-effort YAML 1.2 validation at sign
  time, signing flow, signature-document-is-unsigned-input rule,
  empty payload handling, cross-form independence, and explicit
  signer-side anti-patterns. Exact method, message, enum, field, and
  service-contract shapes live in `signing.proto`.

- **[Verification API](./verification-api.md)** — normative specification of the
  Verification API design model: the five distinguishable verifier
  states, invocation-error surface, return contract, reader-side rule,
  stage attribution, cross-form independence, optional pre-verification
  helpers, and explicitly named anti-patterns. Exact method, message,
  enum, field, and service-contract shapes live in `verification.proto`.
  Used for both YAML and protobuf forms via the
  [Transcription API](./transcription-api.md)'s `Decompose` outcomes.

- **[`algorithms/01-ED25519_PUREEDDSA_RAW_RS64_CANONICAL.md`](./algorithms/01-ED25519_PUREEDDSA_RAW_RS64_CANONICAL.md)**
  — normative specification of the
  `ED25519_PUREEDDSA_RAW_RS64_CANONICAL` algorithm (slot 1). Pure EdDSA
  over edwards25519, deterministic signing, raw 64-octet `R ‖ S`,
  strict canonical-encoding verification per
  ["Taming the Many EdDSAs"](https://eprint.iacr.org/2020/1244)
  Algorithm 2 (cofactored equation only). Slot assignment lives in
  [README](./README.md)'s "The Signature Document"; per-algorithm
  conformance rules (key encoding, wire format, signing / verification
  steps, S-component policy, test vectors) live in the algorithm
  document.

- **[`algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md`](./algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md)**
  — normative specification of the `ECDSA_SECP256R1_SHA256_RAW_RS64`
  algorithm (slot 2). ECDSA over secp256r1 (NIST P-256), SHA-256, fixed
  64-octet raw `R ‖ S`, non-deterministic ephemeral nonce per FIPS 186-5
  §6.3. Slot assignment in [README](./README.md); per-algorithm rules
  in the algorithm document.

These hard references — the Markdown references, the protobuf files,
the YAML JSON Schema, and the two algorithm specifications — together
with [README](./README.md) are intended to be tight enough that two
independent implementations agree on every conformance question they
cover.

### History (non-normative)

- **`original-readme.md`** — the historical starting point of the
  repository, preserved for lineage. Not normative.

- **`notes.txt`** — scratchpad. Not normative.

### Diagrams and visual aids (non-normative)

The diagrams below are illustrative companions to the normative
documents. They MUST be kept in sync with the documents they
illustrate, but they are not themselves authoritative; if a diagram
and a normative document disagree, the document wins.

- **[`DIAGRAM.md`](./DIAGRAM.md)** — full end-to-end mermaid view of
  the Signing, Transcription, and Verification API surfaces.
  Companion to the three `*-api.md` design documents and the three
  API `.proto` IDLs. **Maintenance:** any change to a method name,
  state, outcome category, invocation-error category, or stage
  attribution in `signing-api.md`, `transcription-api.md`,
  `verification-api.md`, or [`transcoding.md`](./transcoding.md)
  requires a matching update to `DIAGRAM.md`.

- **[`images/api-flow.svg`](./images/api-flow.svg)** /
  **[`images/api-flow.png`](./images/api-flow.png)** — the
  higher-level Signing → artifact → Verification flow embedded in
  [README](./README.md). **Maintenance:** keep in sync with the
  three `*-api.md` design documents whenever the high-level method
  boundaries shift.

- **[`images/yaml-artifact-transcription-diagram.svg`](./images/yaml-artifact-transcription-diagram.svg)**
  /
  **[`images/yaml-artifact-transcription-diagram.png`](./images/yaml-artifact-transcription-diagram.png)**
  — narrower YAML-form byte-range diagram showing
  `payload_range`, the constrained marker, `signature_carrier_range`
  (markerless), and `signature_document_range` (marker-inclusive)
  across four canonical artifact shapes (signed single document,
  signed multi-document, unsigned, empty-payload signed).
  **Maintenance:** update whenever
  [`artifact-decomposition.md`](./artifact-decomposition.md) renames
  a byte range, the constrained marker profile changes, the
  canonical artifact shapes in [`README.md`](./README.md) or
  [`transcription-api.md`](./transcription-api.md) change, or the
  glossary terms `Signature document` / `Signature carrier` shift.
  The SVG header carries the per-change checklist and the PNG
  rebuild command.

### Conformance fixtures (normative)

The `conformance/` tree carries static fixtures auditors and
implementers use to validate behavior end-to-end. See
[`conformance/AGENTS.md`](./conformance/AGENTS.md) for the per-
subdirectory maintenance contract; in brief, the fixture subdirectories
ship only data — a single Rust generator crate at
[`conformance/rebuild-rs/`](./conformance/rebuild-rs/) reproduces every
fixture bit-identically, with per-spec-surface modules under its `src/`
tree. Upstream-source citations (RFC, FIPS, SEC 1, NIST CAVP / ACVP,
"Taming the Many EdDSAs", etc.) live in the per-subdirectory `README.md`
and in the rustdoc of the matching `src/` module.

**Rule for spec edits:** any change to a normative document that
alters input bytes, expected outcomes, enum membership, or stage
attribution MUST be followed by a conformance pass in the affected
subdirectories. The cross-table:

| Spec change | Affected `conformance/` subdirectory |
| --- | --- |
| `artifact-decomposition.md`, README's artifact-layout examples, constrained marker profile | `yaml-decomposition/` |
| `transcription-api.md`, `transcription.proto`, `yaml_sigil.proto` wire shape, `OuterConformance` enum, protobuf-form inner conformance rules | `protobuf-conformance/` |
| `verification-api.md` "Conformance Profiles" rules as they apply to the YAML signature-document mapping (duplicate keys, unknown keys) | `yaml-signature-conformance/` |
| `Algorithm` enum membership, YAML `alg` string mapping, README's "The Signature Document" alg table | `schema-alignment/` |
| `keyid` constraints (README, `yaml_sigil.proto`, `signing.proto`, `verification.proto`, JSON Schema) | `key-id/` |
| `base64-requirements.md` | `base64/` |
| `algorithms/01-ED25519_PUREEDDSA_RAW_RS64_CANONICAL.md` | `alg-ed25519/` |
| `algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md` | `alg-ecdsa/` |

An auditor of the specification SHOULD be able to validate every
shipped fixture against its upstream source by running the
[`conformance/rebuild-rs`](./conformance/rebuild-rs/) rebuilder and
diffing the output. If a normative change is merged without the
corresponding conformance update, treat it as a defect.

## How to work in this repo

- **For technical questions** about the format (what's signed, what the
  byte layout is, how API calls are shaped, how verification works, how
  transcription works):
  consult the hard technical references first. The README provides
  intent; the hard references provide rules.
- **For design questions** (why was this choice made, what alternatives
  were considered): read the README and the analysis directory.
- **When the README and a hard reference disagree**: this is a defect.
  Flag it. The hard reference is authoritative for conformance, but a
  divergence between the two indicates one or the other needs an
  update.
- **When making changes**: keep cross-references consistent. The
  primary documents and relevant proto comments use the same glossary terms (**Artifact**,
  **Payload stream**, **Signature document**, **Verified payload
  bytes**, **Artifact Decomposition**). Renaming a term means updating
  all affected documents.
- **When editing normative Markdown** (everything under this repo
  except `original-readme.md` and `notes.txt`): follow
  the **Style guide** section below (voice, formatting, word list).
  Use GitHub Flavored Markdown as the source dialect unless a file
  documents a narrower renderer requirement.
  Run `rumdl check` on touched files before landing.

## Style guide

Applies to normative and maintenance Markdown: repository-root spec
documents, `schema/`, `algorithms/`, `conformance/` (including
per-subdirectory `README.md` files), `DIAGRAM.md`, and proto-adjacent
companions. **Exempt:** `original-readme.md` and `notes.txt`. Those
files keep their own voice; do not rewrite them to satisfy this guide
unless the author explicitly asks.

Write like you are explaining something to a colleague. Be direct,
specific, and concise. This repository is a specification, not
application code; the audience is implementers, auditors, and reviewers
who need unambiguous rules, not marketing copy.

Terminology is defined in [README](./README.md)'s glossary and in the
**Naming** section below. Use those forms consistently; do not invent
synonyms for glossary entries.

The Markdown dialect target is GitHub Flavored Markdown (GFM), as
rendered by GitHub repository views. Rely on GitHub's generated
document outline for navigation. Avoid renderer-specific inline
attributes such as `{width=50%}` in new content unless the file
explicitly targets a separate renderer.

### Voice and tone

- Use active voice. "Decompose returns `Unsigned`" not "`Unsigned` is
  returned by Decompose."
- Use second person ("you") when addressing the reader.
- Use present tense. "Verification returns `MalformedAttemptedSigned`"
  not "Verification will return …"
- State facts. Do not hedge with "simply," "just," "easily," or "of
  course."
- RFC 2119 keywords (`MUST`, `SHOULD`, `MAY`, and so on) are normative
  when used in that sense; do not soften or decorate them.

### Things to avoid

These patterns are common in LLM-generated text and erode trust with
technical readers. Remove them during review.

| Pattern | Problem | Fix |
| --- | --- | --- |
| Unnecessary bold | "This is a **critical** step" on routine instructions. | Reserve bold for glossary terms (per README), genuine warnings, and rare emphasis. Prefer `code` for field names, states, and wire identifiers. |
| Em dashes everywhere | "The carrier — which is markerless — crosses the API boundary." | Use commas or split into two sentences. Em dashes are fine sparingly but should not appear multiple times per paragraph. |
| Superlatives | "YamlSigil provides a robust, seamless signing experience." | Say what the format does, not how great it is. |
| Hedge words | "Simply invoke Decompose" or "You can easily verify …" | Drop the adverb. "Invoke Decompose." |
| Emoji in prose | Decorative emoji in specification Markdown. | No emoji in normative prose. |
| Rhetorical questions | "Need byte-identical decomposition? Read on!" | State the purpose directly. |

### Formatting rules

- End every sentence with a period.
- Use `code` formatting for protobuf message and field names, enum
  values, verifier states, YAML mapping keys (`schema`, `alg`,
  `keyid`, `signature`), file paths, and literal octet or wire values.
- Use fenced `yaml` blocks for artifact layout examples, `protobuf` or
  `text` for wire layouts and transcripts, and `shell` for copyable
  tooling commands (`buf`, `cargo`, `jq`, `rumdl`). Do not prefix
  shell commands with `$`:

  ```shell
  buf lint proto && buf format proto -d
  ```

- Use tables for structured comparisons (enum mappings, stage
  attribution, fixture inventories). Keep tables simple (no nested
  formatting).
- Use GitHub Flavored Markdown alert notices for non-normative notes
  and implementation asides when the content benefits from a visible
  notice label. Supported labels are `> [!NOTE]`, `> [!TIP]`,
  `> [!IMPORTANT]`, `> [!WARNING]`, and `> [!CAUTION]`. Use plain
  Markdown blockquotes (`>`) for lower-emphasis asides. Do not use bold
  callouts or documentation-framework components this repository does
  not use.
- Do not number section titles. Write "Artifact Decomposition" not
  "Section 3: Artifact Decomposition."
- Do not use colons in titles. Write "Conformance Profiles" not
  "Profiles: Conformance."
- Use colons only to introduce a list. Do not use colons as
  general-purpose punctuation between clauses.
- When editing in-scope Markdown, run `rumdl check` and fix reported
  layout issues. Do not break emphasis (`*…*`, `**…**`) or
  `[markdown](links)` across line wraps.

### Word list

Use the [README glossary](./README.md#glossary) forms. Common
substitutions and casing:

| Use | Do not use |
| --- | --- |
| **Artifact** (glossary term) | conflating "artifact" with unrelated senses without definition |
| **Payload stream**, **Signature document**, **Signature carrier**, **Verified payload bytes**, **Artifact Decomposition** | ad-hoc synonyms (`body`, `footer`, `sig block`, …) |
| **Transcription**, **Transcoding** (glossary processes) | transcription / transcoding when naming the defined process |
| Signing API, Transcription API, Verification API | signing api, the transcription layer (when naming the spec surface) |
| `YamlSigil.v1alpha1` | Yaml Sigil, YAML Sigil (for the overall specification name) |
| `YamlSigilSignature.v1alpha1` | conflating with the protobuf package alone |
| `yaml_sigil.v1alpha1` (protobuf package) | `nvidia.ddurst.yaml_sigil.v1alpha1`, `YamlSigil.v1alpha1` (as package name) |
| `SignedYamlArtifact`, `YamlSigilSignature` (message types) | signed yaml artifact (prose without `code`) |
| `Verified`, `Unsigned`, `MalformedAttemptedSigned`, `SignedButAlgorithmUnsupported`, `SignedButFailedVerification` | paraphrased verifier outcomes |
| YAML | yaml, Yaml (for the format name) |
| protobuf | Protobuf (unless starting a sentence) |
| JSON Schema | json schema, Json Schema |
| NVIDIA | Nvidia, nvidia |
| `keyid` | key ID, KeyId (for the signature-document field) |
| canonical `alg` strings (README table) | `ALGORITHM_…` prefixed spellings in YAML-form discussion |

Protobuf IDL discussion may use Buf-style prefixed enum names where the
IDL requires it; spec-level YAML discussion uses unprefixed canonical
`alg` strings per README.

## Naming

Two names appear together throughout the documents and should not be
confused:

- **`YamlSigil.v1alpha1`** — the overall specification name.
  Used to identify this format as a whole.
- **`YamlSigilSignature.v1alpha1`** — the schema/message name for
  the contents of the signature document. The `schema:` field inside a
  signature document carries this value. In the protobuf form, the
  corresponding message type is `YamlSigilSignature`.

The overall specification is `YamlSigil.v1alpha1`; the structured
metadata that lives inside the signature document is
`YamlSigilSignature.v1alpha1`.
