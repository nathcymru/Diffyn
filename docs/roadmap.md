# Roadmap

This roadmap covers **October 2026 through October 2027**. Priorities may change in response to defects, security reports and participant feedback. It describes intended work, not promised delivery dates.

## Reliability and distribution

Maintain the 2.x MCP interfaces, exact-detail preservation and local processing model. Extend regression cases where real failures expose gaps, keep dependency and runtime checks current, and verify compatibility with supported MCP hosts. Keep releases, source tags, downloadable packages and container aliases consistent through the documented release process.

Complete the maintainer continuity arrangements described in [governance](governance.md). Maintain measured test coverage and repeatable-build evidence, and address remaining assurance gaps before claiming a higher certification level.

## Communication and accessibility evaluation

Evaluate prompts, tools and portable skills with neurodivergent participants using consent-based, task-focused studies. Prioritize user control, preservation of meaning, recovery after interruptions, readable action plans and honest uncertainty. Publish methods and findings without exposing participant data or treating engineering tests as clinical validation.

Improve the website and integration guidance from keyboard, screen-reader, zoom, reduced-motion and mobile feedback. Keep access to original content and unresolved steps visible in host guidance.

## Language and host support

Explore Welsh-language guidance and clearer language-coverage reporting. Separate locale-aware presentation from facts, quantities and immutable source text. Add language support only with suitable linguistic review and preservation tests.

Follow stable MCP changes, especially host support for portable skills, structured tool results and resource discovery. Document which behavior belongs to Diffyn and which depends on the host rather than promising uniform support.

### Native host plugins

Offer small, independently testable contributions that package the existing Rust server and one canonical skill for a host's native extension system:

- [Claude Code plugin, issue 41](https://github.com/nathcymru/Diffyn/issues/41).
- [ChatGPT desktop plugin, issue 42](https://github.com/nathcymru/Diffyn/issues/42).
- [Mistral Vibe Code plugin, issue 43](https://github.com/nathcymru/Diffyn/issues/43).
- [Gemini CLI extension, issue 44](https://github.com/nathcymru/Diffyn/issues/44).

Each issue identifies prerequisites, source files, checks and completion criteria. Keep runtime behavior in Rust and use the host's required manifest and skill formats. Record tested platforms and account requirements; native desktop or CLI support does not imply support in a provider's web or mobile chat product. Public marketplace submission is a separate maintenance task.

## Optional Cloudflare deployment

[Issue 45](https://github.com/nathcymru/Diffyn/issues/45) proposes an installer for an authenticated MCP service in the user's own Cloudflare account, with direct client access or configuration through a Cloudflare MCP server portal. This is a staged engineering project, outside the beginner issue set.

1. **Establish the design:** document the remote privacy boundary, authentication, provider logging, resource limits and costs; prove shared Rust core compatibility with a Worker WebAssembly target.
2. **Build the service:** add a supported remote MCP transport and preserve behavior through shared native and hosted test cases; document any necessary non-Rust adapter.
3. **Prepare deployment artifacts:** publish verified bundles tied to immutable source commits and signed provenance.
4. **Build the installer:** provide guided configuration, a preview of resources and costs, and selectable `latest-stable`, `latest` and exact-version release choices.
5. **Verify operations:** test installation, portal configuration, upgrades, rollback, access control, interrupted-operation recovery and removal.

`latest-stable` is the default. The installer's opt-in `latest` channel may include a newer, explicitly published preview when preview publishing is supported. Both choices exclude draft releases and PR reservation tags. Resolve a release once and verify its complete deployment bundle; missing artifacts or invalid provenance must stop installation. Existing GitHub and container `latest` aliases remain stable-only.

Hosted processing changes where submitted content is handled. Publish that data flow and require explicit setup choices before enabling the deployment. Local installation remains available, and the project does not operate a shared hosted service as part of this proposal.

## Outside the planned scope

Diffyn will not infer diagnoses or abilities from personal labels, provide clinical diagnosis or treatment, claim to certify user understanding, or replace professional judgment. A centrally operated Diffyn service, content telemetry, persistent personal profiles and outbound model calls remain outside this roadmap. The optional user-owned Cloudflare deployment requires the public design and privacy review described above before implementation.

Propose changes through [issues](https://github.com/nathcymru/Diffyn/issues) or [Discussions](https://github.com/nathcymru/Diffyn/discussions), following the [contribution guide](../.github/CONTRIBUTING.md).
