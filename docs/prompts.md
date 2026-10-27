# Prompt templates

Diffyn 2.0.0 provides **14 prompts** for the host model. They are reusable instructions, not local model calls or guaranteed outcomes. Each accepts required nonblank `context` and optional plain-text `preferences`, with a combined limit of 32,768 UTF-8 bytes. Unknown arguments and retry state are rejected.

| API name | Workflow |
| --- | --- |
| `neuro_inclusive_persona_wrapper` | [Apply explicit communication preferences](prompts/persona-wrapper.md) |
| `adaptive_input_interrogator` | [Focused clarification](prompts/adaptive-interrogator.md) |
| `choose_communication_style` | [Preview and choose a format](prompts/choose-communication-style.md) |
| `explain_in_layers` | [Keep an overview and supporting detail consistent](prompts/explain-in-layers.md) |
| `understand_message` | [Separate stated content from possible interpretations](prompts/understand-message.md) |
| `repair_communication` | [Clarify mismatched expectations](prompts/repair-communication.md) |
| `rewrite_in_my_voice` | [Edit without changing authorship or commitments](prompts/rewrite-in-my-voice.md) |
| `clarify_task_brief` | [Identify requirements and missing information](prompts/clarify-task-brief.md) |
| `start_a_task` | [Find a feasible first action](prompts/start-a-task.md) |
| `resume_after_interruption` | [Return to supplied task state](prompts/resume-after-interruption.md) |
| `compare_options` | [Compare choices using stated priorities](prompts/compare-options.md) |
| `explain_numbers_and_dates` | [Explain exact values and calendar assumptions](prompts/explain-numbers-and-dates.md) |
| `prepare_conversation` | [Prepare goals, questions and boundaries](prompts/prepare-conversation.md) |
| `request_communication_adjustment` | [Draft a practical adjustment request](prompts/request-communication-adjustment.md) |

## Request a prompt

Send this object as the parameters of `prompts/get` after the client's MCP connection is established:

```json
{
  "name": "rewrite_in_my_voice",
  "arguments": {
    "context": "Make this clearer: I cannot do Friday. Tuesday works if the figures arrive first.",
    "preferences": "Keep my direct tone. Do not add an apology."
  }
}
```

The result contains separate user-role messages for the static template, supplied context and optional preferences. The host applies its own instruction hierarchy and performs the requested reasoning. Labels and message separation provide provenance, not a guarantee against prompt injection.

## Compatibility

The two original API names remain available. `adaptive_input_interrogator` is displayed as **Focused clarification**. All 14 prompts share the same argument contract. Clients may expose prompts differently or not support them; the linked Markdown can also be used as ordinary guidance.

For longer workflows, see the [six portable skills](skills.md). For deterministic operations and their limits, see the [communication contract](communication-contract.md).
