# Example requests

These examples use synthetic text. The objects are tool arguments; the host MCP
client wraps them in a `tools/call` request and handles the protocol lifecycle.
Names and schemas are discoverable through `tools/list`.

## Layout without changing meaning

Tool: `reformat_for_cognitive_load`

```json
{
  "text": "The appointment is at 14:30. Bring the letter. Do not stop your medication. Ask the clinician if you need advice.",
  "profile": {
    "preferences": {
      "max_sentences_per_chunk": 2,
      "plain_text": true
    }
  }
}
```

The formatter can insert paragraph breaks while preserving wording and exact
values. It does not write a summary or change the medical instruction. Structured
content, code, quotations, and linked text may be left intact to preserve meaning.
Always inspect `changes`, `warnings`, `review_required`, and `limitations`.

Every profile field is optional. If useful, a person may supply
`"self_declared_neurotypes": ["adhd", "dyslexia"]` alongside `preferences`.
Labels never change the settings or establish the person's reading ability.

Supported explicit preferences:

| Field | Default | Meaning |
| --- | --- | --- |
| `max_sentences_per_chunk` | `2` | A layout grouping preference from 1 to 5 |
| `max_checklist_items` | `7` | An active group of 1 to 20 steps; other steps are retained |
| `use_bold` | `false` | Optional generated emphasis where supported; the formatter does not infer which words are important |
| `plain_text` | `false` | Suppress generated decoration; preserves existing source formatting |
| `literal_language` | `true` | Include optional literal alternatives; never silently rewrite meaning |

Plain text takes precedence over generated bold when both are requested. These
defaults are configurable product choices, not clinical thresholds.

## Make existing actions easier to follow

Tool: `generate_executive_function_checklist`

```json
{
  "complex_task": "1. Find the appointment letter.\n2. Check the date and location.\n3. Put the letter in your bag.",
  "profile": {
    "preferences": {
      "max_checklist_items": 2
    }
  }
}
```

The result retains all supplied actions. The preference controls the active group,
not deletion of later actions. The original remains available for review. The
tool does not establish that the supplied ordering is safe or complete.

For a broad request such as `"Organise my move"`, the local tool should report
insufficient structure instead of inventing a plan. An upstream model can use
the clarification prompt to gather the most useful missing information and
clearly label any proposed steps as suggestions.

## Review possible ambiguity

Tool: `verify_clarity_and_literalism`

```json
{
  "text": "Let's touch base tomorrow.",
  "profile": {
    "preferences": {
      "literal_language": true
    }
  }
}
```

The tool preserves the source and flags known phrases with optional explanations.
A phrase can have a literal meaning in context; every finding needs review.
No findings means only that the finite phrase list found no matches, not that the
text is clear or free of sarcasm. English phrase heuristics are not a multilingual
language-understanding system.

## Request a reusable prompt

Send `prompts/get` with:

```json
{
  "name": "adaptive_input_interrogator",
  "arguments": {
    "context": "I need to organise my move, but I do not know where to start.",
    "preferences": "Ask one question at a time. Use clear language without bold."
  }
}
```

The alternative prompt name is `neuro_inclusive_persona_wrapper`. Both accept
required nonblank `context` and optional plain-text `preferences`, with a combined
32 KiB UTF-8 limit. They return static guidance and separate messages containing
the supplied data. They do not execute model calls or override the host's rules.

## Read bundled guidance

Send `resources/read` with:

```json
{
  "uri": "diffyn://guidance/profile-matrix"
}
```

Other exact URIs are `diffyn://guidance/core-instructions`,
`diffyn://guidance/style-guide`, and `diffyn://guidance/evidence-and-limits`.
No file paths, external URLs, or URI variants are resolved. References in the
resources are citations; the server does not fetch them.
