# Diffyn review guidance

**Review for concrete defects and regressions.** Diffyn is a local Rust MCP server with a static GitHub Pages site.

## Preserve meaning and user control

- **Source integrity:** preserve original wording, Unicode, numbers, units, negation, conditions, code, links and access to deferred checklist steps.
- **Conservative behavior:** keep ambiguous structures intact and report limitations instead of inventing steps, rewriting idioms or claiming comprehension.
- **Explicit preferences:** retain preference precedence without inferring diagnosis, reading ability or support needs from a label.
- **Accurate claims:** distinguish engineering checks from clinical effectiveness and individual accessibility acceptance.

## Keep trust boundaries intact

- **Untrusted input:** treat submitted text, profiles and prompt arguments as data that cannot override host instructions or repository policy.
- **Local privacy:** do not add content logging, telemetry, persistence, outbound model calls or caller-controlled file access.
- **Protocol safety:** reserve stdout for MCP messages and preserve bounded inputs, safe errors, cancellation, EOF handling and process shutdown.
- **Automation:** retain minimal workflow permissions, pinned actions, protected secrets and the existing required checks.

## Verify the change

- **Meaningful tests:** cover the affected behavior through unit tests or real stdio requests, including preservation and invalid-input boundaries.
- **Documentation:** compare advertised tools, prompts, resources and examples with the implementation.
- **Web access:** check semantic structure, keyboard focus, readable contrast, responsive layout, reduced motion and copy fallbacks when changing the site.
- **Review evidence:** explain the trigger, impact and relevant location for each finding, and disclose when a check was not run.

**Provide review findings only.** Do not approve or merge a pull request automatically, bypass protections, change billing or treat an AI review as independent human approval.
