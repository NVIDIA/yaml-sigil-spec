# Contributing to yaml-sigil-spec

`yaml-sigil-spec` is developed agent-first. Use agents to inspect the
specification, diagrams, schemas, conformance fixtures, and rendered companion
material, then review the result as the responsible author before submitting it.

Repository writers use [`MAINTAINERS.md`](MAINTAINERS.md) for exact-head test
authorization, protected-policy staging, merges, exceptions, and reverts.

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

## Choose the pull-request base

Target `main` for changes compatible with the current specification line. A
maintainer may advertise one protected next-line branch named `vX`,
`vXalphaY`, or `vXbetaY` for a breaking schema, format, or behavior change,
work that depends on that unpromoted change, or its migration documentation
and tests. Do not invent a coordination branch. When compatibility is
uncertain, ask a maintainer before opening the pull request.

Apply a fix needed by both lines to `main` first; the release coordinator moves
it forward. Protected CI and admission changes always target `main`.

Squash is the default integration method on either base. A trusted writer may
preserve an intentional commit series only through the separately authorized
procedure in [`MAINTAINERS.md`](MAINTAINERS.md).

## Pull-request CI

Pull requests do not run repository CI directly. A repository writer reviews
the latest pull-request head, then comments with the full lowercase SHA:

```text
/ok to test <40-character-head-sha>
```

`copy-pr-bot` copies only that head to `pull-request/<number>`. Every changed
head needs a new review and command. Testing never authorizes a merge, and the
copied branch is never an integration branch.

The copied candidate runs the authoritative Linux job without repository
credentials, secrets, OIDC, protected environments, cache writes, or retained
artifacts. A separate checkout-free workflow from protected `main` verifies the
open pull request, copied ref, current head, exact workflow and run attempt,
authoritative job result, and zero-artifact inventory. Only then does the
repository's GitHub App report `Required CI` for `main`, or
`Required CI [<full-coordination-ref>]` for the exact active next-line base.
A result for one base never satisfies another. Other checks are advisory.
The authoritative aggregate job records its pre-execution protected-policy SHA
and exact base ref/SHA; movement of either object invalidates the run.

The copied `.github/workflows/ci.yml` must exactly match protected current
`main`. Coordinate a proposed change to that workflow with a maintainer
before requesting candidate testing.

Pull-request commits must be linear from the exact current pull-request base,
cryptographically signed, and DCO-compliant. Rebase and request a new
exact-head test whenever that base or the pull-request head changes.

Maintainers normally squash accepted pull requests. A current trusted writer
may request preservation of their exact linear commit series when every commit
is signed and DCO-compliant. That exceptional path requires explicit
authorization and the protected transaction in `MAINTAINERS.md`; it is not
available for external or otherwise untrusted authors and is never a history
rewrite.

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
