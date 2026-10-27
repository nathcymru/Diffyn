# Contribute with Codespaces

**Use the development container to work on Diffyn without installing its toolchain on your computer.** The same configuration works in GitHub Codespaces and the VS Code Dev Containers extension.

## What is included

| Tool | Version or source |
| --- | --- |
| Rust, Cargo, Clippy and rustfmt | 1.99.0 |
| Node.js and npm | Node.js 24.22.1 and its bundled npm |
| Python and virtual environments | Debian Bookworm Python 3.11 |
| Git and SSH client | Signed Debian packages |

The [Dockerfile](../.devcontainer/Dockerfile) pins the official Rust and Node images by digest. Debian packages come from the 26 October 2026 snapshot. `RUSTUP_TOOLCHAIN` selects Rust 1.99.0 inside the container, including when the repository's default toolchain is `stable`. The image supports Linux x86-64 and ARM64.

The development user is `vscode`, without administrator access. The configuration adds no Docker socket, extra host mounts, elevated privileges, secrets or forwarded ports. Local Dev Containers mounts the project checkout by default. Diffyn uses standard input and output; it does not need a public development server.

## Open the environment

1. **Choose the branch you intend to work on.** For a contribution from a fork, open that fork and branch.
2. **Review the container configuration before opening an unfamiliar branch.** Dockerfiles, editor extensions and project commands can execute code. This configuration has no automatic project setup commands or extension installs.
3. **On GitHub, choose Code → Codespaces → Create codespace.** Check the account that will pay for it and your available allowance first. Start with the smallest available machine; larger machines are optional.
4. **Wait for the container build**, then open its terminal at the repository root.

For local development, open a trusted checkout in VS Code and choose **Dev Containers: Reopen in Container**. You need a working local container engine. GitHub's hosted compute allowance does not apply to this local option.

GitHub controls Codespaces availability, authentication and billing. Review its [billing guidance](https://docs.github.com/en/billing/concepts/product-billing/github-codespaces) before creating a hosted environment. This repository does not configure prebuilds or request additional credentials.

Some local container engines preserve a different owner for the mounted checkout. If Git reports **dubious ownership**, first confirm that the terminal is in your trusted repository root. Then trust that directory inside the container:

```sh
git config --global --add safe.directory "$(pwd)"
```

This setting applies to the current directory only. Do not use a wildcard to trust every repository.

## Set up the checkout

**Run these commands yourself after reviewing the branch.** They install the locked documentation dependencies and the pinned Python style tool. No paid model API or API key is needed.

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r scripts/requirements-style.txt
npm --prefix scripts/docs ci --ignore-scripts --no-audit --no-fund
```

The `.venv` and `node_modules` directories are ignored by Git. Activate `.venv` again in each new terminal before running Ruff. Dependency downloads need internet access; building Rust dependencies can execute their build scripts.

## Run the checks

**Start with the checks for the files you changed.** The [contribution guide](../.github/CONTRIBUTING.md) explains the full review requirements.

For Rust or embedded guidance changes:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 tests/stdio_protocol.py
```

Build the binary and run the protocol tests from the **same checkout and version**. The tests compare the reported version and embedded skills with that checkout.

For Python, JavaScript or documentation changes:

```sh
python3 -m unittest discover -s tests -p 'test_*.py'
ruff check scripts tests
ruff format --check scripts tests
npm --prefix scripts/docs run style:check
npm --prefix scripts/docs run lint
npm --prefix scripts/docs run check
```

Container image tests, release publishing and platform-specific checks run in CI. This contributor environment does not include a Docker daemon or release credentials.

## Keep your identity and work safe

**Check your Git identity before committing.** GitHub may populate it for you:

```sh
git config --get user.name
git config --get user.email
```

If you want email privacy, use the exact noreply address from your [GitHub email settings](https://github.com/settings/emails). Set your own name and chosen email for this repository with `git config user.name` and `git config user.email`.

Read the [Developer Certificate of Origin](../.github/DCO), then use `git commit --signoff` for each contribution. Sign-offs contain your name and email and remain public. They are separate from cryptographic commit signing. Follow the [DCO contribution policy](../.github/CONTRIBUTING.md#sign-off-your-commits) and ask the maintainer to [reserve a version](releases/README.md#prepare-a-version) before opening a pull request.

Commit and push work you want to keep. **Stop a Codespace when you finish** to stop active compute use; retained storage may still count toward your allowance. Delete it when you no longer need it, after saving your work. See [stopping a Codespace](https://docs.github.com/en/codespaces/developing-in-a-codespace/stopping-and-starting-a-codespace) and [deleting a Codespace](https://docs.github.com/en/codespaces/developing-in-a-codespace/deleting-a-codespace).

## Rebuild and maintain the environment

After a container configuration change, use **Codespaces: Rebuild Container** or **Dev Containers: Rebuild Container**. Files under `/workspaces` survive a Codespaces rebuild; other container files may not. Repeat manual setup if the environment or its dependencies were removed. See [GitHub's rebuild guidance](https://docs.github.com/en/codespaces/developing-in-a-codespace/rebuilding-the-container-in-a-codespace).

Maintainers should update the image digests, tool versions and Debian snapshot together, then build and test the new environment. Fixed inputs make dependency selection repeatable; they also require explicit updates to receive security fixes. This development image is not a promise of byte-for-byte reproducible builds or a substitute for CI.

The configuration follows the [Dev Container Dockerfile guide](https://containers.dev/guide/dockerfile), [GitHub Codespaces security guidance](https://docs.github.com/en/codespaces/reference/security-in-github-codespaces) and [Debian snapshot documentation](https://snapshot.debian.org/).
