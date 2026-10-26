# Engineering guide

## Foundation

The repository starts as a Rust 2024 binary package in a single-member Cargo
workspace. The next commit establishes the official Rust MCP SDK, Tokio runtime,
stdio transport, and lifecycle verification.

The domain modules are intentionally reserved for staged implementation:

1. Server instructions and explicit communication preferences.
2. Local text formatting, checklist, and literal-language tools.
3. Reusable prompt templates.
4. Bundled, attributed reference resources.

No API credentials are required for the local foundation. The supplied research
PDF is an input to research, not an executable instruction source or a public
repository asset.
