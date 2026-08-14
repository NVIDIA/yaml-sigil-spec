---
name: update-non-normative
description: Use when updating or reviewing yaml-sigil-spec non-normative companion material after specification changes, including diagrams, SVG and PNG images, schema-adjacent documentation, conformance indexes, examples, and cross-references.
---

# Update Non-Normative Material

## Purpose

`yaml-sigil-spec` owns normative specifications and companion material for
`YamlSigil.v1alpha1`. This skill keeps non-normative and maintenance-facing
material aligned with the authoritative rules after a specification change.

Use this skill when a normative change may require updates to diagrams,
rendered images, explanatory documentation, schema-adjacent notes, conformance
indexes, examples, cross-references, or maintenance instructions.

Some reviewed files are normative inputs, such as `.proto` files,
`schema/YamlSigilSignature.v1alpha1.schema.json`, algorithm specifications, and
conformance fixtures. Review them because companion material depends on them.
Do not hide a normative contract change inside a non-normative cleanup.

## Invariants

- Read the repository-root `AGENTS.md` and every applicable subdirectory
  `AGENTS.md` before editing files in that scope.
- Treat normative specifications as authoritative. Update non-normative
  material to match the normative source, not the other way around.
- Treat listed paths as a starting point, not a closed list.
- Keep source diagrams and rendered images synchronized when both are tracked.
- Preserve `original-readme.md` and `notes.txt` unless the request explicitly
  asks to update historical or scratchpad material.
- Keep changes scoped to `yaml-sigil-spec`. Coordinate downstream repository
  updates after this repository's change is ready and after it merges.

## Workflow

1. Start from the repository root and inspect local instructions and worktree
   state:

   ```shell
   git status --short
   find . -name AGENTS.md -print
   ```

   Read the root `AGENTS.md` and any subdirectory `AGENTS.md` that governs
   touched files. The `conformance/` tree has its own maintenance contract.

2. Identify the complete source change set. If the branch has a base ref, start
   with that full diff so unlisted files are not missed:

   ```shell
   git diff --stat <base-ref>...HEAD
   git diff --name-only <base-ref>...HEAD
   ```

   If there is no base ref, inspect the worktree and staged changes:

   ```shell
   git diff --stat
   git diff --cached --stat
   git diff --name-only
   git diff --cached --name-only
   ```

3. Review the known non-normative and companion surfaces:

   - `DIAGRAM.md`: API-shape Mermaid diagram, outcome categories, invocation
     errors, stage attribution, and YAML byte-range companion links.
   - `images/api-flow.svg` and `images/api-flow.png`: high-level Signing API,
     Transcription API, artifact-form, and Verification API flow.
   - `images/yaml-artifact-transcription-diagram.svg` and
     `images/yaml-artifact-transcription-diagram.png`: YAML-form byte ranges,
     constrained marker profile, canonical artifact shapes, and glossary terms.
   - `schema/README.md`: schema alignment notes, JSON Schema limitations, and
     recommended editing workflow.
   - `conformance/README.md` and per-subdirectory `conformance/*/README.md`:
     fixture inventory, expected outcomes, upstream-source references, and
     documented compromises.
   - `AGENTS.md` and scoped `AGENTS.md` files: maintenance rules and path maps
     that describe the changed surfaces.
   - Any changed or newly added Markdown, JSON, SVG, PNG, or diagram source
     that documents, illustrates, indexes, or references the changed behavior.

   Review any unlisted changed files that could affect companion material.
   Update this map when files move, new companion artifacts appear, or a change
   reveals a cleaner review path.

4. Map specification changes to companion updates:

   | Specification change | Companion surfaces to check |
   | --- | --- |
   | Signing, Transcription, or Verification API method changes | `DIAGRAM.md`, `images/api-flow.svg`, `images/api-flow.png`, API companion links. |
   | Verifier states, invocation errors, `DecomposeOutcome`, `PreVerifyOutcome`, or stage attribution | `DIAGRAM.md`, `conformance/README.md`, affected conformance subdirectory `README.md` files. |
   | Artifact decomposition ranges, constrained marker rules, or YAML artifact shapes | `images/yaml-artifact-transcription-diagram.svg`, its PNG sibling, `DIAGRAM.md`, `conformance/yaml-decomposition/README.md`. |
   | Canonical YAML carrier string values or YAML ↔ protobuf round-trip rules | `conformance/transcoding/README.md` and its paired fixtures. |
   | `YamlSigilSignature.v1alpha1`, `Algorithm`, `alg`, `keyid`, or `signature` shape changes | `schema/README.md`, `conformance/schema-alignment/README.md`, `conformance/key-id/README.md`, affected algorithm and base64 fixture documentation. |
   | Algorithm rule changes | Affected `alg-*` conformance `README.md` files, upstream-source citations, documented fixture compromises. |
   | Conformance fixture or generator changes | `conformance/README.md`, affected subdirectory `README.md` files, `conformance/AGENTS.md` when maintenance rules change. |
   | Glossary or terminology changes | `README.md` references, `DIAGRAM.md`, SVG `<title>`, `<desc>`, labels, alt text, and conformance documentation. |

5. Update diagrams deliberately:

   - Edit Mermaid source in `DIAGRAM.md` when API shape, states, outcomes,
     errors, or stage attribution changes.
   - Edit SVG source before rendered PNG output.
   - For `images/yaml-artifact-transcription-diagram.svg`, follow the checklist
     in the SVG header and regenerate the PNG sibling with the documented
     command:

     ```shell
     convert -density 120 -background white \
       images/yaml-artifact-transcription-diagram.svg \
       images/yaml-artifact-transcription-diagram.png
     ```

   - For `images/api-flow.svg`, update the SVG and PNG together. Use the
     repository-approved renderer if one is documented. If no renderer is
     documented or available, record the missing PNG regeneration in the change
     summary.

6. Run the complete local validation sequence from the repository root:

   ```shell
   cargo xtask ci
   ```

   This covers Markdown, Protobuf, JSON Schema, and the complete locked Rust
   rebuilder workspace. Run the repository's link sweep separately when one is
   configured.

7. Summarize the sync before review:

   - Name the normative source changes that triggered the non-normative update.
   - List each companion surface updated.
   - List reviewed surfaces left unchanged and why.
   - Call out any generated image that could not be regenerated.
   - Call out any downstream repository follow-up that remains.

8. Coordinate downstream repositories:

   - Before merge, notify maintainers of `yaml-sigil-traits` and
     `yaml-sigil-rs` when public trait vocabulary, DTO shape, schema artifacts,
     protobuf IDL, algorithm membership, verifier states, conformance fixtures,
     or expected outcomes changed.
   - After merge, coordinate the exact merged commit or tag with
     `yaml-sigil-traits` so it can advance its `source-spec` pin when needed.
   - After merge, coordinate the exact merged commit or tag with
     `yaml-sigil-rs` so it can import affected local artifacts and update
     implementation behavior, tests, and documentation when needed.
