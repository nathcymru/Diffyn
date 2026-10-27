# Build and connect Diffyn

**Diffyn is a local stdio MCP server** built with Rust, Tokio and the official Rust MCP SDK.

No model account, API key, database or network listener is required to run it.

## Build the executable

Install [stable Rust](https://rust-lang.org/tools/install/), then run:

```sh
git clone https://github.com/nathcymru/Diffyn.git
cd Diffyn
cargo build --release --locked
```

**Output:** The executable is `target/release/diffyn`, or `target/release/diffyn.exe` on Windows.

If Rust is installed but `cargo` is unavailable in a Unix shell, load its environment first:

```sh
. "$HOME/.cargo/env"
```

- **Rust baseline:** The package declares Rust 1.99 and edition 2024.
- **Toolchain:** `rust-toolchain.toml` selects stable Rust with formatting and linting tools.
- **Dependencies:** `Cargo.lock` fixes resolved versions, while the manifest pins `rmcp` to 3.7.0.
- **Network use:** Building may download dependencies, while normal server operation makes no outbound calls.

SDK reference: [mcp-rust-sdk-3-7-0](bibliography.bib).

## Connect an MCP client

Configure a client that supports **local stdio** to launch the executable using its absolute path.

This example illustrates a common configuration shape; use the syntax required by your chosen client.

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

- **Expected startup:** The process waits for MCP input without printing a welcome banner.
- **Standard output:** Only protocol messages belong on stdout.
- **Cloud hosts:** An upstream AI application may still use cloud services and retain messages under its own policies.
- **Hosted connections:** Integrations that require Streamable HTTP need another transport and deployment configuration.

Publishing the repository does not deploy the server or connect it automatically to an AI application.

## Run with Docker

The GitHub Packages image provides the Linux AMD64 executable for clients that can launch Docker. Native downloads remain available for all four release platforms.

```json
{
  "mcpServers": {
    "diffyn": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i", "--platform=linux/amd64",
        "--network=none", "--read-only", "--cap-drop=ALL",
        "--security-opt=no-new-privileges",
        "ghcr.io/nathcymru/diffyn:2.0.0"
      ]
    }
  }
}
```

Keep stdin open with `-i` and do not allocate a terminal with `-t`: MCP uses process pipes. The image runs as a non-root user, requires no writable volume and exposes no network service. Docker needs network access to pull the image; the running container does not. ARM hosts need Docker's Linux AMD64 emulation support.

Pin a version or published image digest for predictable installations. `latest` follows the latest verified stable release. See [package verification](releases/README.md#verify-a-download) before first use.

Integration context: [openai-mcp-server](bibliography.bib) and [openai-remote-mcp](bibliography.bib).

## Discover the capabilities

The server advertises **tools**, **prompts**, **resources** and the **Skills extension**.

| Request | What it returns |
| --- | --- |
| `tools/list` | Nine tools with input schemas, output schemas and annotations. |
| `prompts/list` | Fourteen reusable prompt templates and their argument descriptions. |
| `resources/list` | Ten embedded Markdown resources with exact URIs and byte sizes. |
| `skills/list` and `skills/get` | Six workflows through the advertised Skills extension. |

**Registries have one page:** Continuation cursors are rejected rather than silently ignored.

Use the [communication contract](communication-contract.md) for supported preferences and the [examples](examples.md) for requests and expected results.

## Protocol behaviour

The SDK handles MCP lifecycle and JSON-RPC framing, with process tests covering these two paths:

- **`2025-11-25`:** The client uses the `initialize` and initialized-notification lifecycle.
- **`2026-07-28`:** The client uses `server/discover` with the required metadata on subsequent requests.

The latter carries `io.modelcontextprotocol/protocolVersion` and `io.modelcontextprotocol/clientCapabilities` under `params._meta`.

Protocol reference: [mcp-versioning-2026-07-28](bibliography.bib).

**Input bounds** protect availability without estimating a person's cognitive capacity:

- **Transport:** One incoming JSON line may contain at most 1 MiB, including its newline.
- **Tools:** Source text may contain at most 32,768 UTF-8 bytes and must not be blank.
- **Prompts:** Context and preference text share a combined 32,768-byte limit.
- **Resources:** A supplied URI may contain at most 32,768 bytes and must exactly match the registry.
- **Profiles:** Runtime checks enforce preference ranges and the five-label compatibility limit.
- **New tools:** Field, list and aggregate limits are documented in the [communication contract](communication-contract.md); they reject invalid input without silent truncation.

**Schemas and runtime checks work together:** Unknown fields are rejected, and invalid values are never accepted merely because schema metadata exists.

The SDK discards syntactically invalid JSON and oversized lines, then accepts later valid messages; invalid envelopes receive JSON-RPC errors.

The process tests verify this actual behaviour rather than expecting a `-32700` response for every malformed input.

## Shutdown and error handling

- **Normal closure:** EOF closes the process, including before the first protocol request.
- **Stop signals:** SIGINT and Unix SIGTERM request cancellation before or after connection setup.
- **Cleanup:** Active cancellation allows up to three seconds for service shutdown.
- **Runtime exit:** A 250-millisecond limit prevents portable blocking stdin reads from holding shutdown indefinitely.
- **Private errors:** Operational diagnostics use fixed messages, and tool errors do not echo input values.

The stdio connection is intended for a **trusted local client**, with no public multi-tenant endpoint provided.

## Privacy boundary

**Application processing stays local:** Diffyn has no outbound model calls, input-driven file reads, analytics, saved profiles or conversation store.

- **Embedded content:** Prompts and resources are compiled into the executable rather than loaded from caller-controlled paths.
- **Exact routing:** Resource URIs do not become filesystem paths or network requests.
- **No content logs:** The binary installs no tracing subscriber, so `RUST_LOG` does not activate SDK payload logging.
- **Process memory:** Submitted content remains in memory while being handled and is returned in the requested result.

These boundaries do not guarantee memory zeroization, prevent operating-system crash dumps or govern the upstream client's retention.

## Repository structure

```text
src/main.rs              Runtime and process exit
src/server.rs            MCP metadata and dispatch
src/server/transport.rs  Bounded stdio and shutdown
src/tools/               Local operations and explicit preferences
src/prompts/             Prompt registry and argument validation
src/resources/           Embedded resources and URI validation
src/skills.rs            Embedded skill manifest and extension dispatch
skills/                  Six portable SKILL.md workflows
docs/guidance/           Runtime instructions and reference text
docs/prompts/            Runtime prompt templates
docs/assets/             Images and charity artwork
tests/stdio_protocol.py  Real-process MCP tests
.github/                 Community files, funding and CI
```

See [prompt discovery](prompts.md), [skill installation](skills.md) and [v2 migration](migration-v2.md).

**Rebuild after changing bundled Markdown:** The executable contains the text embedded at compile time.

The [architecture guide](architecture.md) shows the request flow and module relationships.

## Run the checks

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 tests/stdio_protocol.py
python3 -m unittest discover -s tests -p 'test_*.py'
```

- **Unit tests:** Check preservation, Unicode, preferences, validation and prompt/resource boundaries.
- **Process tests:** Exercise real MCP requests, error handling, malformed input, privacy, EOF and signals.
- **Python tests:** Cover archive contents, portable skill packaging, unsafe symlink rejection and DCO validation.
- **CI platforms:** The workflow runs on Linux and macOS.
- **Separate verification:** Windows-specific lifecycle behaviour and connections to individual AI applications need their own checks.

**Software tests do not establish clinical effectiveness or accessibility acceptance.** Host rendering, assistive technologies and participatory evaluation remain separate responsibilities described in the [research basis](research.md).
