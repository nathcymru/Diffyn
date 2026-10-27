# Repository settings

**Diffyn accepts public contributions through reviewed pull requests.** This guide records the repository's settings baseline and explains which optional services it uses. The settings were reviewed on **27 October 2026**.

Repository settings are stored on GitHub, separately from source files. Check the live settings when changing this baseline; editing this document does not change enforcement.

## General and community access

| Setting | Configuration and purpose |
| --- | --- |
| Identity | Public `nathcymru/Diffyn`, MIT licence, default branch `main`, with the project website linked from its description |
| Forks and social preview | Forking is allowed; a custom social-preview image is configured |
| Template repository | Off; contributors fork this project rather than generate unrelated copies |
| Release immutability | On; published tags and assets cannot be replaced |
| Issues and pull requests | Enabled for all users; contribution guidance, issue forms and a PR template are provided |
| Discussions | Enabled for questions, ideas and examples |
| Wiki | Enabled, with editing restricted to collaborators; `/docs` remains the maintained documentation source |
| Projects | Enabled; issues, release milestones and the written roadmap remain the current planning records |
| Sponsorships | Enabled; the funding links lead to the named charities, as explained in the donation guide |
| Archive Program | Enabled to preserve the public source |
| Merge methods | Merge, squash and rebase are available; every method remains subject to branch rules |
| Automatic merging | Off; the maintainer makes the final merge decision |
| Automatic branch deletion | On for merged PR branches; immutable version tags are retained |
| Update-branch suggestions | Off; required up-to-date checks still prevent merging an outdated branch |
| Web commit sign-off | Required; the separate DCO check covers commits created outside GitHub's editor |
| Commit comments and linked-issue closure | Enabled for public discussion and completion tracking |
| Git LFS archives | Off; the project does not depend on Git LFS |
| Bulk push limit | No additional limit; main and version-tag protections still apply to every affected reference |
| Visibility and ownership | Public and active; no ownership transfer or archive operation is part of routine maintenance |

There are no additional repository collaborators. Grant new maintainers only the access their role needs, and update [governance](governance.md). Keep interaction restrictions off during ordinary operation so new contributors can participate. Review restrictions and limits are available for a specific abuse incident; they are not a prerequisite for contributing.

Public reviews are welcome. Only approvals that GitHub recognises under the branch rules satisfy the required review. A comment or an automated review is not an independent maintainer approval.

## DCO and branch rules

**A DCO sign-off certifies the right to submit a contribution.** It is not cryptographic commit signing, a contributor licence agreement, or a checkbox that replaces commit-level checks. Read the [sign-off instructions](../.github/CONTRIBUTING.md#sign-off-your-commits) and [DCO text](../.github/DCO).

The required **DCO sign-off** job checks every PR commit with code from the trusted base branch. It accepts a matching author or committer certification under the documented DCO policy. It never runs code supplied by the PR. GitHub's web sign-off setting supplements this check.

| Ruleset | Enforcement |
| --- | --- |
| Protect main | Active, with no bypass actors: PRs, resolved review conversations, 22 required checks, up-to-date branches, CodeQL security findings at medium severity or higher, standard errors, and protection against deletion or force pushes |
| Human review with owner exception | Active: one approval, stale approvals dismissed, latest push approved; only the owner can bypass this approval rule through a PR |
| Automatic Copilot review | Requests review on new PRs and subsequent pushes, excluding drafts; service access and usage limits still apply |
| Immutable version and candidate tags | Active for `v*`, with no bypass actors; updates and deletion are prohibited |

The owner's review exception applies to **any PR the owner merges**, including another contributor's PR. It does not bypass required checks or the CodeQL rule. Review this exception when another maintainer joins.

Temporary feature branches remain editable so contributors can rebase and correct their commits. Do not apply main's approval requirements to every working branch. Before introducing a maintained release branch, define its checks, version ordering and publication path; the current release process publishes validated commits from `main`.

Classic branch and legacy tag protections are unused because the active rulesets provide enforcement. Do not add a second overlapping protection merely to populate an empty settings page. See GitHub's [rule layering](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets).

## Actions

| Setting | Configuration and purpose |
| --- | --- |
| Allowed actions | GitHub-owned actions plus `ossf/scorecard-action` and `step-security/harden-runner`; other verified publishers are not automatically allowed |
| Version pinning | Full commit SHAs required; Dependabot proposes reviewed updates |
| Default token | Read-only contents and packages; individual jobs request only the additional permissions they need |
| PR creation and approval | Actions cannot create or approve PRs through the repository-wide permission setting |
| Fork approval | First-time contributors need workflow approval; fork tests receive no repository secrets |
| Logs and artifacts | Repository retention is 90 days; individual workflows can retain temporary artifacts for less time |
| Cache | Seven-day retention and a 10 GB eviction limit; an eviction limit is not a spending cap |
| Runners | GitHub-hosted runners; no persistent self-hosted runners or private network integration |
| OIDC | Default immutable owner and repository identifiers; no separate external cloud trust is configured |

OpenID Connect (OIDC) provides short-lived workflow identity for signed attestations and supported services. It does not require a stored cloud access key. A future cloud deployment must define its own audience, source and environment restrictions before trusting that identity.

### Event policy

The active **Trusted pull request metadata checks** policy allows `pull_request_target` only for:

- `.github/workflows/dco.yml`
- `.github/workflows/labeling.yml`

These workflows read PR metadata using trusted base-owned code. The DCO job is read-only; the labelling job can update labels. Neither checks out or executes a contributor's branch.

GitHub's default policy is scheduled to block this event in affected public repositories on **2 December 2026**. The explicit two-file policy preserves the required checks while leaving other workflows under the platform default. Read [GitHub's event-policy guidance](https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target) before adding an exception.

**Policy insights are diagnostic records, not another policy.** Check the newest DCO and labelling events after changing a policy. Older “Would block” entries describe the earlier configuration and remain in the history.

The [Harden-Runner guide](harden-runner.md) explains the Linux repeatability job's network monitoring, public telemetry and free-plan limits.

## Deployments and development environments

| Area | Configuration and purpose |
| --- | --- |
| Pages | Publishes `/docs` from `main` at the default GitHub Pages address; HTTPS is enforced |
| `github-pages` environment | Only `main` can deploy; no tag patterns, unused `gh-pages` exception or administrator bypass |
| Deployment reviewers and timer | No extra deployment approval or waiting period; publication follows the already-reviewed main branch |
| Environment secrets and variables | None; the static site needs no credentials |
| Custom domain | None; no external DNS or certificate setup is required |
| Repository webhooks | None; existing Actions and app installations deliver the needed integrations |
| Deploy keys | None; release automation uses short-lived GitHub tokens and attestations |
| Codespaces | The optional [development container](codespaces.md) supplies the contributor toolchain |
| Prebuilds | None; no scheduled prebuild compute or storage is provisioned |

The development container can run locally. Creating a hosted Codespace is a separate user action and is subject to that account's quota and billing settings. Review those settings before creation; a public repository does not make all hosted development time free.

Do not create an unused webhook or runner. A future integration must identify its destination, event scope, authentication, owner and removal procedure. A future deployment environment must have an actual workload and explicit branch restrictions.

## Security and reporting

- **Private vulnerability reporting:** enabled, with no mandatory CWE classification and a daily limit of ten reports. Reporters do not need to know a vulnerability taxonomy before asking for help.
- **Dependency graph, Dependabot alerts, malware alerts and security updates:** enabled. Version updates use the checked-in Dependabot configuration.
- **Automatic dependency submission:** disabled; the present dependency manifests and lockfiles are scanned directly. Reassess if a supported build ecosystem needs generated dependency information.
- **Automatic alert dismissal:** disabled for the development-dependency and malware presets. Review findings individually.
- **Grouped security updates:** the general switch is off; the committed configuration defines the intended grouping and review workflow.
- **CodeQL:** advanced workflow configuration covers Rust, Python, JavaScript and Actions. Check-run failure reporting and main's rule use medium-or-higher security findings and standard errors.
- **Secret scanning and push protection:** enabled for supported patterns. Generic-pattern and additional provider-validity features are not configured; availability depends on GitHub's account and product requirements.
- **Copilot Autofix suggestions:** enabled for supported CodeQL findings. Suggestions require review; automatic agentic fixes and additional AI scans are off.

Actions, Agents, Codespaces and Dependabot have no repository secrets. Actions and Agents have no repository variables. Do not add credentials to make an empty settings page appear complete. Never place private credentials in a public issue, configuration example or audit report.

Use the [security policy](../.github/SECURITY.md) for vulnerability reports and the [maintenance guide](maintenance.md#triage-repository-practice-alerts) for the documented Scorecard limitations.

## Copilot and issue planning

Repository instructions are enabled. Copilot approvals are disabled and do not substitute for independent human review. Automatic review requests depend on the requesting account's access; a configured ruleset alone does not establish that a review ran.

Cloud-agent workflow runs and automation PR workflows require approval. Automation triggers require write access. Event automations are enabled, with their ordinary service scope; no repository-specific automation or agent credential is provisioned by this project. Both cloud-agent and review firewalls use the recommended allowlist without custom entries.

No custom repository MCP server is configured. GitHub's built-in agent tools belong to GitHub's service; they are separate from the local Diffyn MCP process. Review any additional server's data access before enabling it.

Issue-agent suggestions use the **Cautious** setting: high-confidence metadata changes may be applied automatically. Maintainers should review labels and milestones, particularly beginner tasks and release assignments. The repository's own labelling workflow has a narrower, explicit path-based scope.

## Apps and external references

Use native GitHub features and existing checks where they meet the need. A second DCO app, dependency updater or coverage gate is unnecessary when it would duplicate the existing required controls.

The StepSecurity Actions Security app is installed with **only Diffyn selected** and complements the pinned monitoring action. Read its [scope and installation guidance](harden-runner.md) before changing access. Retain the free public-repository plan and inspect the actual permissions form rather than relying on a Marketplace summary.

Other app installations can be managed at the account level and may also serve unrelated repositories. Review their selected repositories and pending permission requests with the account owner. Do not change an account-wide installation to Diffyn-only without checking those dependencies. An app's presence is not evidence that every feature is licensed, configured or running.

Custom autolinks are unset because this project uses GitHub issues, PRs and commits. Their ordinary references already become links; an external tracker is not required.

## Keep the baseline current

1. **Inspect the affected setting** and the workflow or guide that depends on it.
2. **Record the intended scope**, including permissions, costs and contributor impact.
3. **Apply the smallest change** and read back the saved configuration.
4. **Verify the affected path**, such as a real PR check or Pages deployment.
5. **Update this guide** and the relevant contributor instructions through a versioned PR.

The audit covers repository controls and the visible app configuration. It does not certify external providers, personal account recovery, account billing, or another repository's dependencies. Keep credentials and private account details out of the public evidence.
