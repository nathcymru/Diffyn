# Diffyn: research basis for neurodivergent communication support

**Purpose:** translate accessibility guidance into adjustable communication tools
that preserve meaning and respect individual preferences.

Diffyn supports people who want clearer wording, manageable task sequences, or
less visually dense text. A diagnosis is not required, and neurotype labels do
not determine a person's intelligence, reading ability, or preferred format.

## Scope and evidence

**Method:** this is a focused engineering synthesis of accessibility standards,
clinical and self-advocacy guidance, and selected original studies.
It is not a systematic review, clinical intervention study, or accessibility
conformance assessment.

**Interpretation:** recommendations for clinical or educational settings inform
design possibilities; they do not demonstrate that an LLM tool improves health,
comprehension, or executive function.

- **Accessibility standards** identify requirements for a complete interface; server output alone cannot establish conformance.
- **Practice and clinical guidance** inform adaptable communication options; context and individual needs remain decisive.
- **Original studies** describe measured samples and tasks; results do not establish universal preferences.
- **Engineering decisions** define tool behavior and safeguards; a product default is not a clinical threshold.

**References:** citation keys link to the machine-readable
[bibliography](bibliography.bib), which records source titles, authors,
publication details, and direct URLs.

## Findings and design implications

### Personal choice and cognitive accessibility

**User control:** W3C COGA recommends clear content, manageable steps, reduced
memory demands, and personalization that preserves meaning.
It also recommends involving people with cognitive and learning disabilities in
design and testing. [w3c-coga-2021](bibliography.bib)

**Status:** COGA is supplemental Working Group Note guidance, separate from
WCAG conformance requirements. WCAG 2.2 addresses the rendered interface and
complete processes; relevant requirements include structure, contrast, reflow,
keyboard access, and tolerance of user text-spacing overrides.
[w3c-coga-2021](bibliography.bib), [w3c-wcag22-2024](bibliography.bib)

**Diffyn implication:** offer reversible formatting choices and preserve the
information needed to act; leave font, spacing, contrast, and assistive-technology
behavior to the host interface and its evaluation.

### Autism and communication

**Individual needs:** NICE adult autism guidance emphasizes autonomy,
appropriate communication, and individual needs; its adaptations for
psychological interventions include concrete structure and reduced ambiguity.
Applying those ideas to LLM output is an engineering extrapolation.
[nice-cg142](bibliography.bib)

**Accessible formats:** the autistic-led Autistic Self Advocacy Network describes
Easy Read as a distinct format using accessible language and pictures.
Availability of this format does not justify assigning it to every autistic
person. [asan-one-idea-per-line](bibliography.bib)

**Diffyn implication:** offer explicit references and stable terminology while
retaining requested humour, warmth, technical depth, and creative expression.
Do not infer communication preferences from diagnostic support categories.

### Attention, planning, and coordination

**Task support:** NICE ADHD guidance describes individualized environmental
adjustments, including reduced distraction, shorter periods of focus, breaks,
and written reinforcement; it does not prescribe an LLM response length or a
universal number of steps. [nice-ng87](bibliography.bib)

**Sequencing:** NHS adult dyspraxia guidance describes movement and coordination
differences alongside possible planning and memory difficulties, with written
instructions and extra time among practical adjustments.
[nhs-dyspraxia-adults](bibliography.bib)

**Diffyn implication:** make the next useful action visible, retain prerequisites,
and offer smaller groups without deleting later steps; never infer physical
ability or invent a procedure from an underspecified goal.

### Reading and presentation

**Adaptable style:** the British Dyslexia Association recommends readable type,
spacing, and restrained emphasis; these are presentation options to adapt to the
reader, rather than guarantees attached to a diagnosis.
[bda-style-guide-2026](bibliography.bib)

**Specialist fonts:** Joseph and Powell studied 71 children aged 8–12, including
37 with dyslexia, and found no word- or passage-reading advantage for Dyslexie
over Calibri when spacing was controlled, although letter naming differed.
This does not establish an optimal adult font.
[joseph-powell-2022](bibliography.bib)

**Bionic Reading:** Beelders studied 53 fluent adult English readers and found
no significant improvements in reading speed, fixation duration, or fixation
count; the study was not a clinical ADHD intervention trial.
Individual preference and demonstrated general benefit remain different claims.
[beelders-2025](bibliography.bib)

**Diffyn implication:** offer short sections and optional restrained emphasis,
without automatically introducing specialized fonts, fragmented word bolding,
or a fixed reading grade.

## Lived experience, agency and AI communication

The 2.0.0 synthesis adds primary human-computer interaction studies to the earlier accessibility and practice sources. The studies below inform design questions; none evaluates Diffyn itself.

| Study | Evidence and sample | Implication and limit |
| --- | --- | --- |
| Jang et al., CHI 2024 | Eleven autistic participants compared a GPT-4 chatbot with a disguised human confederate for workplace advice; an employment practitioner identified problematic advice. [jang-et-al-2024](bibliography.bib) | Support user choice while reviewing suggestions that prioritise employers, masking or disclosure; expressed preference does not establish advice quality. |
| Çarik et al., January 2025 | Qualitative analysis of discussions across 61 Reddit communities identified uses, configuration workarounds and concerns. [carik-et-al-2025](bibliography.bib) | Make focused workflows discoverable and accept imperfect input; online self-reports do not establish prevalence or effectiveness. |
| Jamshed et al., ASSETS 2025 | Interviews with 19 neurodivergent students described useful assistance alongside configuration burden, disrupted routines and flattened personal voice. [jamshed-et-al-2025](bibliography.bib) | Preserve authorship and offer flexible task re-entry; the study is largely undergraduate, at one institution, and based on accounts rather than observed intervention outcomes. |
| Pinto et al., November 2025 | A seven-day diary study and interviews with 13 adults with ADHD examined communication and task support. [pinto-et-al-2025](bibliography.bib) | Offer manageable starting points and user-selected engagement; reported experiences do not establish treatment or body-doubling efficacy. |
| Wohn et al., CHI 2026 | An audit of 345,000 outputs from six LLMs, contextualised by interviews with 11 autistic adults, found that autism disclosure could shift advice toward stereotyped avoidance. [wohn-et-al-2026](bibliography.bib) | Use concrete preferences rather than diagnosis-based defaults and preserve ambitions and agency; the tested models and scenarios do not establish all-model behaviour or prove that instructions eliminate bias. |
| Kan et al., May 2026 | Participatory group discussions with 20 autistic adults in Singapore identified ambiguous work processes and shared responsibility for inclusion. [kan-et-al-2026](bibliography.bib) | Support reciprocal clarification and concrete adjustment requests; findings are context-specific and this paper does not report chatbot outcome evidence. |

### Communication repair and authorship

**Design response:** a message-understanding workflow distinguishes stated content, possible interpretations and unknowns. A repair workflow drafts a concrete clarification without claiming to know another person's private intentions. Voice-preserving editing retains refusals, uncertainty, commitments and chosen tone. These are engineering responses to the studies above, not validated measures of empathy or social competence.

**Agency:** advice must not assume that safety means withdrawal, that politeness means agreement, or that a diagnosis establishes risk tolerance. Diagnostic disclosure is never inserted into a draft automatically. Preferences can vary across tasks and over time.

### Starting and returning to tasks

**Design response:** clarify the goal, known prerequisites and completion criteria, then offer the requested level of support. A resume card records caller-supplied progress and a recorded next action. It does not infer completion or retain a hidden history. Structural plan checks can identify inconsistent references or dependencies, but cannot establish real-world safety or completeness.

### Numbers, dates and alternative explanations

**Additional COGA patterns:** the existing 2021 guidance specifically addresses implied content, task preparation and alternatives for numerical concepts. These are newly examined portions of an existing source, not new empirical findings. [w3c-coga-2021](bibliography.bib)

**Design response:** retain exact values and show supported calculations with units and labelled rounding. Supplement an exact number with an explanation rather than replacing it. Relative dates need a reference date, and ambiguous calendar conventions remain unresolved until the relevant context is known. Numerical explanation is not a diagnosis-specific rounding algorithm.

## Combined needs and uncertainty

**Compose preferences:** support needs can overlap, and useful adjustments can
conflict; the combinations below are engineering options, not validated clinical
prescriptions. Personalization provides the design basis.
[w3c-coga-2021](bibliography.bib)

- **Attention and reading:** use short groups with sparse optional emphasis, since too many visual anchors can increase clutter.
- **Predictability and task initiation:** keep a stable structure with one next action, since excessive detail can obscure the starting point.
- **Reading and motor sequencing:** use linear instructions with named controls, with fewer microsteps for familiar tasks when preferred.
- **Literal and low-pressure language:** offer explicit options in neutral wording while retaining requirements and urgency.
- **Numeracy:** retain exact values alongside optional explanations, without substituting approximate quantities.

**Diagnostic uncertainty:** the National Autistic Society describes disagreement
and evidence gaps around PDA, while the CDS work-group report identifies
unresolved classification questions; neither establishes a validated
condition-specific LLM formatting algorithm.
[nas-demand-avoidance](bibliography.bib), [becker-et-al-2023](bibliography.bib)

**Boundary:** support a requested tone, pace, or format without diagnosing a
condition, estimating capacity, or claiming to bypass a neurological response.

## Implementation principles

**Design decisions:** the following safeguards define Diffyn's intended scope;
they are engineering commitments rather than measured clinical effects.

- **Preserve meaning:** retain exact quantities, units, dates, links, code, quotations, negation, conditions, and safety information.
- **Respect choice:** apply explicit preferences without hidden diagnosis-based presets or mandatory health disclosure.
- **Separate operations:** distinguish layout changes, extracted actions, model-generated suggestions, and unresolved questions.
- **Expose uncertainty:** keep originals available and require review when a heuristic cannot establish meaning or completeness.
- **Limit collection:** process submitted text locally without application-level content logging or persistent profiles.
- **Respect host boundaries:** treat supplied text as data while leaving instruction authority and client retention policies with the host.

**Current behavior:** the [communication contract](communication-contract.md),
[architecture](architecture.md), and [engineering guide](engineering.md)
describe the implemented fields, mechanisms, and limitations.
The evidence reviewed here does not establish a universal sentence limit,
checklist size, readability score, or cognitive-load threshold.

## Evidence-to-feature traceability

The bundled [evidence-to-feature map](guidance/evidence-to-feature-map.md) links each research family to the corresponding workflow and its boundary. Local preservation checks, source references, dependency validation and output-shape checks are engineering controls; their correctness is assessed by software tests, not inferred from the cited studies.

## Evaluation priorities

**Software verification:** automated checks can assess preservation, argument
validation, resource routing, and protocol behavior; they cannot establish
comprehension or acceptance by intended users.

**Participatory evaluation:** involve consenting neurodivergent people with
varied and combined preferences, without requiring diagnostic disclosure,
following COGA's emphasis on user involvement.
[w3c-coga-2021](bibliography.bib)

- **Task outcomes:** assess initiation, resumption after interruption, mistakes and retained meaning against the original information.
- **Agency and authorship:** assess whether people recognise their own voice, retain boundaries and can reject unwanted assistance.
- **Configuration effort:** assess the work needed to discover and adjust a workflow, including declined or unsuccessful uses.
- **Uncertainty:** assess whether interpretations, unknown requirements and stale progress remain understandable.
- **User experience:** measure perceived effort, control, preferred detail, and willingness to use the format again.
- **Language coverage:** evaluate Welsh, English, and other intended languages without assuming English heuristics transfer.
- **Host accessibility:** test keyboard use, screen readers, zoom, reflow, contrast, and text-spacing overrides in real clients.
- **Failure review:** include ambiguity, mixed formatting, conditional steps, safety instructions, and conflicting preferences.

**Evidence boundary:** Diffyn has not established clinical effectiveness,
comprehensive accessibility conformance, or general improvements in reading or
executive function.

**Citation:** use the preferred research-report citation in
[CITATION.cff](../CITATION.cff); source metadata is available in
[bibliography.bib](bibliography.bib).
