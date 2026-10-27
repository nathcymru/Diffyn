# Skills

Diffyn 2.0.0 includes **six focused workflows**. Each is a self-contained `SKILL.md` with discovery metadata, operational guidance, an example and failure handling. A skill guides the host model; it does not run an LLM, send a message or grant permission to act elsewhere.

| Skill | Use it for |
| --- | --- |
| [diffyn-understand-document](../skills/diffyn-understand-document/SKILL.md) | Explain a document or message while retaining facts and uncertainty |
| [diffyn-task-support](../skills/diffyn-task-support/SKILL.md) | Clarify, start or resume work from supplied state |
| [diffyn-repair-communication](../skills/diffyn-repair-communication/SKILL.md) | Clarify an exchange or request a communication adjustment |
| [diffyn-write-in-my-voice](../skills/diffyn-write-in-my-voice/SKILL.md) | Edit writing while preserving voice, boundaries and commitments |
| [diffyn-make-decision](../skills/diffyn-make-decision/SKILL.md) | Compare choices using the person's priorities |
| [diffyn-prepare-conversation](../skills/diffyn-prepare-conversation/SKILL.md) | Prepare goals, questions, boundaries and optional rehearsal |

## Portable installation

Copy the selected folder from `skills/` into the skills directory supported by your host. Keep the folder name and its `SKILL.md` together. Hosts differ in where skills are installed, how discovery works and whether a restart is required; follow the host's current documentation. Installing all six is optional.

Release archives include the `skills/` directory and this guide as `SKILLS.md`. The Cargo source package also includes the workflows. The server embeds the same skill files at build time, so changing an installed copy does not change a running executable.

If the host has no skill discovery, open the relevant file and use its instructions for the current task. Each workflow has a plain-text fallback and can operate without Diffyn's MCP tools. It should discover available schemas before using a tool.

## Native MCP discovery

Diffyn advertises the `io.modelcontextprotocol/skills` extension. A supporting client can call `skills/list` for the manifest and `skills/get` with its exact `uri` for one manifest. Manifests include exact file URIs, byte sizes and SHA-256 digests. The only exposed skill file is each workflow's `SKILL.md`; directory browsing and arbitrary file access are not supported.

For example, `skills/get` accepts `{"uri":"skill://diffyn-task-support/SKILL.md"}`. Skill text is available through `resources/read` at the same exact URI:

```json
{
  "uri": "skill://diffyn-task-support/SKILL.md"
}
```

The standard `resources/list` registry remains the ten guidance resources. A client that does not implement the Skills extension can use those resources, prompts, or portable files instead.

## Example requests

- “Use the document workflow to explain what I need to do, keeping the original deadlines.”
- “Help me resume this task from my notes. Do not assume any other steps are complete.”
- “Rewrite this in my voice. Keep the refusal and the condition on Tuesday.”
- “Help me prepare a workload conversation, including what I will not commit to.”

## Boundaries

The host controls model behaviour, permissions, language capabilities, rendering and retention. Skill instructions cannot guarantee correct interpretation or remove bias. Diffyn does not save preferences, retain progress, schedule reminders or send drafts. Source-based checks support review; they do not certify meaning, safety, clinical benefit or accessibility conformance.
