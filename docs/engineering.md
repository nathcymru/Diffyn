# Engineering guide

## Build and connect

Diffyn is a local MCP server using Rust 2024, Tokio, and the official `rmcp` SDK.
Install [stable Rust](https://rust-lang.org/tools/install/), then run:

If Rust is already installed but `cargo` is unavailable in a Unix shell, load its
environment first with `. "$HOME/.cargo/env"`.

```sh
git clone https://github.com/nathcymru/Diffyn.git
cd Diffyn
cargo build --release --locked
```

The executable is `target/release/diffyn` (`diffyn.exe` on Windows). Configure an
MCP client that supports local stdio to launch that executable. Use the absolute
path on your machine; this illustrative configuration is not a universal client
configuration format:

```json
{
  "mcpServers": {
    "diffyn": {
      "command": "/absolute/path/to/Diffyn/target/release/diffyn",
      "args": []
    }
  }
}
```

The process waits for MCP messages on standard input. No banner on standard
output is expected. No model account, API key, database, or network listener is
required. Network access is needed to download build dependencies, not to run
the server. An upstream LLM may use a cloud service under its own policies.

Rust 1.99 is the project's declared baseline. `rust-toolchain.toml` selects stable
and `Cargo.lock` fixes dependency versions. `rmcp` is pinned to 3.7.0; the SDK's
own minimum Rust version is lower than this project's tested baseline.

## Capabilities and boundaries

| Capability | Behavior | Boundary |
| --- | --- | --- |
| Core instructions | Preference-led, adult-respectful host guidance | Host decides whether to follow it; never replaces higher-priority instructions |
| `reformat_for_cognitive_load` | Conservative local layout changes | No semantic summary or diagnosis |
| `generate_executive_function_checklist` | Extract explicit supplied steps and identify missing structure | Cannot invent a complete plan from an arbitrary goal |
| `verify_clarity_and_literalism` | Flag a finite set of possible English idioms | Does not prove clarity, detect all sarcasm, or replace uncertain language |
| `neuro_inclusive_persona_wrapper` | Reusable preference-led task prompt | User text is a separate data message; host must maintain instruction boundaries |
| `adaptive_input_interrogator` | Prompt to ask the most useful missing question | Does not request a diagnosis or repeatedly ask known preferences |
| Bundled resources | Core instructions, style guide, profile matrix, evidence and limits | Text resources cannot set fonts, colours, spacing, or accessibility of the host UI |

Tool inputs support an optional `profile` with explicitly supplied labels and
preferences. Labels do not select an assumed reading ability or trigger a hidden
clinical preset. Omission is valid; a diagnosis is never required.

See the schemas returned by `tools/list` for exact fields and the examples in
[examples.md](examples.md). Tool results include preserved source text, review
requirements, and limitations. A client should show the useful result and relevant
uncertainty without dumping internal metadata onto the person.

## Protocol and process lifecycle

The SDK handles JSON-RPC framing and MCP lifecycle. Verification covers the legacy
`2025-11-25` initialize/initialized lifecycle and the `2026-07-28`
`server/discover` lifecycle with required request metadata. The latter uses
`io.modelcontextprotocol/protocolVersion` and
`io.modelcontextprotocol/clientCapabilities` under `params._meta`.

Incoming JSON lines have a 1 MiB limit. Text tools apply a stricter 32 KiB UTF-8
payload limit, and profile limits are checked in code. Schema generation alone is
not validation. Invalid arguments produce errors, never silently truncated output.
The stdio connection is intended for a trusted local client, not a multi-tenant
untrusted public endpoint.

The SDK discards syntactically invalid JSON and oversized lines and accepts later
valid messages. It returns JSON-RPC errors for invalid envelopes. This behavior is
tested as implemented; the project does not claim a `-32700` response for every
invalid input.

EOF, SIGINT, and Unix SIGTERM close the process. Cancellation allows bounded
cleanup. A manually managed Tokio runtime limits shutdown waiting because
portable standard-input reads can block on macOS and Windows. Standard output is
reserved for protocol messages; fixed operational errors use standard error.

## Privacy

Diffyn's application code has no outbound model calls, file reads driven by user
input, analytics, persistent profiles, or conversation store. Resources are
embedded at compile time and addressed through an exact URI allowlist. Submitted
text exists in process memory while it is being handled and is returned only as
part of the requested result. This does not guarantee memory zeroization, prevent
OS crash dumps, or control upstream client retention.

No tracing subscriber is installed. SDK debug events can contain malformed
payloads, so setting `RUST_LOG` does not activate content logging in this binary.
Do not add general request logging without reviewing these boundaries.

## Repository layout

```text
src/main.rs              Runtime and process exit
src/server.rs            MCP metadata, capability registration, dispatch
src/server/transport.rs  Bounded stdio and shutdown
src/tools/               Local transformations and explicit profiles
src/prompts/             Prompt registry and argument validation
src/resources/           Embedded resource registry and URI validation
assets/                  Original instructions and reference text
tests/stdio_protocol.py  Real-process MCP integration tests
docs/research.md         Evidence, source limits, and rejected assumptions
```

## Verification

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 tests/stdio_protocol.py
```

Unit tests check preservation, preferences, validation, prompt boundaries, and
resource routing. Process tests check both protocol lifecycles, discovery and
calls, malformed input recovery, input privacy, EOF, and signal shutdown with
bounded waits. CI runs on Linux and macOS. Windows-specific lifecycle behavior
and actual connections to branded AI applications require separate verification.

Tests do not establish clinical effectiveness or acceptance by neurodivergent
users. Participatory evaluation, multilingual semantic testing, assistive
technology testing, and real host rendering remain necessary before broader
claims. See [research.md](research.md) for the evidence assessment.

## Integration references

- [Official Rust MCP SDK release](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.7.0)
- [MCP versioning](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning)
- [OpenAI's description of MCP capabilities](https://developers.openai.com/plugins/concepts/mcp-server)
- [OpenAI remote MCP integration](https://developers.openai.com/api/docs/guides/tools-connectors-mcp)

This release provides local stdio. Hosted integrations that require Streamable
HTTP need an additional transport and deployment configuration; publishing this
repository does not connect or deploy it to ChatGPT, Claude, Mistral, or Gemini.
