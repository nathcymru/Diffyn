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

## Evaluation priorities

**Software verification:** automated checks can assess preservation, argument
validation, resource routing, and protocol behavior; they cannot establish
comprehension or acceptance by intended users.

**Participatory evaluation:** involve consenting neurodivergent people with
varied and combined preferences, without requiring diagnostic disclosure,
following COGA's emphasis on user involvement.
[w3c-coga-2021](bibliography.bib)

- **Task outcomes:** assess completion, mistakes, and retained meaning against the original information.
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
