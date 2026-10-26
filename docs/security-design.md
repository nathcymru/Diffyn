# Security design

**Diffyn processes text inside a local MCP server.** The client starts the
process, sends requests over standard input and receives JSON-RPC responses on
standard output.

This guide describes the current implementation and its limits. Use the
[security policy](../.github/SECURITY.md) to report a vulnerability privately.

## Trust and data flow

- **Trust the host:** the operating system and the MCP client control the process and its permissions.
- **Validate submitted data:** text, profiles, prompt arguments and resource identifiers may be malformed or hostile.
- **Keep processing local:** the server makes no outbound model calls or runtime source fetches.
- **Return results to the caller:** responses intentionally include original text where the tool contract requires it.
- **Review upstream use:** the client decides whether to display, retain or send those results to a model or another service.

The server provides **no remote authentication or multi-user isolation**.
Do not expose its stdio transport as a public service without a separate
access-control and resource-management design.

## Controls at each boundary

- **Protocol messages:** each line is limited to 1 MiB, including its delimiter. Total request volume and concurrent work remain unbounded.
- **Tool inputs:** typed parsing rejects unknown fields; code validates blank text, a 32 KiB text limit and preference ranges. Valid text can still be misleading or harmful.
- **Prompt arguments:** combined context and preferences are limited to 32 KiB and separated from the static template. This is not an injection-proof sandbox; the host must enforce its own instructions.
- **Resources:** exact URI matching selects assets embedded at compile time. Changes to these assets need review because they change guidance.
- **Errors:** fixed operational messages and sanitized tool errors avoid echoing submitted values. Successful results intentionally contain caller data.
- **Shutdown:** EOF ends the session; supported stop signals trigger bounded cleanup. The operating system and client control process termination and crash handling.

**Implementation:** [transport](../src/server/transport.rs),
[MCP handlers](../src/server.rs), [tool validation](../src/tools/mod.rs),
[profiles](../src/tools/profile.rs), [prompt registry](../src/prompts/mod.rs)
and [resource registry](../src/resources/mod.rs).

## Privacy and permissions

- **No application persistence:** the server does not save submitted text or profiles.
- **No input logging:** the executable installs no tracing subscriber and excludes submitted content from operational errors.
- **No shell execution:** tool text is processed as data; the domain tools do not launch commands or load user-selected files.
- **No application credentials:** the server implements no password store, authentication scheme or custom cryptographic protocol.
- **Memory remains sensitive:** input can exist in process memory, operating-system swap or crash dumps; the server does not promise secure erasure.

Run the process with only the host permissions it needs. Apply operating-system
resource limits where appropriate: individual input bounds do not provide a
global CPU, memory or request-rate quota.

## Preserve meaning and user control

**Formatting is not a semantic safety check.** Diffyn uses deterministic layout
rules, explicit checklist extraction and a finite set of clarity patterns.

- **Retain the source:** tool responses keep the original text or task for comparison.
- **Keep unresolved work visible:** checklist output retains deferred steps and reports missing structure.
- **Explain uncertainty:** outputs include limitations, review requirements or warnings where applicable.
- **Respect preferences:** profile labels are self-declared; the server does not infer a diagnosis from writing style.
- **Check consequential content:** the host and user must verify conditions, quantities and instructions before acting.

See the [communication contract](communication-contract.md) for tool-specific
behaviour and the [evidence limits](guidance/evidence-and-limits.md).

## Verify changes

**Test the boundary that changed.** Passing checks provide evidence about the
tested behaviour; they do not prove the absence of vulnerabilities.

- **Tool regression tests:** cover input bounds, Unicode, source preservation, conditions and quantities in [Rust tests](../src/tools/tests.rs).
- **Process tests:** exercise malformed messages, oversized lines, privacy canaries, request validation and lifecycle behaviour in [the protocol suite](../tests/stdio_protocol.py).
- **Fuzzing:** randomized text and JSON inputs check tool invariants using the production library; see the [fuzzing guide](fuzzing.md).
- **Static and dependency checks:** CodeQL, Clippy and advisory checks run through the [maintenance workflows](maintenance.md).

Before merging a security-sensitive change, check that it:

1. **Rejects invalid inputs** in executable code, with meaningful negative tests.
2. **Preserves privacy** across errors, logs, fixtures and failure artifacts.
3. **Keeps text as data** and preserves exact values and the original source.
4. **Bounds new work** and explains any remaining resource limits.
5. **Uses minimal permissions** and reviews added dependencies and external access.
6. **Updates the relevant guidance** and reports any unresolved security issue through the private process.
