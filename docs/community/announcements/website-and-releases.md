---
title: "Find the website, setup guide and release downloads"
category: announcements
---

**Use the [Diffyn website](https://nathcymru.github.io/Diffyn/) for an introduction and the [repository documentation](https://github.com/nathcymru/Diffyn/tree/main/docs) for implementation details.**

The **0.1.0** interface provides three tools, two prompt templates and four reference resources over local stdio.

## Choose the route you need

- **Packaged downloads:** use [GitHub Releases](https://github.com/nathcymru/Diffyn/releases) and follow the instructions attached to the relevant release.
- **Build from source:** follow the [engineering guide](https://github.com/nathcymru/Diffyn/blob/main/docs/engineering.md).
- **Try a request:** copy a synthetic example from the [request guide](https://github.com/nathcymru/Diffyn/blob/main/docs/examples.md).

Diffyn is not published to crates.io, so `cargo install diffyn` is not the installation route.
A local executable still needs an MCP client that can launch it over stdio.

For setup help, use **Q&A** and include your operating system, client name and a redacted error message if relevant.
