# Contributing to Diffyn

**Start with one concrete improvement.** Bug reports, documentation fixes,
accessibility feedback and code changes are welcome.

## Choose a route

- **First contribution:** choose a [good first issue](https://github.com/nathcymru/Diffyn/issues?q=is%3Aissue%20is%3Aopen%20label%3A%22good%20first%20issue%22), read its completion checklist and comment with the part you would like to try.
- **Bug:** open a [bug report](https://github.com/nathcymru/Diffyn/issues/new?template=bug_report.yml).
- **Feature:** submit a [feature request](https://github.com/nathcymru/Diffyn/issues/new?template=feature_request.yml).
- **Question or early idea:** use [Discussions](https://github.com/nathcymru/Diffyn/discussions).
- **Vulnerability:** follow the [private security reporting process](SECURITY.md).

Each beginner issue identifies the files to inspect, a small first step and the checks needed for review. Questions and partial progress are welcome. If platform access blocks a client check, record the limitation and ask for maintainer help; you do not need to buy API credits to contribute packaging or offline checks.

## Protect people's information

- **Use synthetic examples** or remove identifying details before posting.
- **Keep health information private** unless the person has chosen to share it.
- **Remove credentials**, private conversations and other people's data.
- **Do not require a diagnosis** to explain a communication preference or accessibility barrier.

## Prepare a change

1. **Read** the [architecture](../docs/architecture.md) and [communication contract](../docs/communication-contract.md).
2. **Create a branch** for a focused change.
3. **Preserve meaning**, exact values, user preferences and access to the original text.
4. **Add automated tests for every major new feature.** Bug fixes must include a regression test when the failure is reproducible; explain any exception in the pull request.
5. **Run the relevant checks** before opening a pull request.

## Check code changes

Rust follows the [Rust Style Guide](https://doc.rust-lang.org/style-guide/), enforced by `rustfmt` and Clippy with warnings denied. Python uses four-space indentation, explicit imports and the formatting rules in [ruff.toml](../ruff.toml). Ruff checks formatting, import order, undefined names, unused imports and selected syntax and style errors. JavaScript uses two-space indentation, semicolons and single quotes under the [Prettier configuration](../.prettierrc.json). The required source style check runs both tools with pinned versions.

Use descriptive names, `const` for JavaScript values that are not reassigned, and explicit asynchronous error handling. Never interpolate untrusted values into shell commands. Automated style checks complement review of clarity, behavior and trust boundaries.

Major new functionality must have automated tests before acceptance. Include normal behavior, meaningful edge cases and relevant trust boundaries. New tests run in CI with the existing suite; tests that only reproduce the implementation do not replace behavioral coverage. See [verification](../docs/verification.md) for coverage and repeatability checks.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 tests/stdio_protocol.py
python3 -m unittest discover -s tests -p 'test_*.py'
```

Install the source style tools in a local Python environment and the existing documentation workspace:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r scripts/requirements-style.txt
npm --prefix scripts/docs ci --ignore-scripts --no-audit --no-fund
ruff check scripts tests
ruff format --check scripts tests
npm --prefix scripts/docs run style:check
```

Apply formatting with `ruff format scripts tests` and `npm --prefix scripts/docs run style:fix`. Review import fixes from `ruff check --fix scripts tests` before committing.

- **Keep standard output for MCP** and avoid logs containing submitted text.
- **Validate inputs in code** rather than relying only on generated schemas.
- **Test meaningful boundaries**, including Unicode, conditions, quantities and malformed requests.
- **Describe remaining limits** without claiming clinical effectiveness from passing tests.

## Write accessible documentation

Follow the [style guide](../docs/guidance/style-guide.md).

- **Use descriptive headings** to make the page easy to scan.
- **Keep paragraphs short** and give each bullet one clear sentence.
- **Bold key terms** instead of emphasising whole paragraphs.
- **Add diagrams** when they clarify architecture, concepts or data flow.
- **Provide text alternatives** for diagrams and meaningful images.
- **Preserve mature vocabulary** and explain necessary technical terms.

## Update prompts, skills and research

Guidance in `docs/guidance/`, templates in `docs/prompts/` and workflows in `skills/` are embedded at compile time. Rebuild after editing them and run the Rust and documentation checks. Skill metadata must match the directory name and embedded registry; keep each workflow self-contained with realistic examples and failure handling.

- **Use primary sources** for substantive evidence claims.
- **Add source metadata** to the [BibTeX bibliography](../docs/bibliography.bib).
- **Link citation keys** from the relevant guide or research section.
- **Distinguish evidence from design choices** and retain uncertainty that affects use.
- **Write original summaries** rather than copying articles or third-party guides.
- **Include neurodivergent participants** in evaluation without requiring diagnostic disclosure.

## Submit a pull request

Every pull request needs a fresh, higher version and an annotated candidate tag created before the PR. Follow [version reservation](../docs/releases/README.md#prepare-a-version) and use the versioned PR helper. A later push needs a new candidate number for the same reserved version; do not move an existing tag or reuse a version from another PR.

Contributors without permission to create repository tags should share their proposed branch in an issue or Discussion and ask the maintainer to reserve its candidate before opening the PR. The helper supports a separate `--head-repo` for fork branches. Versions are accepted in increasing order; if a higher version merges first, an older outstanding reservation needs a replacement PR with a fresh version.

- **Explain the problem** and the resulting behaviour.
- **List relevant checks** and any limitations.
- **Update affected guides**, examples and citations.
- **Keep discussion respectful** under the [Code of Conduct](CODE_OF_CONDUCT.md).

The [maintenance guide](../docs/maintenance.md) describes automated checks and
dependency policy.

## Sign off your commits

**Read the [Developer Certificate of Origin](DCO) before contributing.**
Each pull-request commit needs a final `Signed-off-by` trailer matching its
author or committer name and email.

```sh
git commit --signoff
```

- **Certify only contributions you have the right to submit** under the DCO.
- **Use your own Git identity**; a configured GitHub noreply email is supported.
- **Keep every commit signed off**, including fixups and merge commits in the pull request.
- **Remember that sign-offs are public** and retained with the contribution history.
- **Sign-off is a declaration**, separate from cryptographic commit signing or independent review.

The check reads commit metadata and runs the checker from the trusted base
branch. It checks up to 250 commits; split larger changes into smaller pull
requests. The [fuzzing guide](../docs/fuzzing.md) describes additional checks for
changes to the Rust tools.
