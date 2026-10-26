---
title: "Maintainer example: keep completed checkboxes out of the next action"
category: show-and-tell
---

**This is a maintainer-created example using synthetic text.** Completion comes from an explicit checkbox in the source, not from the tool observing an action.

## Source checklist

```text
- [x] Open the document.
- [ ] Check the title.
- [ ] Save a copy.
```

Pass this text as `complex_task` to `generate_executive_function_checklist`.

## Expected behavior

- **Completed item:** “Open the document.” retains its checked status.
- **Next action:** “Check the title.” is the first item not explicitly marked complete.
- **Preserved source:** the original checklist remains available for review.

The result does not prove that the document was opened or that saving a copy is the right next procedure.
It reflects the supplied marks and ordering.

This distinction helps a host avoid presenting an already checked source item as the next action.
