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
| Documentation integrity | Markdown, local links, anchors, citations, Mermaid syntax and workflow syntax | Pull requests and main pushes |
| External link availability | Reachability of referenced websites | Weekly or manually |
| OpenSSF Scorecard | Repository security practices | Main pushes and weekly |

**Scope:** these checks detect specific errors and known advisories. They do not certify that a dependency is free of malicious code or establish clinical effectiveness, accessibility conformance, or host-interface quality.

**Availability:** external websites can reject automated requests or fail temporarily. Review those results separately from the checks required to merge code.

## Run checks locally

**Rust:** install the stable toolchain, then use the verification commands in the [engineering guide](engineering.md).

**Documentation:** use Node.js 24 or newer. Dependencies are development-only and do not become part of the Rust executable.

```sh
npm --prefix scripts/docs ci --ignore-scripts --no-audit --no-fund
npm --prefix scripts/docs run lint
npm --prefix scripts/docs run check
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
  docs/bibliography.bib CITATION.cff .github/FUNDING.yml
```

The [workflow](../.github/workflows/links.yml) uses a release archive checked against a fixed SHA-256 digest. It limits request time, retries and concurrency, and excludes private network addresses.

## Review dependency updates

**Dependabot:** weekly version updates cover Cargo, SHA-pinned GitHub Actions, and the documentation package. Security-update PRs use GitHub's separately enabled security-update service.

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
- **Bundled guidance:** changes in `docs/guidance/` or `docs/prompts/` are compiled into the server and need the Rust tests as well as documentation checks.

**Check boundary:** syntax and link checks cannot verify the truth of a citation, the accessibility of a diagram, or the appearance of GitHub's rendered page. Review those outcomes directly.

## Maintain workflow permissions

**Use ordinary pull requests.** PR checks receive no repository secrets, and checkout does not persist credentials.

- **Default token:** keep repository content access read-only.
- **CodeQL:** limit `security-events: write` to the analysis jobs. Rust's `build-mode: none` can still execute build scripts and procedural macros.
- **Scorecard:** keep publishing in its dedicated default-branch job with its required security-event and identity-token permissions.
- **Documentation:** dependency installation disables npm lifecycle scripts; parsers run in a job without write access.
- **Communication:** checks publish job results and summaries; they do not create issues or post PR comments.

Before requiring a new check in branch rules, confirm that it succeeds on GitHub and use its exact check name. Required workflows run on every pull request so path filtering cannot leave them permanently pending.

**Separate signals:** a successful CodeQL analysis means the scan completed; repository code-scanning rules determine which findings prevent merging. Keep Scorecard and third-party link availability outside the required merge checks.
