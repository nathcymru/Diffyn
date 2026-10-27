# Diffyn

![Diffyn: clearer structure, lower overwhelm, better communication](docs/assets/banner.png)

[![CI](https://github.com/nathcymru/Diffyn/actions/workflows/ci.yml/badge.svg)](https://github.com/nathcymru/Diffyn/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/github/license/nathcymru/Diffyn)](LICENSE)
[![Rust version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fnathcymru%2FDiffyn%2Fmain%2FCargo.toml&query=%24.package.rust-version&label=Rust&logo=rust)](https://rust-lang.org/)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/15973/badge)](https://www.bestpractices.dev/projects/15973)
[![Protocol: MCP](https://img.shields.io/badge/Protocol-MCP-4203BF)](https://modelcontextprotocol.io/)

**Communication tools that adapt to the person.**

Diffyn is a local Rust server for the **Model Context Protocol (MCP)**.
It provides text tools, prompt templates, portable skills and reference guidance for AI clients
serving neurodivergent people.

The name brings together **Different** and the Welsh **Diffinio**: to define.
The aim is clear communication that respects individual preferences, autonomy
and access to detail.

[Website](https://nathcymru.github.io/Diffyn/) ·
[Downloads](https://github.com/nathcymru/Diffyn/releases) ·
[Get started](docs/engineering.md) ·
[Examples](docs/examples.md) ·
[Architecture](docs/architecture.md) ·
[Research](docs/research.md) ·
[Discussions](https://github.com/nathcymru/Diffyn/discussions)

## What Diffyn provides

**Nine local tools** support reliable operations on supplied information:

- Reformat paragraphs, extract explicit steps and review bounded clarity findings.
- Resolve explicit preferences and compare protected details before and after an edit.
- Check a supplied action plan, build a resume card and explain supported numerical calculations.
- Check requested presentation constraints without assigning a readability or capacity score.

**[Fourteen prompts](docs/prompts.md)** guide explanation, clarification, task support, communication repair, writing and decision-making. **[Six portable skills](docs/skills.md)** combine these into focused workflows. **Ten resources** explain the operating rules, evidence and boundaries.

The tools run locally without a model account, API key or hosted service. Semantic rewriting and reasoning belong to the host model. Original information, provenance and uncertainty stay available for review; no tool certifies comprehension, safety or clinical benefit.

See the [communication contract](docs/communication-contract.md) for exact behaviour and [migration guide](docs/migration-v2.md) when upgrading.

## Designed for different processing needs

- **Literal communication:** make instructions, references and assumptions explicit.
- **Attention and planning:** keep the next action visible and group existing steps.
- **Reading comfort:** use short sections, descriptive headings and optional emphasis.
- **Motor sequencing:** offer ordered instructions with stable names for objects and controls.
- **Combined needs:** apply the person's chosen supports without a fixed neurotype preset.

These options can support people who identify as autistic, ADHD, AuDHD, dyslexic
or dyspraxic, alongside anyone who prefers these formats.
**The person's choices take priority over labels.**

## Get started

**Requirements:** Rust 1.99 or later and an MCP client that supports local stdio.

This checkout documents **2.0.0**. The tagged command below requires that version to appear in [GitHub Releases](https://github.com/nathcymru/Diffyn/releases); before publication, use the [source build guide](docs/engineering.md#build-the-executable).

```sh
cargo install --git https://github.com/nathcymru/Diffyn --tag v2.0.0 --locked diffyn
```

1. **Install** with the command above, or choose a [prebuilt download](https://github.com/nathcymru/Diffyn/releases).
2. **Connect** your client using the [configuration guide](docs/engineering.md#connect-an-mcp-client).
3. **Try** a request from the [examples](docs/examples.md).

Diffyn starts when the client launches the executable.
The current transport is **local stdio**; hosted HTTP integration is not included.

See the [release guide](docs/releases/README.md) for supported platforms and package verification.

## Privacy and security

- **No persistence:** Diffyn does not store submitted text or profiles.
- **No content logging:** standard output carries MCP messages only.
- **No outbound model calls:** formatting and reference access run locally.
- **Client policies still apply:** the upstream AI client controls its own data handling.

Read the [security policy](.github/SECURITY.md) for supported versions and
**private vulnerability reporting**.

## Research and citation

The [research report](docs/research.md) describes the evidence, methodology
and evaluation priorities behind Diffyn.

- **Sources:** [central BibTeX bibliography](docs/bibliography.bib).
- **Cite the project:** [CITATION.cff](CITATION.cff).
- **Behaviour:** [communication contract](docs/communication-contract.md).

## Contribute

- **Propose a change:** follow the [contribution guide](.github/CONTRIBUTING.md).
- **Report a bug:** use the [issue templates](https://github.com/nathcymru/Diffyn/issues/new/choose).
- **Share an idea:** join [GitHub Discussions](https://github.com/nathcymru/Diffyn/discussions).
- **Participate respectfully:** follow the [Code of Conduct](.github/CODE_OF_CONDUCT.md).

## Optional charity donations

Diffyn is developed voluntarily and remains **free to use and fork**.
The [charity donation page](.github/DONATE.md) links directly to independent
neurodiversity charities.

**No affiliation. No obligation.** Donations go to the selected charity.

## Cyflwyniad Cymraeg

**Offer cyfathrebu sy'n addasu i'r unigolyn.**

Gweinydd MCP lleol a chod agored yw Diffyn. Mae'n cynnig offer testun,
prompts a chanllawiau ar gyfer cyfathrebu sy'n parchu dewisiadau'r unigolyn.

- **Gwybodaeth glir:** adrannau byr a chamau gweithredu syml.
- **Dewis personol:** mae dewisiadau'r unigolyn yn dod gyntaf.
- **Parch ac annibyniaeth:** nid oes angen datgelu diagnosis.

Daw'r enw o'r gair Saesneg **Different** a'r gair Cymraeg **Diffinio**.

## License

Original code and project documentation use the [MIT license](LICENSE).
Charity names and logos remain the property of their respective organisations;
see the [artwork credits](docs/assets/charities/README.md).
