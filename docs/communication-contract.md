# Communication contract

**The person's expressed preferences come first.** Diffyn provides nine deterministic tools, fourteen prompts, ten guidance resources and six skills. No diagnosis or ability assessment is required.

This page describes implemented interfaces. The [research basis](research.md) separates supporting evidence from engineering choices. The host performs semantic reasoning, presents results and controls its own permissions and retention.

## Respect and control

Address capable teenagers and adults without infantilising them. Preserve exact values, conditions, negation, uncertainty, obligations and safety information. Keep the person's voice and boundaries intact. Do not infer diagnosis, intelligence, consent capacity, feelings or completed work from language.

The intended audience is **16+**, or **13+ with strict adult supervision**. This is an audience statement, not an age-verification service. The person can request another format, more detail or no accommodation.

## Shared validation

Unknown fields are rejected at typed request boundaries. Invalid or oversized input is rejected without silent truncation. Error messages exclude supplied content. All offsets are half-open UTF-8 byte ranges, not character positions.

`tools/list` provides generated input and output schemas. Runtime byte and aggregate limits also apply; a schema character bound does not replace byte validation. Tool results contain structured content and a matching JSON text representation for client compatibility.

## Supported preferences

The original layout, checklist and clarity tools accept optional `profile`:

```json
{
  "preferences": {
    "max_sentences_per_chunk": 2,
    "max_checklist_items": 7,
    "use_bold": false,
    "plain_text": false,
    "literal_language": true
  }
}
```

| Field | Default | Local effect |
| --- | --- | --- |
| `max_sentences_per_chunk` | `2` | Groups eligible prose; allowed range 1–5 |
| `max_checklist_items` | `7` | Active checklist window; allowed range 1–20 |
| `use_bold` | `false` | Permits the generated checklist title to use emphasis |
| `plain_text` | `false` | Suppresses generated decoration without stripping source formatting |
| `literal_language` | `true` | Includes conditional interpretations with clarity findings |

**Plain text suppresses generated bold.** The formatter does not choose words to emphasise. Omitted fields use neutral product defaults. These defaults are not clinical thresholds.

For compatibility, the profile also accepts up to five optional `self_declared_neurotypes` entries from `autism`, `adhd`, `dyslexia`, `dyspraxia` and `audhd` (`AuDHD` is an alias). Labels never change settings and are unnecessary for every feature. Host choices such as tone and answer order are not additional fields in this legacy profile.

## Layout tool

`reformat_for_cognitive_load` accepts required nonblank `text` up to **32,768 UTF-8 bytes** and optional `profile`.

The tool changes eligible inter-sentence whitespace into paragraph breaks, preserving every non-whitespace character. `original_text` and `formatted_text` remain available. Protected spans identify source material that is left intact; surrounding eligible prose can still be formatted. Warnings and `changes` explain the result. `review_required` is true when changes or warnings occur.

This is not summarisation, vocabulary simplification or a measure of cognitive load. An unchanged result is not accessibility certification.

## Checklist tool

`generate_executive_function_checklist` accepts nonblank `complex_task` up to **32,768 UTF-8 bytes**, optional `profile`, optional `source_revision`, optional `start_at_step_id` and a default-empty `progress` array.

Each progress item contains `step_id` and `status`, where status is `pending`, `completed`, `blocked` or `skipped`. At most **1,024 source steps and 1,024 progress entries** are accepted. Copy the prior result's revision and IDs; they are opaque values bound to unchanged source text. Progress or a requested starting ID requires the matching `source_revision`. Stale revisions, unknown IDs and duplicate progress entries are rejected.

The tool extracts explicit marked lists or supported English and Welsh sequence cues. It does not infer a procedure from a broad goal. Such a goal returns `insufficient_structure`, empty steps and missing information. `suggested_steps` remains empty; host-proposed actions must be labelled separately.

| Output | Meaning |
| --- | --- |
| `all_steps` | Every extracted source item in order |
| `extracted_steps` | Active window of unfinished items, including blocked items |
| `deferred_steps` | Every source item outside that window, including completed or skipped items |
| `step_id`, `status`, `status_origin` | Stable source identity and the origin of progress information |
| `completed` | Original checkbox evidence only; caller progress does not rewrite it |
| `source_revision`, `active_start_step_id` | Values for explicitly carrying state to another call |
| `next_action` | First eligible action in the active view; a blocked leading item does not silently promote a later item |

All source text stays available. Moving the window does not approve skipping dependencies. Every checklist requires review of completeness, chronology and action granularity. A pause marker is only a presentation suggestion.

## Clarity tool

`verify_clarity_and_literalism` accepts nonblank `text` up to **32,768 UTF-8 bytes**, optional `profile`, optional `language` and optional `categories`.

Omitted language defaults to `en`. A supplied tag contains at most **35 ASCII letters, digits and hyphens** in nonempty segments. English tags are supported; other languages return explicit unsupported coverage with no analysis. This is declared coverage, not automatic language detection.

Categories are `english_idioms`, `vague_time` and `vague_reference`, each at most once. Omission enables the original nine English idiom rules; an empty list disables rules. The optional categories are finite phrase checks, not semantic detectors.

Results retain the source and include exact finding offsets, rule IDs, applied categories, language support, protected spans, analyzed bytes and skipped protected bytes. At most **128 findings** are expanded; total and remaining counts are explicit. Conditional interpretations follow the `literal_language` preference. Every result requires contextual review, including a result with no matches.

## Preference resolver

`resolve_communication_preferences` accepts optional `general`, `task` and `current_request` objects. Explicit values apply in that order, with the most specific supplied value winning. Every layer is validated, even if a later layer overrides it.

Each layer supports the five local preferences above, plus these host preferences:

| Field | Allowed values | Default |
| --- | --- | --- |
| `language` | Nonblank text, at most 64 UTF-8 bytes | Unspecified |
| `detail` | `concise`, `standard`, `detailed` | `standard` |
| `tone` | `neutral`, `warm`, `direct` | `neutral` |
| `answer_order` | `answer_first`, `context_first` | `answer_first` |
| `question_style` | `one_at_a_time`, `grouped` | `one_at_a_time` |

The result separates `tool_preferences` from `host_preferences` and records `sources`, `conflicts`, `precedence` and limitations. It does not apply host preferences, translate text or save a profile. Copy the local settings into `profile.preferences` for a compatible tool.

## Protected-detail comparison

`compare_protected_details` accepts `original_text`, `candidate_text` and optional `protected_text`. Each main text is nonblank and at most **32,768 UTF-8 bytes**. At most **64 explicit protected strings** are allowed, each nonblank and at most **1,024 bytes**, with an aggregate protected-string limit of **32,768 bytes**.

Finite automatic recognition covers numeric strings, selected units and date forms, HTTP(S) URLs and backtick or tilde code. It can recognise forms such as `£25`, `10 mg`, `2026-10-27`, `27 October 2026` and `27 Hydref 2026`; it does not interpret dates, convert units or equate differently written values. URL candidates retain ambiguous trailing punctuation for review.

Explicit protected strings let the caller include names, refusals, conditions or other critical wording. Each detail reports exact occurrence counts and source/candidate byte spans. At most **128 groups** and **16 spans per text per group** are expanded; full counts and omitted/truncated indicators remain available.

`semantic_equivalence_checked` is always false and `review_required` is always true. Equal counts do not establish that a value remains attached to the correct person, condition or obligation. Automatic recognition is not an exhaustive inventory of meaningful details.

## Supplied-plan validation

`validate_action_plan` accepts optional `source_text` and **1–100** `steps`. Each step has `id`, `text`, `origin`, optional `source_ref`, optional `depends_on` and optional `completion`.

- `origin` is `source`, `user` or `proposal`.
- `source_ref` contains `start_byte` and `end_byte` and must exactly identify the step text in `source_text`; source-origin steps require a reference.
- `completion` is `unknown` by default, or `not_started`, `in_progress`, `completed`, `pending`, `blocked`, `skipped`.
- IDs are nonblank and at most **64 bytes**, text at most **4,096 bytes**, and dependencies at most **100** per step.
- Combined source, step text and identifiers are limited to **32,768 UTF-8 bytes**.

The result reports structural findings, cycles, `structurally_valid`, supplied steps and mandatory review. Invalid references, inconsistent completion claims and graph problems are findings; malformed or oversized arguments are errors. A proposal cannot establish completed work. The tool does not invent, reorder, execute or certify a procedure. Findings may be bounded with total and omitted counts retained.

## Resume cards

`build_resume_card` accepts `goal`, `steps`, optional `current_step_id`, optional `next_step_id` and optional `blockers`. Each step has `id`, `text` and explicit `status` from `unknown`, `not_started`, `in_progress`, `completed`, `pending`, `blocked`, `skipped`.

Goal, step text and individual blockers are nonblank and at most **4,096 bytes**. IDs are at most **64 bytes**. There are at most **100 steps**, **20 blockers** and **32,768 combined UTF-8 bytes**. Current and next IDs must identify supplied steps that are neither completed nor skipped. A blocked step may be the current state but cannot be selected as the next action.

The result includes the supplied state, completed IDs, selected current/next steps, blockers, a plain-text card, provenance and `saved: false`. No next action is inferred when `next_step_id` is absent. Checklist statuses can be carried into the card. The card also accepts `unknown`, `not_started` and `in_progress`; these are explicit caller-supplied states, not inferred transitions.

## Numeric explanations

`explain_numeric_values` accepts `operation`, decimal-string `value`, optional `other_value`, optional `from_unit` and `to_unit`, and `decimal_places` from **0–9**, default **6**.

Operations are `add`, `subtract`, `multiply`, `divide`, `percentage_of`, `percentage_change` and `convert_unit`. `percentage_of` calculates `value / 100 × other_value`; `percentage_change` treats `value` as the old value and `other_value` as the new value. Plain decimal inputs allow at most **18 integer digits and 9 fractional digits**. Checked integer rational arithmetic rejects unsupported values, zero divisors and overflow.

Supported conversions stay within one dimension: `mm`, `cm`, `m`, `km`; `mg`, `g`, `kg`; `ml`, `l`; or `ms`, `s`, `min`, `h`. Unsupported units and irrelevant operation fields are rejected. There is no currency, calendar, timezone or dosage engine.

Results include `inputs`, decimal `result`, optional `result_unit`, `exact_fraction`, `rounded`, the rounding rule, an explanation and limitations. A displayed rounded decimal is not the exact fraction. Numerical correctness does not establish the suitability of an input or professional recommendation.

## Response-contract checking

`check_response_contract` accepts nonblank `text` and a required `preferences` object. Preferences may set `max_paragraph_chars` (**20–10,000**), `max_sentence_words` (**3–200**), `required_headings` (at most **20**), `forbidden_terms` (at most **50**), `plain_text` or a literal `next_action_marker`.

Headings, terms and a supplied marker are nonblank, contain no line breaks and are at most **128 bytes** each; combined text and preference strings are at most **32,768 UTF-8 bytes**.

| Check | What it measures |
| --- | --- |
| Paragraph length | Unicode scalar values in trimmed, blank-line-separated paragraphs; these are not necessarily displayed characters |
| Sentence length | Whitespace-separated words in segments ending at `.`, `!`, `?` or a newline; abbreviations, numbers and code need review |
| Required headings | Exact text on a trimmed standalone line, optionally preceded by Markdown `#` markers |
| Forbidden terms | Case-sensitive literal matches, including substrings within longer words |
| Plain text | Common Markdown markers for headings, strong emphasis, backtick code and links; this is not a complete markup parser |
| Next-action marker | Literal presence of the supplied text, without assessing whether the action is useful or safe |

At most **128 findings** are expanded, with total and omitted counts, `checked`, `unassessed` and mandatory review. There is no generic pass, cognitive-load score, rewrite or guarantee that a response is usable. Limits express the caller's preference and are never inferred from a diagnosis.

## Prompts, resources and skills

All [fourteen prompts](prompts.md) accept required nonblank `context` and optional plain-text `preferences`, sharing a **32,768-byte** limit. The static template, context and nonblank preferences are separate user-role messages. Unknown arguments, retry state and input responses are rejected. Prompt guidance does not replace host instructions or execute a model call.

The ten guidance URIs use `diffyn://guidance/` with these exact names:

- `core-instructions`
- `style-guide`
- `profile-matrix`
- `evidence-and-limits`
- `preferences-and-scope`
- `communication-repair-and-voice`
- `task-planning-and-resumption`
- `numbers-dates-and-units`
- `language-and-host-capabilities`
- `evidence-to-feature-map`

Guidance is embedded Markdown with actual byte sizes. The [six skills](skills.md) use the advertised `io.modelcontextprotocol/skills` extension and exact `skill://<skill-name>/SKILL.md` reads. Resource and skill requests never resolve caller-chosen files or network addresses. Registry continuation and retry state are not supported.

## Host presentation and privacy

Show the useful result and relevant uncertainty without requiring the person to interpret all metadata. Review warnings, coverage, omitted findings, preserved source content and any semantic transformation. A tool's successful execution is not approval for an external action.

Diffyn makes no model calls and does not persist submitted text, preferences or task state. The host controls typography, assistive technology, model use and retention. Local processing does not guarantee end-to-end privacy, memory zeroisation or protection from operating-system crash dumps.
