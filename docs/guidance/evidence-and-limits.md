# Evidence and accessibility limits

**Purpose:** Diffyn offers local text tools and reusable communication guidance;
it does not diagnose, assess capacity, provide treatment, or guarantee
comprehension.

## Interpret the evidence

- **Standards:** WCAG 2.2 addresses complete rendered products, so a prompt or server test cannot establish conformance ([w3c-wcag22-2024](https://github.com/nathcymru/Diffyn/blob/main/docs/bibliography.bib)).
- **Practice guidance:** W3C COGA supports clear content and personalization as supplemental guidance, separate from mandatory WCAG success criteria ([w3c-coga-2021](https://github.com/nathcymru/Diffyn/blob/main/docs/bibliography.bib)).
- **Clinical context:** applying NICE communication recommendations to LLM output is an engineering extrapolation ([nice-cg142](https://github.com/nathcymru/Diffyn/blob/main/docs/bibliography.bib), [nice-ng87](https://github.com/nathcymru/Diffyn/blob/main/docs/bibliography.bib)).
- **Study limits:** research on a specialist typeface in children and Bionic Reading in fluent adult readers does not establish a universal neurodivergent formatting prescription ([joseph-powell-2022](https://github.com/nathcymru/Diffyn/blob/main/docs/bibliography.bib), [beelders-2025](https://github.com/nathcymru/Diffyn/blob/main/docs/bibliography.bib)).
- **Product choices:** sentence grouping, checklist size, and input limits are configurable engineering decisions, not measured cognitive thresholds.

## Primary studies added for 2.0.0

Exploratory studies of neurodivergent LLM use inform discoverability, task resumption and voice preservation. A CHI 2026 audit of six models and 345,000 outputs, accompanied by 11 autistic interviews, identifies stereotype risks when personalising advice from diagnostic disclosure. A participatory Singapore study with 20 autistic adults motivates reciprocal communication and adjustment requests. These are design inputs, not evaluations of Diffyn. The [evidence-to-feature map](https://github.com/nathcymru/Diffyn/blob/main/docs/guidance/evidence-to-feature-map.md) identifies each source and limit.

## Tool and privacy boundaries

**Conservative operations:** layout changes, step extraction, bounded phrase matching, protected-detail comparisons, structural plan checks and output-shape checks cannot establish complete meaning, infer a safe procedure or detect all ambiguity. Exact supported arithmetic does not establish that a calculation is appropriate to the situation.
Keep originals available and review uncertain results.

**Local processing:** Diffyn does not persist profiles or submitted text, make
outbound model calls, or log request content at the application level.
The host client may retain the conversation independently.

**Fixed resources:** the server returns bundled text from an exact URI
allowlist; links are references and are not fetched by the server.
Prompt labels and separate messages do not guarantee resistance to injection.

## Evaluation remains necessary

**Host responsibility:** fonts, colours, spacing, semantic rendering, keyboard
interaction, and assistive-technology integration depend on the client.

**User involvement:** assess task success, retained meaning, effort, and
preference fit with neurodivergent users; software checks cannot substitute for
participatory evaluation.

**Further detail:** the
[research report](https://github.com/nathcymru/Diffyn/blob/main/docs/research.md)
explains the evidence and evaluation priorities, and the
[bibliography](https://github.com/nathcymru/Diffyn/blob/main/docs/bibliography.bib)
provides complete source metadata.
