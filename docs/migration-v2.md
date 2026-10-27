# Migrate to Diffyn 2.0.0

Version 2.0.0 expands the local server from three tools, two prompts and four resources to **nine tools, fourteen prompts, ten guidance resources and six skills**. The original API names remain available. The major version also marks changes to checklist presentation and richer result schemas.

## Upgrade the executable

Install the published 2.0.0 tag below, or select a newer compatible 2.x package from [GitHub Releases](https://github.com/nathcymru/Diffyn/releases). The [source guide](engineering.md#build-the-executable) also covers repository builds.

```sh
cargo install --git https://github.com/nathcymru/Diffyn --tag v2.0.0 --locked diffyn
```

Restart or reconnect the MCP client so it starts the new executable and refreshes discovery. A server embedded into a running client is not updated by editing Markdown files. Confirm the reported server version and discover the current schemas before making requests. See [installation](engineering.md) and [package verification](releases/README.md).

## Existing integrations

| Surface | Migration action |
| --- | --- |
| Original tool names | Keep the same names and basic requests; accept the new output fields |
| Optional profile | Existing five local preferences and optional labels remain valid; labels still have no effect on defaults |
| Checklist active group | `extracted_steps` now contains unfinished items; use `all_steps` for the complete ordered source |
| Checklist deferred group | Expect completed, skipped and earlier items as well as later items; it means outside the active window |
| Checklist progress | Carry the unchanged source, exact `source_revision` and step IDs; supply progress explicitly |
| Original checkbox field | `completed` remains source evidence; use `status` and `status_origin` for effective supplied progress |
| Clarity checking | Omitted categories retain the nine idiom rules; opt into `vague_time` and `vague_reference` explicitly |
| Language coverage | English remains the supported clarity language; unsupported declared languages return no analysis |
| Protected content | Layout and clarity now report spans and can process eligible prose around protected content |
| Prompt identifiers | Both old identifiers remain; the interrogator's displayed title becomes Focused clarification |
| Guidance URIs | All four old exact URIs remain, with six additions |

If the checklist source changes, obtain new IDs and a new revision. Do not carry old progress across edited text or silently map steps by position. A blocked leading action prevents automatic promotion of the next action, and moving a window past unfinished work produces a warning.

## New capabilities

Use `resolve_communication_preferences` to combine explicitly supplied general, task and current-request settings. Copy `tool_preferences` into a legacy tool's `profile.preferences`; apply `host_preferences` through the host. Nothing is saved automatically.

Use protected-detail comparison, plan validation, resume cards, exact numeric explanation and response-contract checking for their specific deterministic operations. Review their coverage and limitations. None is a semantic, safety or accessibility certificate. See the [full contract](communication-contract.md) and [examples](examples.md).

## Skills and client compatibility

A client implementing `io.modelcontextprotocol/skills` can discover the six workflows natively. Other hosts can install the portable skill folders or use ordinary prompts and resources. The Skills extension does not add arbitrary filesystem reads or grant action permissions. Follow the [skills guide](skills.md) for both routes.

## Privacy and testing

The server remains local, stateless and free of outbound model calls. Hosts retain control of model use, presentation and their own data policies. Test the upgraded client against representative text, changed source revisions and unsupported language cases before relying on it. Software tests do not replace user or host accessibility evaluation.
