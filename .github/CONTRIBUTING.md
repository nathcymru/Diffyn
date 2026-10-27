# Contributing to Diffyn

**Start with one concrete improvement.** Bug reports, documentation fixes,
accessibility feedback and code changes are welcome.

## Choose a route

- **Bug:** open a [bug report](https://github.com/nathcymru/Diffyn/issues/new?template=bug_report.yml).
- **Feature:** submit a [feature request](https://github.com/nathcymru/Diffyn/issues/new?template=feature_request.yml).
- **Question or early idea:** use [Discussions](https://github.com/nathcymru/Diffyn/discussions).
- **Vulnerability:** follow the [private security reporting process](SECURITY.md).

## Protect people's information

- **Use synthetic examples** or remove identifying details before posting.
- **Keep health information private** unless the person has chosen to share it.
- **Remove credentials**, private conversations and other people's data.
- **Do not require a diagnosis** to explain a communication preference or accessibility barrier.

## Prepare a change

1. **Read** the [architecture](../docs/architecture.md) and [communication contract](../docs/communication-contract.md).
2. **Create a branch** for a focused change.
3. **Preserve meaning**, exact values, user preferences and access to the original text.
4. **Add regression coverage** when changing behaviour or fixing a reproducible failure.
5. **Run the relevant checks** before opening a pull request.

## Check code changes

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 tests/stdio_protocol.py
python3 -m unittest discover -s tests -p 'test_*.py'
```

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
