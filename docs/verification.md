# Coverage and build repeatability

Diffyn measures source coverage and compares clean builds in the
[verification workflow](../.github/workflows/verification.yml). Reports identify
the tested commit and remain available as workflow artifacts for 30 days.

## Source coverage

Each language has its own measurement and an **80% minimum**. A strong result
in one language cannot compensate for a failure in another.

| Scope | Measurement | Tests |
| --- | --- | --- |
| Rust server and library | LLVM executable regions and lines | Unit tests and the real-process MCP suite |
| Python repository automation | Executable statements | Release, container, version, DCO and coverage-accounting tests |
| JavaScript website and repository automation | V8 source coverage reported by c8 | Browser interactions, release orchestration and documentation validation |

The Rust report includes protocol handlers, transport, registries and every
tool. A second report counts unique production code regions, excluding the
project's `cfg(test)` modules and deduplicating generic instantiations and
unit/process binaries. The accounting has its own regression tests. LLVM
regions are not a literal statement count or a branch-coverage measurement.

Python measurement includes every script in `scripts/`, including unexecuted
files. JavaScript measurement includes website scripts, documentation utilities
and release orchestration; test files and installed dependencies are outside its denominator.
Browser tests retain source filenames when evaluating scripts so their
execution contributes to the correct file.

**Coverage is evidence of execution.** It does not establish that every
behaviour, error condition, supported platform or accessibility need is tested.
Read individual file results and uncovered locations when reviewing changes.

### Run locally

Install the documented Rust toolchain, Node.js 24 or newer, and Python 3.11 or
newer. Install the pinned development tools in an isolated Python environment:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.9.1 --locked
python3 -m venv /tmp/diffyn-verification-env
. /tmp/diffyn-verification-env/bin/activate
python3 -m pip install coverage==7.17.0
npm --prefix scripts/docs ci --ignore-scripts --no-audit --no-fund
bash scripts/check-coverage.sh
```

Reports are written under `target/verification/`.
`rust.json` and `rust.lcov` contain the original LLVM results;
`rust-production.json` contains the production-region calculation;
`python.json` and `javascript/coverage-summary.json` contain the other results.

## Regression coverage audit

The audit below covers the project's complete published history from its first
commit on **26 October 2026** through
[`5e5fbd4`](https://github.com/nathcymru/Diffyn/commit/5e5fbd44b08257ce3a710fbadd3155b0a267070a)
on **27 October 2026**. It includes the source commits incorporated by squash
merges and corrections to documentation, installation and the website as well
as the Rust tools. The public issue tracker contained no issue reports when
checked on 27 October 2026; commit changes and pull-request descriptions provide
the defect record for this period. Test evidence includes the retrospective
metadata regressions added in **2.0.1**; the correction dates remain unchanged.

Each row counts one corrected behaviour. A recorded manual check, an existing
general test or a new feature's tests do not count as a newly added regression
for that correction. Purely decorative refinements, dependency maintenance,
new capabilities and a scanner false positive are outside this defect count.

| Date | Corrected behaviour and change record | Added automated regression |
| --- | --- | --- |
| 26 October | [Sponsor button had no active donation destination](https://github.com/nathcymru/Diffyn/commit/d84152603b049441017fe8458da9fb217a1cc7cb) | Added in 2.0.1: [public metadata tests](../scripts/docs/public-metadata.test.mjs) reject the original empty `custom` list and verify that the shipped Sponsor link reaches the published charity page. |
| 26 October | [GitHub could not render the preferred report citation](https://github.com/nathcymru/Diffyn/pull/2) | Added in 2.0.1: [public metadata tests](../scripts/docs/public-metadata.test.mjs) reject the original report without an institution and enforce the explicit institution required by the GitHub formatter. The test checks this known rendering prerequisite; it does not run GitHub's renderer. |
| 26 October | [Git installation was ambiguous after adding the fuzz package](https://github.com/nathcymru/Diffyn/pull/35) | The [release workflow](../.github/workflows/release.yml) installs the named package from Git and runs the MCP process suite against the installed executable. |
| 26 October | [Words joined when mobile styling hid line breaks](https://github.com/nathcymru/Diffyn/commit/8893798a1eabca91fd3298525e91d29528cfaecb) | None recorded. |
| 26 October | [The landing-page illustration was stretched across the document](https://github.com/nathcymru/Diffyn/pull/37) | None retained; responsive browser inspection was manual. |
| 26 October | [Opening a drawer displaced its control](https://github.com/nathcymru/Diffyn/commit/ab5c878e55c395f3794bc7cc20fe049ed032d976) | None for rendered control position; DOM interaction tests do not measure layout. |
| 26 October | [Nested panels prevented scrolling back into the page](https://github.com/nathcymru/Diffyn/commit/ab5c878e55c395f3794bc7cc20fe049ed032d976) | None for browser scroll chaining. |
| 27 October | [Equivalent bullet markers received inconsistent clarity analysis](https://github.com/nathcymru/Diffyn/pull/38) | `clarity_analyzes_equivalent_bullets_and_surrounding_prose` in [the Rust tests](../src/tools/tests.rs). |
| 27 October | [CRLF text prevented checklist extraction](https://github.com/nathcymru/Diffyn/pull/38) | `checklist_crlf_checkboxes_and_code_are_preserved` and `crlf_sequencing_allows_terminal_line_endings` in [the Rust tests](../src/tools/tests.rs). |
| 27 October | [Completed source steps occupied the active checklist window](https://github.com/nathcymru/Diffyn/pull/38) | `checklist_window_moves_past_completed_source_steps` in [the Rust tests](../src/tools/tests.rs). |

**Result: 6 of 10 recorded corrections have a corresponding added automated
regression (60%).** Four received regression tests with the correction; two
received retrospective tests in 2.0.1. The metadata checks also run in the
documentation validator, so restoring either historical failure blocks CI.
The four website layout and scrolling corrections remain recorded without
automated regressions.

The contribution policy requires a regression test for reproducible fixes;
future audits must include every recorded correction in the review period,
including those without tests. Tests for new release automation are separate
and do not increase this historical numerator. Test counts and overall source
coverage cannot substitute for this ratio.

## Repeatable executable builds

```sh
bash scripts/check-repeatable-build.sh
```

This check builds **the committed `HEAD`**, not uncommitted changes. It exports
the same source into two temporary directories, uses separate Cargo target
directories, retains the lockfile and toolchain, and compares the resulting
executables byte for byte. Source paths are remapped to one stable path and
`SOURCE_DATE_EPOCH` is set to the commit timestamp. Existing compiler flags are
retained. Temporary build trees are removed when the check exits.

The report records the commit, target, compiler identity, source timestamp and
executable SHA-256 digest. CI checks the Linux release target; the script also
supports a local macOS check.

This establishes repeatability within the recorded environment. It does not
claim identical results across different compilers, operating systems or
independent builders. Archive timestamps, container layers and platform
signatures are outside this executable comparison. Use the
[release verification process](releases/README.md) to authenticate downloaded
artifacts and their build provenance.
