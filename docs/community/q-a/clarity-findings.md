---
title: "Does “no clarity findings” mean my wording is clear?"
category: q-a
---

**No.** An empty findings list does not establish clear wording. It can mean that the enabled phrase rules found no matches, or that no eligible text was analyzed.

## What a result can tell you

- **A match:** a known phrase may need contextual review.
- **An interpretation:** an optional suggestion can explain a possible figurative meaning.
- **Coverage:** `language_supported`, `applied_categories` and `analyzed` show whether English rules were available, enabled and applied to eligible text.
- **Skipped material:** protected spans are left unanalyzed while eligible surrounding prose can still be checked.
- **No rewrite:** the original text remains unchanged.

An unsupported language, an empty category list or fully protected text can all produce no findings without analysis. When `analyzed` is true, no findings means only that the enabled rules found no matches in the eligible text.

A phrase can be literal, technical or appropriate in context.
The tool cannot detect every ambiguity, infer sarcasm reliably or verify another person's understanding.

Use its findings as prompts for review, not as a clarity score or accessibility certificate.
See the [clarity contract](https://github.com/nathcymru/Diffyn/blob/main/docs/communication-contract.md#clarity-tool) for exact coverage and limits.
