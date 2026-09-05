# Contributing to yaml-sigil-spec

`yaml-sigil-spec` is developed agent-first. Use agents to inspect the
specification, diagrams, schemas, conformance fixtures, and rendered companion
material, then review the result as the responsible author before submitting it.

## The Critical Rule

**You must understand your contribution.** AI-assisted contributions are
welcome, but you must be able to explain what changed, why it changed, and how
it affects the normative specification or companion material. Do not submit
generated text, schemas, diagrams, fixtures, or documentation that you cannot
defend without the agent open.

## AI Usage

`yaml-sigil-spec` is agent-first, not agent-only.

- **Do** use agents to read the specification, check cross-references, inspect
  conformance data, and draft updates.
- **Do** use the skills in `.agents/skills/`; they capture repository-specific
  workflows for non-normative material and spec companion updates.
- **Do** question the agent until you understand the normative impact, edge
  cases, and downstream implementation impact of your change.
- **Do not** submit changes you cannot explain in your own words.
- **Do not** use agents as a substitute for reading the relevant specification
  sections, conformance notes, and maintainer guidance.

## Pull-request CI

Pull-request CI is orchestrated only by workflow and policy loaded from current
protected `main`. A repository writer must review the exact latest pull-request
head and comment `/ok to test <head-sha>`. Only that exact lowercase,
40-character SHA command starts candidate validation; every new head requires
a new review and command.

Changes to workflow policy, protected validation tools, manifests, lockfiles,
toolchain or dependency policy, the conformance rebuilder, or its vendored ACVP
corpus are security-sensitive. Each commit must preserve the original human
author while a current repository writer becomes the verified committer, and
its message must contain exact DCO trailers for both identities. For a fork,
maintainer edits must remain enabled on the original pull request. After
reviewing that adopted history, a writer comments
`/ok to test-and-adopt <head-sha>`. Ordinary changes must not use the adoption
command, and sensitive changes must not use the ordinary command.

Record the authorization comment ID and time. GitHub event delivery may take
up to 20 minutes, so the absence of a run or acknowledgement during that
window is not a reason to repeat the command. After 25 minutes, inspect the
Actions run list and the original comment, and distinguish a queued run from a
missing event before posting at most one replacement command for the still
current head.

An authorization is invalid after any head, base, protected-policy, comment
body or timestamp, repository identity, or writer-permission change. Never
accept a late acknowledgement or job result for an invalidated binding.

The protected parent uses a contents-read token only to fetch and verify the
exact authorized head. Candidate-controlled processes then run as a
purpose-created disposable operating-system identity with read-only source and
tool inputs, a minimal environment, and no repository credential, secret,
OIDC, write permission, cache save, or retained artifact. The runner-command
directory is inaccessible to that identity, no trusted Action or post-step
follows candidate execution, and the job fails unless every candidate process
is quiescent and the identity is removed.

Every human-authored pull-request commit must form a linear history from
current `main`, be GitHub Verified, and contain the exact DCO identity required
for direct or adopted history. The contributor's fork branch remains the
pull-request head; a writer's command authorizes testing only and does not
authorize integration.

Protected checkout verifier regressions also run on GitHub-hosted Linux,
macOS, and Windows workers. The Windows leg uses a real directory junction and
a short-name-shaped path to prove fail-closed handling without retaining
artifacts.

Before final authorization, fetch current upstream `main`, rebase the original
contributor branch with `git rebase --gpg-sign <upstream>/main`, and push the
rewritten branch back to the same fork with `--force-with-lease`. Confirm every
rewritten commit is GitHub Verified and DCO-compliant, then request testing for
the new exact SHA. Do not copy the contribution onto a repository-owned branch
merely to run CI.

The protected conformance job bounds the vendored ACVP snapshot before Cargo
compiles candidate Rust. The rebuilder then uses one anchored no-follow read,
preflights byte, group, case, decoded-field, and replay-work limits without
retaining corpus collections, and deserializes the same byte snapshot.
Oversized or noncanonical input fails before cryptographic replay.

Repository Actions execution protection is an additional platform control,
not the source of `/ok to test` authority. When that policy is in **Evaluate**
mode, its warnings are telemetry only: they neither allow nor block a workflow.
The protected `issue_comment` controller and its exact-SHA reauthorization
remain the operational boundary. A maintainer reviews the completed results
and separately decides whether to integrate the pull request.

#### Signing Off Your Work

* We require that all contributors "sign-off" on their commits. This certifies that the contribution is your original work, or you have rights to submit it under the same license, or a compatible license.

  * Any contribution which contains commits that are not Signed-Off will not be accepted.

* To sign off on a commit you simply use the `--signoff` (or `-s`) option when committing your changes:
  ```bash
  $ git commit -s -m "Add cool feature."
  ```
  This will append the following to your commit message:
  ```
  Signed-off-by: Your Name <your@email.com>
  ```

* Full text of the DCO (https://developercertificate.org/):

  ```
    Developer Certificate of Origin
    Version 1.1

    Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

    Everyone is permitted to copy and distribute verbatim copies of this
    license document, but changing it is not allowed.


    Developer's Certificate of Origin 1.1

    By making a contribution to this project, I certify that:

    (a) The contribution was created in whole or in part by me and I
        have the right to submit it under the open source license
        indicated in the file; or

    (b) The contribution is based upon previous work that, to the best
        of my knowledge, is covered under an appropriate open source
        license and I have the right under that license to submit that
        work with modifications, whether created in whole or in part
        by me, under the same open source license (unless I am
        permitted to submit under a different license), as indicated
        in the file; or

    (c) The contribution was provided directly to me by some other
        person who certified (a), (b) or (c) and I have not modified
        it.

    (d) I understand and agree that this project and the contribution
        are public and that a record of the contribution (including all
        personal information I submit with it, including my sign-off) is
        maintained indefinitely and may be redistributed consistent with
        this project or the open source license(s) involved.
  ```
