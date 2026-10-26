---
title: "Does Diffyn need an API key or a particular AI model?"
category: q-a
---

**The local Diffyn server needs no model account or API key.** It runs deterministic text operations and returns bundled prompts and resources.

This maintainer-written FAQ distinguishes the server from its host.

## What you do need

- **An executable:** build Diffyn or use a suitable binary from [GitHub Releases](https://github.com/nathcymru/Diffyn/releases).
- **A compatible client:** it must be able to launch a local MCP server over stdio.
- **The correct path:** configure the client with the executable's absolute path.

The AI host may have its own subscription, API key or cloud-service requirements.
Its model use and data policies remain separate from Diffyn's local processing.

The current server does not provide a hosted HTTP endpoint, and publishing the repository does not automatically connect it to an AI application.

Use the [connection guide](https://github.com/nathcymru/Diffyn/blob/main/docs/engineering.md#connect-an-mcp-client) for setup.
