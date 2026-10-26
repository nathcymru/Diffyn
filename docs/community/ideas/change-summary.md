---
title: "Idea: a clearer summary of what formatting changed"
category: ideas
---

**Proposal for discussion:** hosts could present a compact explanation of layout changes without exposing the entire JSON result.

Diffyn already returns `changes`, warnings and the original text.
The open question is how to make that information useful in the final interface.

## A possible presentation

> Two paragraph breaks were added; the wording is unchanged.

That sentence would be appropriate only when it accurately describes the tool result.
Protected sections and warnings would still need to remain visible when relevant.

## Design questions

- **Placement:** should the explanation appear beside the result or in an optional details view?
- **Comparison:** would highlighting only changed whitespace help or distract?
- **Access:** how should a screen reader announce the difference?

Which detail would help you decide whether to accept the formatted result?
This discussion does not commit the project to a particular interface.
