# Example requests

**These examples use synthetic text.** Tool examples show argument objects that an MCP client wraps in a `tools/call` request.

The client handles connection setup, and `tools/list` supplies the current tool schemas.

## Add space without rewriting

**Tool:** `reformat_for_cognitive_load`.

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

**Expected `formatted_text`:**

```text
The appointment is at 14:30. Bring the letter.

Do not stop your medication. Ask the clinician if you need advice.
```

- **Preserved:** Wording, the time and the instruction beginning with “Do not” stay intact.
- **Review:** `changes` describes the inserted break and `review_required` is true.
- **Protected content:** Structured text, code, quotations and links may remain unchanged.

The tool changes layout without writing a summary or changing the supplied medical instruction.

## Choose settings without a diagnosis

**Every profile field is optional**, and omitting `profile` uses neutral product defaults.

```json
{
  "text": "Open the document. Check the title. Save your changes.",
  "profile": {
    "preferences": {
      "max_sentences_per_chunk": 1
    }
  }
}
```

- **Optional labels:** A person may add `"self_declared_neurotypes": ["adhd", "dyslexia"]` alongside `preferences`.
- **Same settings:** Labels do not change defaults or establish reading ability.
- **Plain-text priority:** `plain_text: true` suppresses generated bold even when `use_bold` is true.
- **Exact fields:** The [communication contract](communication-contract.md#supported-preferences) lists all five preferences, defaults and bounds.

## Keep an existing sequence manageable

**Tool:** `generate_executive_function_checklist`.

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

**Expected result:** The first two actions appear in `extracted_steps`, and the third remains in `deferred_steps`.

- **Original retained:** `original_task` contains the full supplied sequence.
- **Next action:** `next_action` is `Find the appointment letter.`.
- **Visible limit:** The display window changes presentation without deleting later steps.
- **Review:** Supplied order, completeness and safety still need checking.

An explicit checked item such as `- [x] Find the appointment letter.` retains its completed status and is not selected as the next action.

## Handle a goal that needs more detail

**Tool:** `generate_executive_function_checklist`.

```json
{
  "complex_task": "Organise my move"
}
```

**Expected result:** `extraction_method` is `insufficient_structure`, with empty step arrays and a question in `missing_information`.

- **No invented plan:** The tool cannot infer the location, dates, equipment or prerequisites from this goal.
- **Focused clarification:** The host can ask for the most consequential missing information.
- **Host suggestions:** A model may propose a sequence separately if it labels assumptions and proposed steps clearly.

The clarification prompt below can help the host decide what to ask.

## Review a possible idiom

**Tool:** `verify_clarity_and_literalism`.

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

**Expected finding:** `touch base` appears at UTF-8 byte offsets **6–16**, with a conditional suggestion to name who should contact whom and why.

- **No rewrite:** `original_text` remains unchanged.
- **Context matters:** The phrase could be literal or appropriate in another context.
- **Preference effect:** Setting `literal_language` to false keeps the finding but omits its suggested interpretation.
- **Limited coverage:** No findings means only that the finite English phrase list found no matches in the scanned text.

Use byte offsets against the UTF-8 source rather than treating them as character indices.

## Request a prompt

**Method:** `prompts/get`.

```json
{
  "name": "adaptive_input_interrogator",
  "arguments": {
    "context": "I need to organise my move, but I do not know where to start.",
    "preferences": "Ask one question at a time. Use clear language without bold."
  }
}
```

**Expected result:** Separate user-role messages contain the template, supplied context and supplied preferences.

- **Other templates:** All [fourteen prompts](prompts.md) use the same argument names.
- **Required context:** `context` must be a nonblank string.
- **Free-text preferences:** Prompt preferences are a string, not the tool profile object.
- **Combined limit:** Context and preferences together may contain at most 32,768 UTF-8 bytes.
- **Host responsibility:** Templates do not execute model calls or replace higher-priority instructions.

## Read bundled guidance

**Method:** `resources/read`.

```json
{
  "uri": "diffyn://guidance/profile-matrix"
}
```

**Expected result:** One `text/markdown` resource contains the combined preference matrix.

- **Other resources:** Ten exact guidance URIs are listed in the [communication contract](communication-contract.md).
- **Exact match:** File paths, external URLs, query strings and other URI variants are rejected.
- **Local content:** References are citations that the server does not fetch.

## Carry checklist progress explicitly

After the first checklist call, copy its exact `source_revision` and the `step_id` to update. Send the same `complex_task`, the copied revision and `progress: [{"step_id": "the returned ID", "status": "completed"}]`. The quoted ID here describes where to copy a value; it is not a valid hard-coded identifier.

The completed item remains in `all_steps` and outside the active `extracted_steps` window. Source checkbox evidence remains unchanged. If the source changes, request a new checklist before applying progress. A moved display window does not authorize skipping earlier requirements.

## Resolve preferences for this request

**Tool:** `resolve_communication_preferences`.

```json
{
  "general": {"use_bold": true, "detail": "detailed"},
  "task": {"answer_order": "answer_first"},
  "current_request": {"plain_text": true, "detail": "concise"}
}
```

The result selects concise host detail, retains answer-first ordering and suppresses generated bold because plain text is selected. It records the source of each setting and the conflicts. Copy `tool_preferences` into `profile.preferences` for an original text tool; the host applies the separate host choices. Nothing is saved.

## Review changed details

**Tool:** `compare_protected_details`.

```json
{
  "original_text": "The fee is £25. I cannot attend on Friday.",
  "candidate_text": "The fee is £35. I can attend on Friday.",
  "protected_text": ["I cannot attend on Friday."]
}
```

The output reports changed recognised amounts and the missing protected refusal. A person or host must also review the meaning. Automatic matching does not detect every obligation or establish equivalence when counts match.

## Check a supplied action plan

**Tool:** `validate_action_plan`.

```json
{
  "steps": [
    {"id": "draft", "text": "Draft the outline", "origin": "user", "completion": "not_started"},
    {"id": "review", "text": "Review the outline", "origin": "proposal", "depends_on": ["draft"]}
  ]
}
```

This checks a supplied dependency structure. The proposed review remains distinct from the user's instruction. A structurally valid result does not show that the plan is complete or safe for the real task.

## Build a return point

**Tool:** `build_resume_card`.

```json
{
  "goal": "Finish the report",
  "steps": [
    {"id": "outline", "text": "Draft the outline", "status": "completed"},
    {"id": "figures", "text": "Check table 2", "status": "not_started"}
  ],
  "next_step_id": "figures",
  "blockers": []
}
```

The card retains the stated completion and records “Check table 2” as the explicitly supplied next action. It returns `saved: false`. Without `next_step_id`, it does not choose one.

## Explain a numerical calculation

**Tool:** `explain_numeric_values`.

```json
{
  "operation": "percentage_of",
  "value": "12.5",
  "other_value": "80",
  "decimal_places": 2
}
```

The result is `10`, with exact fraction `10/1` and `rounded: false`. This means 12.5% of 80. The tool does not infer whether the calculation is appropriate to a bill, risk statement or other context.

For a supported conversion, use `{"operation":"convert_unit","value":"90","from_unit":"min","to_unit":"h"}`. The result is 1.5 hours. Calendar dates and time zones are outside the local numeric tool.

## Check a requested response shape

**Tool:** `check_response_contract`.

```json
{
  "text": "## Next action\nOpen the report.",
  "preferences": {
    "required_headings": ["Next action"],
    "next_action_marker": "Open the report.",
    "max_paragraph_chars": 120
  }
}
```

The tool checks the supplied literal marker, heading and length preference. It reports what was checked and what remains unassessed; it cannot determine whether opening the report is the right action. The length preference is user-selected, not a clinical limit.

## Read a skill

A client supporting the Skills extension can call `skills/get` with:

```json
{"uri": "skill://diffyn-write-in-my-voice/SKILL.md"}
```

This returns the manifest. Use `resources/read` with that same URI for the skill text. A host without native skill discovery can install the portable folder instead; see the [skills guide](skills.md).

## Present the result usefully

**The host should show the relevant answer**, with enough uncertainty and source context for the person to make an informed choice.

- **Check warnings:** A protected section may have been preserved without analysis or formatting.
- **Keep deferred steps:** A short display window must not conceal later requirements.
- **Respect preferences:** The person can request plain text, more detail or a different presentation.
- **Avoid raw metadata dumps:** Explain meaningful limitations without making the person interpret every JSON field.

See the [communication contract](communication-contract.md) for the full behaviour and the [engineering guide](engineering.md) for connection setup.
