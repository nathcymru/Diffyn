---
title: "Maintainer example: add paragraph breaks without rewriting"
category: show-and-tell
---

**This is a maintainer-created example using synthetic text.** It demonstrates layout behavior, not a user testimonial or a measured accessibility outcome.

## Request

Call `reformat_for_cognitive_load` with:

```json
{
  "text": "Open the report. Check the title. Save the file.",
  "profile": {
    "preferences": {
      "max_sentences_per_chunk": 1,
      "plain_text": true
    }
  }
}
```

## Expected layout

```text
Open the report.

Check the title.

Save the file.
```

**What changed:** paragraph breaks make each existing sentence a separate block.
The wording and order remain intact, and the original is included in the result.

A different person or task may benefit from a more compact layout.
The [examples guide](https://github.com/nathcymru/Diffyn/blob/main/docs/examples.md) shows other settings and review requirements.
