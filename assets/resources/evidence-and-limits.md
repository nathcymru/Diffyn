# Evidence and accessibility limits

Reviewed 26 October 2026. Diffyn provides local communication aids and reusable guidance. It does not diagnose, assess capacity, provide clinical treatment, or guarantee comprehension.

## Evidence types

- Standards describe requirements for a complete rendered product. [WCAG 2.2](https://www.w3.org/TR/WCAG22/) conformance cannot be inferred from a prompt or successful server test.
- Practice guidance offers starting points to adapt. [W3C COGA](https://www.w3.org/TR/coga-usable/) is supplemental guidance, separate from mandatory WCAG success criteria.
- Clinical guidance has its own setting. Applying [NICE adult autism recommendations](https://www.nice.org.uk/guidance/CG142/chapter/Recommendations) to LLM communication is an engineering extrapolation.
- Primary studies apply to their samples and tasks. A [specialist-font study](https://pmc.ncbi.nlm.nih.gov/articles/PMC9804695/) included children; a [Bionic Reading study](https://pmc.ncbi.nlm.nih.gov/articles/PMC12565662/) examined fluent adult English readers. Neither establishes a universal adult neurodivergent formatting prescription.
- Product defaults and input limits are engineering choices, not measured cognitive thresholds.

## Source provenance

The supplied research PDF's 18 pages were read, including its bibliography, and representative tables were visually checked. The original document is not redistributed. Its publication provenance and review status were not established. The repository research review records claims retained, qualified, and rejected; reading a bibliography is not independent validation of every cited item.

The [BDA guide](https://cdn.bdadyslexia.org.uk/uploads/documents/Advice/style-guide/BDA-Style-Guide-2023.pdf?v=1680084017), [NICE ADHD guidance](https://www.nice.org.uk/guidance/ng87/resources/attention-deficit-hyperactivity-disorder-diagnosis-and-management-pdf-1837699732933), and [ASAN Easy Read resource](https://autisticadvocacy.org/resources/accessibility/easyread/) were partly available through indexed primary-source text when direct retrieval was denied. Full-document verification of those sources remains open.

This is a bounded engineering synthesis, not a systematic literature review or evidence of clinical effectiveness. It rejects fixed support-level-to-capacity mappings, automatic numerical rounding, and blanket suppression of urgency.

## Technical and privacy boundaries

Deterministic formatting, extraction, and phrase matching cannot reliably rewrite every sentence, infer a correct plan, or detect all sarcasm and ambiguity. The client must review suggestions and preserve the original when meaning is uncertain. Prompt labels are not a guarantee against injection.

The server's resources are fixed bundled text. Resource reads do not fetch their linked sources or access arbitrary files. Formatting is local; input text and profiles are not persisted or logged by Diffyn. The upstream client may independently retain the conversation, so this is not an end-to-end privacy promise.

The host controls fonts, colors, spacing, semantic rendering, and assistive-technology integration. Client rendering tests, keyboard and screen-reader checks, and participatory evaluation with neurodivergent people remain necessary. Assess task success, retained meaning, effort, and preference fit; software tests alone cannot establish these outcomes.
