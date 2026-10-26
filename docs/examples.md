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

- **Alternative template:** `neuro_inclusive_persona_wrapper` uses the same argument names.
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

- **Other resources:** `core-instructions`, `style-guide` and `evidence-and-limits` use the same `diffyn://guidance/` prefix.
- **Exact match:** File paths, external URLs, query strings and other URI variants are rejected.
- **Local content:** References are citations that the server does not fetch.

## Present the result usefully

**The host should show the relevant answer**, with enough uncertainty and source context for the person to make an informed choice.

- **Check warnings:** A protected section may have been preserved without analysis or formatting.
- **Keep deferred steps:** A short display window must not conceal later requirements.
- **Respect preferences:** The person can request plain text, more detail or a different presentation.
- **Avoid raw metadata dumps:** Explain meaningful limitations without making the person interpret every JSON field.

See the [communication contract](communication-contract.md) for the full behaviour and the [engineering guide](engineering.md) for connection setup.
