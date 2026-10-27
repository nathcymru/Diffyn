# Maintaining Diffyn

**Start with the failed check.** Open its log, identify the affected file or dependency, and make the smallest reviewed correction.

Use the [contribution guide](../.github/CONTRIBUTING.md) for ordinary changes and the [security policy](../.github/SECURITY.md) for private vulnerability reports.

## What runs automatically

| Check | Purpose | When |
| --- | --- | --- |
| Rust and MCP | Formatting, Clippy, unit tests and real-process protocol tests | Pull requests and main pushes; Linux and macOS |
| CodeQL | Static analysis of Rust, Python, Actions and JavaScript | Pull requests, main pushes and weekly |
| Dependency review | Newly introduced known vulnerable dependencies | Pull requests |
| Rust dependency policy | RustSec advisories, licences, permitted sources and dependency policy | Pull requests, main pushes and daily |
| Documentation dependency audit | Known advisories affecting the locked documentation tools | Pull requests, main pushes and daily |
| Documentation integrity | Markdown, links, citations, diagrams, workflow syntax and website interactions | Pull requests and main pushes |
| External link availability | Reachability of referenced websites | Weekly or manually |
| OpenSSF Scorecard | Repository security practices | Main pushes and weekly |
| DCO sign-off | Every PR commit carries a matching contributor sign-off | Pull requests |
| Fuzzing | Randomized inputs exercise text preservation and structured tool requests | Pull requests, main pushes and weekly |
| Release packages | Build and test four platform binaries and the Cargo source package | Pull requests, release tags or manually |
| Container and MCP | Non-root container, registry provenance, vulnerability scan and real-process protocol tests | Pull requests and release publication |
| Source coverage | At least 80% measured coverage in each of Rust, Python and JavaScript | Pull requests and main pushes |
| Source style | Python correctness and formatting rules; consistent JavaScript formatting | Pull requests and main pushes |
| Repeatable Linux build | Compare executable bytes from two clean builds | Pull requests and main pushes |
| PR version reservation | Require a higher unused version and a matching candidate tag | Pull requests |
| Release the main version | Dispatch publication for a new stable version after all main checks pass | Successful main CI or manually |

**Scope:** these checks detect specific errors and known advisories. They do not certify that a dependency is free of malicious code or establish clinical effectiveness, accessibility conformance, or host-interface quality.

**Availability:** external websites can reject automated requests or fail temporarily. Review those results separately from the checks required to merge code.

## Run checks locally

**Rust:** install the stable toolchain, then use the verification commands in the [engineering guide](engineering.md).

**Python:** exercise release archives and DCO validation without creating a release:

```sh
python3 -m unittest discover -s tests -p 'test_*.py'
```

**Documentation:** use Node.js 24 or newer. Dependencies are development-only and do not become part of the Rust executable.

```sh
npm --prefix scripts/docs ci --ignore-scripts --no-audit --no-fund
npm --prefix scripts/docs run lint
npm --prefix scripts/docs run check
node --test scripts/docs/site.test.mjs
npm --prefix scripts/docs audit --package-lock-only --audit-level=low
```

**Dependency policy:** install the exact audited tool version, then check the committed lockfile.

```sh
cargo +stable install cargo-deny --version 0.20.2 --locked
cargo +stable deny --locked check
```

The advisory database refreshes during this check. An offline run cannot discover newly published advisories.

**External references:** with Lychee 0.24.2 installed, run:

```sh
lychee --config .lychee.toml --no-progress \
  README.md 'docs/**/*.md' '.github/**/*.md' \
  docs/index.html docs/bibliography.bib CITATION.cff .github/FUNDING.yml
```

The [workflow](../.github/workflows/links.yml) uses a release archive checked against a fixed SHA-256 digest. It limits request time, retries and concurrency, and excludes private network addresses.

## Review dependency updates

**Dependabot:** weekly version updates cover the application and fuzzing Cargo workspaces, SHA-pinned GitHub Actions, the documentation package and the pinned container base image. Security-update PRs use GitHub's separately enabled security-update service. Dependabot proposals do not reserve project versions: a maintainer must carry the dependency change into a fresh versioned branch, reserve its candidate, open the replacement PR and close the original proposal. The version gate has no dependency-update exception.

- **Read the change:** inspect release notes, dependency changes and any permissions changes.
- **Keep exact pins:** actions use full commit hashes with version comments; documentation dependencies use a committed npm lockfile.
- **Check tool pins:** Cargo-deny, Lychee and actionlint versions are explicit workflow values; review their upstream releases when updating them.
- **Review exceptions:** do not silence a new advisory or expand the licence allowlist just to obtain a passing check.
- **Retain evidence:** explain any justified, narrowly scoped exception in the change and its configuration.

**Current compatibility override:** the documentation package overrides `micromark-extension-math`'s KaTeX dependency to patched version 0.18.2 for GHSA-238p-pmpm-9mq7. [katex-ghsa-238p-pmpm-9mq7](bibliography.bib)

Remove the override when the upstream dependency selects a patched release and both linting and the audit pass.

**Rust licences:** [deny.toml](../deny.toml) permits the licences needed by the present dependency graph. Duplicate transitive versions produce a reviewable warning; vulnerabilities, yanked versions, disallowed licences and unknown sources fail the check.

## Edit documentation safely

- **Links:** use relative paths for repository files; the offline checker also verifies Markdown headings and explicit HTML anchors.
- **Citations:** keep research keys aligned with [bibliography.bib](bibliography.bib); [CITATION.cff](../CITATION.cff) is validated against the official CFF 1.2.0 schema.
- **Diagrams:** Mermaid fences are parsed for syntax. Preview their layout separately and retain a text alternative.
- **Presentation:** Markdownlint permits the intentional HTML used for linked logos and centered sections. Long source lines are allowed; keep rendered paragraphs readable.
- **Bundled guidance:** changes in `docs/guidance/`, `docs/prompts/` or `skills/` are compiled into the server and need the Rust tests as well as documentation checks.

**Check boundary:** syntax and link checks cannot verify the truth of a citation, the accessibility of a diagram, or the appearance of GitHub's rendered page. Review those outcomes directly.

## Review pull requests

- **Required checks:** build, security, dependency, documentation, package and DCO checks remain mandatory. Coverage, repeatability, container and version checks provide additional release evidence.
- **Human review:** changes require one approval, dismissal of stale reviews and approval of the latest push.
- **Owner exception:** only `nathcymru` can bypass the human-approval rule through a pull request; required checks and the CodeQL gate still apply.
- **Copilot:** automatic review requests cover new pull requests and subsequent pushes, excluding drafts; execution depends on the author's Copilot access and available credits.
- **Review findings:** address actionable findings before merging; an AI review does not establish independent human review.

## Triage repository-practice alerts

Scorecard findings describe repository practices as well as code risks. Review the reported condition, existing controls and available remediation before choosing a disposition. A dismissed alert retains its evidence and does not improve the underlying Scorecard result.

The following findings were reviewed on **27 October 2026** and accepted under the current solo-maintainer model:

| Finding | Current controls and remaining limitation | Reassessment |
| --- | --- | --- |
| [Branch protection, alert 1](https://github.com/nathcymru/Diffyn/security/code-scanning/1) | The main protection ruleset has no bypass actors and requires 22 checks, CodeQL, pull requests and protected history. A separate review rule permits only the owner to bypass approval through a PR. Two independent approvals and CODEOWNERS review are not required. | Review the exception and independent approval requirements when additional maintainers join. |
| [Code review, alert 3](https://github.com/nathcymru/Diffyn/security/code-scanning/3) | The reviewed sample contained 0 of 15 changesets with independent approval. Automated tests and analysis remain mandatory; automated review does not supply independent human judgment. | Reassess as genuine independently reviewed changes enter the sample. |
| [Maintained, alert 4](https://github.com/nathcymru/Diffyn/security/code-scanning/4) | The repository was created on 26 October 2026 at 03:48:32 UTC. Scorecard applies an age condition of more than 90 days. | Reassess after 24 January 2027 at 03:48:32 UTC, alongside actual maintenance activity. |

These alerts use GitHub's **Won't fix** disposition to record an accepted present limitation. Do not classify them as false positives or claim that their conditions have been repaired. Reopen an alert if its accepted scope changes or its controls no longer hold.

Main-branch and weekly Scorecard runs, unmodified SARIF results and public result publication continue. Review new findings individually. The [Scorecard check definitions](https://github.com/ossf/scorecard/blob/main/docs/checks.md) explain the measurements; GitHub documents [alert dismissal and reopening](https://docs.github.com/en/code-security/how-tos/manage-security-alerts/manage-code-scanning-alerts/resolve-alerts).

## Maintain workflow permissions

**Use ordinary pull requests.** PR checks receive no repository secrets, and checkout does not persist credentials.

- **Default token:** keep repository content access read-only.
- **CodeQL:** limit `security-events: write` to the analysis jobs. Rust's `build-mode: none` can still execute build scripts and procedural macros.
- **Scorecard:** keep publishing in its dedicated default-branch job with its required security-event and identity-token permissions.
- **Documentation:** dependency installation disables npm lifecycle scripts; parsers run in a job without write access.
- **Communication:** checks publish job results and summaries; they do not create issues or post PR comments.
- **DCO:** the read-only `pull_request_target` job checks out the trusted base commit and reads commit metadata; it never checks out or runs pull-request code.
- **Fuzzing:** use the bounded runs and local reproduction steps in the [fuzzing guide](fuzzing.md).
- **Packages:** follow the [release guide](releases/README.md); publishing permissions are limited to the relevant tagging and publication jobs. Pull requests cannot publish a stable release.

Before requiring a new check in branch rules, confirm that it succeeds on GitHub and use its exact check name. Required workflows run on every pull request so path filtering cannot leave them permanently pending.

**Separate signals:** a successful CodeQL analysis means the scan completed; repository code-scanning rules determine which findings prevent merging. Keep Scorecard and third-party link availability outside the required merge checks.

## Labels and milestones

Use labels to describe a change and milestones to group a concrete release:

| Label | Use |
| --- | --- |
| `bug` | A reproducible defect |
| `enhancement` | New or improved behavior |
| `documentation` | Public guides, website content or examples |
| `accessibility` | An accessibility barrier or improvement |
| `security` | Public security maintenance; report undisclosed vulnerabilities privately |
| `dependencies` | Dependency or base-image updates |
| `ci` | Build, verification or repository automation |
| `release` | Version metadata, distribution or release publication |
| `integrations` | Native host plugins and MCP client connections |
| `deployment` | User-owned installation, configuration and operations |
| `good first issue` | A bounded newcomer task with prerequisites, source pointers and completion checks |
| `help wanted` | Work open to contributor participation |

The labeling workflow adds area labels from trusted pull-request metadata and changed paths. It does not infer whether a change fixes a bug, constitutes a vulnerability or belongs to a particular release. Dependabot supplies its dependency and area labels explicitly.

Assign each version pull request to its intended release milestone. Keep the milestone open until publication and verification are complete; a merged pull request alone does not close the distribution work. Move unfinished work to a later milestone with an explanation instead of marking it complete.
