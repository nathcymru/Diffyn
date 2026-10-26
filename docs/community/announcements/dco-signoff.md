---
title: "Code contributions: add a DCO sign-off"
category: announcements
---

**Code contributions use the Developer Certificate of Origin (DCO).** A sign-off records your certification that the contribution meets its terms.

Read the [contribution guide](https://github.com/nathcymru/Diffyn/blob/main/.github/CONTRIBUTING.md) before signing off.
Only certify work for which you can make that statement.

## Add the sign-off

When creating a commit, use:

```sh
git commit -s
```

Git adds a `Signed-off-by` line using the name and email configured for the commit.
Review those details before publishing, because commit metadata is public.

## What the sign-off means

- **Contribution rights:** the DCO concerns your right to contribute the work under the project's licence.
- **A separate mechanism:** this sign-off is a commit-message trailer, not a cryptographic commit signature.
- **Ordinary discussion:** you do not need to create a signed-off commit to ask a question or report a problem.

If a check reports a missing sign-off, ask for help on the pull request before rewriting shared history.
