---
title: "Find the website, setup guide and release downloads"
category: announcements
---

**Use the [Diffyn website](https://nathcymru.github.io/Diffyn/) for an introduction and the [repository documentation](https://github.com/nathcymru/Diffyn/tree/main/docs) for implementation details.**

The **2.x** interface provides nine tools, fourteen prompt templates, ten guidance resources and six portable skills over local stdio.

## Choose the route you need

- **Packaged downloads:** use [GitHub Releases](https://github.com/nathcymru/Diffyn/releases) and follow the instructions attached to the relevant release.
- **Container:** use the public [GitHub Packages image](https://github.com/nathcymru/Diffyn/pkgs/container/diffyn) with the [Docker configuration guide](https://github.com/nathcymru/Diffyn/blob/main/docs/engineering.md#run-with-docker).
- **Build from source:** follow the [engineering guide](https://github.com/nathcymru/Diffyn/blob/main/docs/engineering.md).
- **Try a request:** copy a synthetic example from the [request guide](https://github.com/nathcymru/Diffyn/blob/main/docs/examples.md).

Diffyn is not published to crates.io, so `cargo install diffyn` is not the installation route.
A local executable still needs an MCP client that can launch it over stdio.

For setup help, use **Q&A** and include your operating system, client name and a redacted error message if relevant.
