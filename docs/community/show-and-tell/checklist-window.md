---
title: "Maintainer example: show two steps and retain the third"
category: show-and-tell
---

**This is a maintainer-created example using synthetic text.** It shows how a display window differs from deleting later work.

## Request

Call `generate_executive_function_checklist` with:

```json
{
  "complex_task": "1. Open the document.\n2. Check the title.\n3. Save a copy.",
  "profile": {
    "preferences": {
      "max_checklist_items": 2
    }
  }
}
```

## What to inspect

- **Active group:** `extracted_steps` contains the first two actions.
- **Later work:** `deferred_steps` retains “Save a copy.”
- **Next action:** `next_action` is “Open the document.”
- **Original:** `original_task` keeps the complete source sequence.

The host should keep the remaining work discoverable.
A group boundary does not establish that pausing there is safe or that the sequence is complete.
