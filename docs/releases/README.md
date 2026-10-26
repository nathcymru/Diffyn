# Releases and packages

**Install a tagged version** to keep the software used by your MCP client predictable.

- **Release notes:** [Diffyn v0.1.0](v0.1.0.md).
- **Downloads:** [GitHub Releases](https://github.com/nathcymru/Diffyn/releases).
- **Build requirements:** Rust 1.99 or newer; Python and Node.js are only needed for development checks.

## Distribution

GitHub Releases hosts platform binaries, a Cargo source package and a SHA-256 manifest.
Dependency versions are recorded in `Cargo.lock`; Dependabot proposes updates for review.
Each package also has a GitHub artifact attestation linking its digest to the release workflow.
Binary archives include the project licence, third-party dependency notices and Rust library licence texts.

**Cargo registry publishing is disabled.** The supported source installation uses the repository's version tag rather than an unrelated registry package with a similar name.

## Publish a version

1. **Update the version** in `Cargo.toml`, `Cargo.lock` and `CITATION.cff`.
2. **Add release notes** at `docs/releases/vMAJOR.MINOR.PATCH.md`.
3. **Merge a signed-off pull request** after the required checks and package builds pass.
4. **Create an annotated tag** named `vMAJOR.MINOR.PATCH` on that main-branch commit.
5. **Push the tag** to start the release workflow.
6. **Check the published assets** and verify a download against `SHA256SUMS`.

The workflow rejects a tag that differs from the Cargo version, a commit outside `main`, or an incomplete package set.
It uploads a draft, checks all six assets, then publishes an immutable release.
Release tags must remain unchanged; publish a new version to correct a release.

## Verify a download

**Linux:** use `sha256sum FILE` and compare its output with `SHA256SUMS`.

**macOS:** use `shasum -a 256 FILE` and compare its output with `SHA256SUMS`.

**Windows PowerShell:** use `Get-FileHash FILE -Algorithm SHA256` and compare the digest with `SHA256SUMS`.

A matching digest confirms that the download matches the release artifact; it does not replace review of the release's origin or platform signing requirements.

**Build provenance:** with GitHub CLI installed, verify the downloaded package:

```sh
gh attestation verify FILE --repo nathcymru/Diffyn
```

Confirm that the verified source reference is the expected `refs/tags/vMAJOR.MINOR.PATCH` release tag.
