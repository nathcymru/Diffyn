---
title: "Maintainer example: request focused clarification"
category: show-and-tell
---

**This is a maintainer-created example using synthetic text.** It demonstrates fetching guidance for a host, not a guaranteed model response.

## Prompt request

Send `prompts/get` with:

```json
{
  "name": "adaptive_input_interrogator",
  "arguments": {
    "context": "I need to prepare a short presentation, but the audience and purpose are unclear.",
    "preferences": "Ask one useful question at a time. Keep technical terms when they matter."
  }
}
```

## What happens

Diffyn returns a static template and separate user-role messages containing the context and preferences.
It does not run a model call or answer the question itself.

**An illustrative host question:** “Who is the presentation for?”
The host should choose the most consequential gap in the actual task rather than follow a fixed questionnaire.

A diagnosis or personal history is unnecessary for this clarification.
