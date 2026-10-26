---
title: "Maintainer example: review an idiom without silently replacing it"
category: show-and-tell
---

**This is a maintainer-created example using synthetic text.** The phrase may be appropriate; the purpose is to make review possible.

## Input

Call `verify_clarity_and_literalism` with:

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

## What the result offers

- **A possible idiom:** “touch base” is flagged for contextual review.
- **A conditional interpretation:** the host can consider naming who should contact whom and why.
- **The unchanged source:** the tool does not silently rewrite the sentence.

“Tomorrow” may also need a concrete date or time in a real task, but that does not mean this phrase matcher detects every such ambiguity.

Keep review broader than the returned list of matches.
