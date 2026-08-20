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

Pull-request CI runs from a repository-owned `pull-request/<number>` branch at
the exact latest pull-request head. Do not push to these branches. Signed
commits from trusted organization members are copied automatically after the
pull request is ready for review; draft pull requests do not auto-sync.

If the copy bot requests validation, a maintainer must review the exact latest
head and comment `/ok to test <head-sha>`. Automated App-authored commits and
other untrusted heads require this SHA-bound approval. Every new untrusted head
requires a new review and comment. The copied branch reports checks to the
matching pull-request head and is removed when the pull request closes or
merges.

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
