# Releases and packages

**Install a tagged version** to keep the software used by your MCP client predictable.

- **Version notes:** [Diffyn v2.0.1](v2.0.1.md) and [Diffyn v2.0.0](v2.0.0.md).
- **Historical release:** [Diffyn v0.1.0](v0.1.0.md).
- **Upgrade:** [Migration to 2.0.0](../migration-v2.md).
- **Downloads:** [GitHub Releases](https://github.com/nathcymru/Diffyn/releases).
- **Build requirements:** Rust 1.99 or newer; Python and Node.js are development tools.

## Distribution channels

| Channel | Contents | Publication |
| --- | --- | --- |
| GitHub Releases | macOS ARM64 and Intel, Linux x86-64 and Windows x86-64 binaries; Cargo source package; SHA-256 manifest | A stable version tag on a validated main commit |
| GitHub Packages | `ghcr.io/nathcymru/diffyn`, a Linux AMD64 container for local MCP clients | After the corresponding immutable GitHub release is verified |
| GitHub Pages | Public documentation at [nathcymru.github.io/Diffyn](https://nathcymru.github.io/Diffyn/) | Main-branch changes through the Pages deployment workflow |

Release archives include the project licence, third-party notices, Rust library licence texts and all six portable skills. Each package has a cryptographically signed GitHub artifact attestation linking its digest to the producing workflow and source commit.

**Cargo registry publishing is disabled.** Source installation uses the repository's version tag. GitHub Packages distributes the container; it does not host a Cargo registry package.

## Prepare a version

Every pull request reserves a new stable version before it is opened. The version must be higher than the base branch, published releases and earlier reservations. A published or previously reserved version cannot be reused for a different pull request:

1. **Choose a stable version** using `MAJOR.MINOR.PATCH`: a major version for incompatible interfaces, a minor version for compatible additions, or a patch version for compatible fixes.
2. **Update the package metadata:** `Cargo.toml`, both `Cargo.lock` files, the exact Diffyn dependency in `fuzz/Cargo.toml`, and `CITATION.cff`.
3. **Add release notes** at `docs/releases/vMAJOR.MINOR.PATCH.md`. Update installation examples in the README and website, affected guides, migration notes and citation dates.
4. **Run the version check:** `python3 scripts/check-release-version.py`. CI also rejects version decreases relative to the base commit.
5. **Reserve a candidate tag** `vMAJOR.MINOR.PATCH-rc.N` at the exact branch commit using the versioned PR helper. The first candidate reserves the version; subsequent pushes to the same PR require the next candidate number.
6. **Assign the release milestone**, label the pull request `release`, and complete review and required checks before merging.

Push the versioned branch, then open its pull request through the helper with an authenticated GitHub CLI session:

```sh
GH_TOKEN="$(gh auth token)" python3 scripts/open-versioned-pr.py \
  --repo nathcymru/Diffyn --branch YOUR_BRANCH --base main \
  --title "Describe the release change" --body-file /path/to/pr-description.md
```

The helper creates the annotated candidate tag before creating the PR. Use `--candidate-only` to reserve the candidate before opening through another client, or to add a new candidate after pushing a revision to the same PR. Existing candidate tags are retained.

The required **Pull request version** check validates the reservation timestamp, branch ownership, version ordering and exact current commit. Repository tag rules prohibit updates and deletion of `v*` tags. A PR opened before its initial reservation must be closed and replaced with a newly reserved, higher version; the same rule applies to Dependabot proposals. GitHub's web interface can still create an invalid PR, but required checks prevent it from merging.

After a required workflow succeeds on `main`, **Release the main version** checks whether the version already has a tag. An existing ancestor stable tag means that version has already entered the release process. A new version waits for the main-branch build, security, documentation, coverage and repeatability checks, then dispatches **Release packages** with the exact main commit and version. Completion of any gated workflow triggers another evaluation, so a slow verification job cannot leave a version waiting indefinitely after the earlier checks finish.

The release workflow rechecks the commit and creates an annotated `vMAJOR.MINOR.PATCH` tag. It builds and tests all four platforms, tests the container, signs package provenance, verifies the complete draft asset set and publishes the release. Tag creation and publication occur in the same workflow: a tag created with `GITHUB_TOKEN` does not start another workflow automatically.

**Release tags are immutable.** Never move an existing tag or replace a published artifact. Candidate tags are also immutable and do not publish packages. After merge, the stable tag identifies the final main-branch commit, which may differ from a candidate commit after a squash or rebase. Correct a released defect with a new version. Git tags are annotated but are not currently cryptographically signed; the downloadable artifacts and container have signed provenance.

## Stable and latest

- **Pinned version:** use the explicit release tag, container version or image digest for repeatable installation.
- **Latest release:** GitHub's latest stable release identifies the current supported stable distribution. Prereleases are outside the automated publication path.
- **Container `latest`:** advances only after the versioned image passes registry-backed MCP tests and its source release is still GitHub's latest stable release. Backfilling an older version must not move this alias backwards.
- **Documentation:** Pages follows `main`; it is a separate deployment from downloadable software. A merged PR can show “No deployments” even when the resulting main commit has deployed successfully.

## Recover a failed publication

Inspect the failed job before retrying. Build failures, unavailable required checks and an incomplete asset set block publication. Do not create another version merely to work around an infrastructure failure before any release has been published.

- **Main checks finished late:** rerun **Release the main version** on `main`.
- **Tag exists but publication failed:** rerun the failed **Release packages** run after correcting the cause. An existing incomplete draft needs inspection before a fresh publication attempt; published immutable assets must remain unchanged.
- **An existing release lacks a container:** dispatch **Release packages** with only `published_tag`, such as `v2.0.0`. This path requires an immutable published stable release from `main`, matching checksums and valid source provenance. It builds the container using the dispatched workflow's packaging recipe; its attestation records that workflow revision separately from the binary's release revision.
- **First container publication:** make the package public in its GitHub package settings, then verify an anonymous pull. Repository visibility does not automatically make a new package public.

Close the milestone only after the release, downloads, provenance, public package and relevant deployment have been verified. A skipped publishing job on a pull request is expected; a failed or missing publication for a new release is not completion.

## Verify a download

Download the package and `SHA256SUMS` from the same release. Compare SHA-256 using `sha256sum FILE` on Linux, `shasum -a 256 FILE` on macOS, or `Get-FileHash FILE -Algorithm SHA256` in Windows PowerShell.

A matching digest confirms file integrity. Verify the signed provenance separately with GitHub CLI:

```sh
gh attestation verify FILE --repo nathcymru/Diffyn \
  --signer-workflow nathcymru/Diffyn/.github/workflows/release.yml \
  --source-digest EXPECTED_RELEASE_COMMIT \
  --deny-self-hosted-runners
```

Obtain `EXPECTED_RELEASE_COMMIT` from the release tag's commit on GitHub, or from a fetched repository using `git rev-parse 'v2.0.0^{commit}'`. For v2.0.0 it is `5e5fbd44b08257ce3a710fbadd3155b0a267070a`.

GitHub CLI obtains the public verification material for the Sigstore signature and validates the workflow's certificate, transparency evidence and artifact digest. No maintainer signing secret is stored with the public downloads. Automated releases can identify `main` as the workflow reference; require the expected source commit rather than assuming every attestation names a tag reference.

For containers, pin the published image digest and verify its attestation using `gh attestation verify oci://ghcr.io/nathcymru/diffyn@sha256:DIGEST --repo nathcymru/Diffyn --signer-workflow nathcymru/Diffyn/.github/workflows/release.yml`. Check the binary source revision and packaging revision in the image metadata; a container backfill can use a newer packaging recipe than its binary release.

These checks do not replace platform-specific code signing or notarization requirements.
