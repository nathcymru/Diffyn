---
title: "Why did the formatter leave my text unchanged?"
category: q-a
---

**An unchanged result can be deliberate.** The layout tool changes eligible inter-sentence whitespace while preserving wording.

This maintainer-written FAQ describes its conservative behavior.

## Common reasons

- **Existing structure:** lists or other markup may already provide organization.
- **Protected material:** code, links, quotations or mathematical content may be left intact.
- **No eligible split:** the text may not contain a recognized place to add a paragraph break.

Check the returned `changes`, `warnings`, `limitations` and `review_required` fields rather than assuming that unchanged text passed an accessibility test.

The tool does not summarize a passage, simplify its vocabulary or determine whether a particular person will understand it.

If the result seems wrong, share a short synthetic input and the layout you expected.
The [layout contract](https://github.com/nathcymru/Diffyn/blob/main/docs/communication-contract.md#layout-tool) explains the boundary.
