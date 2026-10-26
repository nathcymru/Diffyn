# Diffyn

![Diffyn Header](header.png)

[![GitHub License](https://img.shields.io/github/license/nathcymru/Diffyn)](LICENSE)
[![Rust Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fnathcymru%2FDiffyn%2Fmain%2FCargo.toml&query=%24.package.rust-version&label=Rust&logo=rust)](https://rust-lang.org)
[![Protocol](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fnathcymru%2FDiffyn%2Fmain%2FCargo.toml&query=%24.package.metadata.diffyn.protocol&label=Protocol)](https://modelcontextprotocol.io)
[![Community](https://img.shields.io/github/contributors/nathcymru/Diffyn)](https://github.com/nathcymru/Diffyn/issues)

Diffyn is an open-source library of specialized system prompts and instructions designed to align Large Language Models (LLMs) with neurodivergent communication styles, processing needs, and executive functions.

The name bridges the English "Different" and the Welsh "Diffinio" (to define), reflecting our mission: redefining how AI communicates to match the unique way your mind works.

## 🧠 Why Diffyn?
Standard LLM outputs are frequently poorly formatted for neurodivergent individuals. They often feature overwhelming walls of text, ambiguous phrasing, or structures that aggravate executive dysfunction, reading fatigue, and sensory overload.

Diffyn optimizes LLM behaviors to deliver clear, structured, and friction-free interactions tailored to:
- **Autism Spectrum Conditions (ASC):** Eliminates idioms, minimizes conversational fluff, provides hyper-direct answers first, and respects literal interpretations.
- **ADHD & AuDHD:** Uses scannable formatting, punchy summaries, bold visual anchors, and actionable, step-by-step task breakdowns to reduce executive overwhelm.
- **Dyslexia:** Formats text with high-readability layouts, avoids dense paragraphs, and prioritizes clear structural markers.
- **Dyspraxia & Processing Differences:** Structures information sequentially to alleviate cognitive load during complex navigation or instructions.

## 🏴󠁧󠁢󠁷󠁬󠁳󠁿 Cyflwyniad (Welsh Introduction)
Llyfrgell cod agored yw Diffyn sy'n cynnwys cyfarwyddiadau a phrompts system ar gyfer Aliniad Modelau Iaith Mawr (LLMs) ag anghenion cyfathrebu a phrosesu niwrodargyfeiriol.

Daw'r enw o gyfuniad o'r gair Saesneg "Different" a'r gair Cymraeg "Diffinio". Ein nod yw ailddiffinio sut mae deallusrwydd artiffisial yn cyfathrebu, gan sicrhau bod yr allbwn yn cyd-fynd yn berffaith â ffordd unigryw eich meddwl chi o weithio.

Mae Diffyn yn teilwrio ymatebion AI ar gyfer Awtistiaeth, ADHD, AuDHD, Dyslexia, a Dyspracsia drwy gael gwared ar eiriau diangen, strwythuro gwybodaeth yn glir, a chynnig camau gweithredu syml sy'n helpu gyda gorlwytho gwybyddol ac arafwch gweithredol.

---

## Project status

Diffyn's first implementation is a local Rust MCP server with three text tools,
two prompt templates, and four bundled reference resources. The descriptions
above state the project's goals; they are not claims of clinically validated outcomes.
Communication preferences vary within and across neurotypes. Individual choices
take priority over any default, and a diagnosis is never required.

The tools format layout, extract explicit steps, and flag possible idioms. They
preserve source text and report uncertainty; they do not diagnose people or
guarantee a correct plan or semantic rewrite. Processing stays local to the
server, while the upstream AI client's data policies still apply.

Get started with [the engineering guide](docs/engineering.md) and
[example requests](docs/examples.md). Read the [research assessment](docs/research.md)
for evidence, source limitations, and the next evaluation needs.
The code and original project documentation use the [MIT license](LICENSE).

## Optional charity donations

Diffyn is developed voluntarily and remains free to use and fork. If you would
like to support a neurodiversity charity, see the [independent charity donation
page](.github/DONATE.md). Donations are optional, go directly to your chosen
charity, and do not imply any affiliation with Diffyn.

