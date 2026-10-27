//! A small English idiom linter, not a claim to understand intent or sarcasm.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    NeurotypeProfile, ResolvedPreferences, resolve_profile,
    text_spans::{ProtectedSpan, overlaps, protected_spans, unprotected_ranges},
    validate_text,
};

const MAX_FINDINGS: usize = 128;

// Interpretations are conditional suggestions, never automatic replacements.
// A match can be literal, quoted, technical, creative or entirely appropriate.
const IDIOMS: [(&str, &str); 9] = [
    (
        "break a leg",
        "If this is an expression of encouragement, consider: Good luck.",
    ),
    (
        "piece of cake",
        "If this means low difficulty, describe which part is easy and why; difficulty varies by person.",
    ),
    (
        "hit the ground running",
        "If this means starting promptly, state the first action and any actual deadline.",
    ),
    (
        "touch base",
        "If this means making contact, name who should contact whom and the purpose.",
    ),
    (
        "on the same page",
        "If this means agreement, name the particular decision or understanding to confirm.",
    ),
    (
        "give me a hand",
        "If this is a request for assistance, name the specific action or help needed.",
    ),
    (
        "under the weather",
        "If this refers to feeling unwell, say that directly without inferring a diagnosis.",
    ),
    (
        "ball is in your court",
        "If this means another person decides or acts next, state the decision or action explicitly.",
    ),
    (
        "at the end of the day",
        "If this introduces a conclusion rather than a time, state the conclusion directly.",
    ),
];

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClarityCategory {
    EnglishIdioms,
    VagueTime,
    VagueReference,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClarityRequest {
    pub text: String,
    #[serde(default)]
    pub profile: Option<NeurotypeProfile>,
    /// Omitted means english_idioms only. An empty list disables all rules.
    #[serde(default)]
    #[schemars(length(max = 3))]
    pub categories: Option<Vec<ClarityCategory>>,
    /// Explicit language tag; defaults to en. No language is inferred from text.
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ClarityFinding {
    /// UTF-8 byte offsets in original_text. These are not character indices.
    pub start_byte: usize,
    pub end_byte: usize,
    pub phrase: String,
    pub category: String,
    pub rule_id: String,
    pub suggested_interpretation: Option<String>,
    pub note: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ClarityOutput {
    pub original_text: String,
    pub analyzed: bool,
    pub findings: Vec<ClarityFinding>,
    /// Total matches in the portions scanned, including those beyond the output cap.
    pub total_findings: usize,
    pub truncated_findings: bool,
    pub additional_findings_count: usize,
    pub skipped_protected_lines: usize,
    pub protected_spans: Vec<ProtectedSpan>,
    pub analyzed_bytes: usize,
    pub skipped_protected_bytes: usize,
    pub analysis_language: String,
    pub language_supported: bool,
    pub applied_categories: Vec<ClarityCategory>,
    pub preferences: ResolvedPreferences,
    pub review_required: bool,
    pub warnings: Vec<String>,
    pub limitations: Vec<String>,
}

pub fn verify_clarity_and_literalism(request: ClarityRequest) -> Result<ClarityOutput, String> {
    validate_text(&request.text)?;
    let preferences = resolve_profile(request.profile.as_ref())?;
    let categories = request
        .categories
        .unwrap_or_else(|| vec![ClarityCategory::EnglishIdioms]);
    if categories.len() > 3
        || categories
            .iter()
            .enumerate()
            .any(|(index, category)| categories[..index].contains(category))
    {
        return Err("Choose each supported clarity category at most once.".into());
    }
    let language = request.language.unwrap_or_else(|| "en".into());
    if language.is_empty()
        || language.len() > 35
        || language
            .split('-')
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_alphanumeric()))
    {
        return Err(
            "Provide a language tag of at most 35 ASCII letters, digits and hyphens.".into(),
        );
    }
    let language_supported = language
        .split('-')
        .next()
        .is_some_and(|part| part.eq_ignore_ascii_case("en"));
    let protected = protected_spans(&request.text);
    let ranges = unprotected_ranges(&request.text, &protected);
    let mut candidates = Vec::new();
    let mut warnings = Vec::new();
    let mut patterns: Vec<(&str, &str, &str)> = Vec::new();
    if categories.contains(&ClarityCategory::EnglishIdioms) {
        patterns.extend(
            IDIOMS.iter().map(|(phrase, interpretation)| {
                (*phrase, *interpretation, "possible_english_idiom")
            }),
        );
    }
    if categories.contains(&ClarityCategory::VagueTime) {
        patterns.extend(["soon", "later", "asap", "in a bit", "at some point"].into_iter().map(|phrase| (
            phrase, "If action depends on a time, ask for the actual date, time and timezone; do not invent a deadline.", "possible_vague_time"
        )));
    }
    if categories.contains(&ClarityCategory::VagueReference) {
        patterns.extend(["over there", "the usual thing", "do that", "sort it out", "as before"].into_iter().map(|phrase| (
            phrase, "If the reference is unclear in this context, name the intended object, person or action after confirming it.", "possible_vague_reference"
        )));
    }
    let enabled = language_supported && !patterns.is_empty();
    let analyzed_bytes = if enabled {
        ranges.iter().map(|(start, end)| end - start).sum()
    } else {
        0
    };
    let analyzed = enabled
        && ranges
            .iter()
            .any(|(start, end)| !request.text[*start..*end].trim().is_empty());
    if enabled {
        for (range_start, range_end) in &ranges {
            let source = &request.text[*range_start..*range_end];
            // ASCII folding keeps original Unicode byte offsets intact.
            let folded = source.to_ascii_lowercase();
            for (pattern_index, (phrase, _, _)) in patterns.iter().enumerate() {
                for (offset, _) in folded.match_indices(phrase) {
                    let start = range_start + offset;
                    let end = start + phrase.len();
                    if whole_phrase(&request.text, start, end) {
                        candidates.push((start, end, pattern_index));
                    }
                }
            }
        }
    }
    if !language_supported {
        warnings.push("No rules were applied: this tool currently contains English phrase rules only. Source preservation does not establish language coverage.".into());
    } else if patterns.is_empty() {
        warnings.push("No rules were applied because the category list is empty.".into());
    }
    let mut line_start = 0;
    let skipped_protected_lines = request
        .text
        .split_inclusive('\n')
        .filter(|line| {
            let skipped = overlaps(&protected, line_start, line_start + line.len());
            line_start += line.len();
            skipped
        })
        .count();
    let skipped_protected_bytes = protected
        .iter()
        .map(|span| span.end_byte - span.start_byte)
        .sum();
    if !protected.is_empty() {
        warnings.push("Recognized protected spans were not analyzed; eligible surrounding prose was considered separately. Unclosed constructs are protected conservatively.".into());
    }
    candidates.sort_unstable();
    let total_findings = candidates.len();
    candidates.truncate(MAX_FINDINGS);
    let additional_findings_count = total_findings.saturating_sub(candidates.len());
    if additional_findings_count > 0 {
        warnings.push("Only the first 128 findings are expanded; total_findings and additional_findings_count report the remaining matches.".into());
    }
    let findings = candidates.into_iter().map(|(start, end, pattern_index)| {
        let (phrase, interpretation, category) = patterns[pattern_index];
        ClarityFinding {
            start_byte: start,
            end_byte: end,
            phrase: request.text[start..end].to_owned(),
            category: category.into(),
            rule_id: format!("{category}.{}", phrase.replace(' ', "_")),
            suggested_interpretation: preferences.literal_language.then(|| interpretation.to_owned()),
            note: "This phrase may be literal, technical, creative, or appropriate in context. Confirm intended meaning before making any change.".into(),
        }
    }).collect();
    Ok(ClarityOutput {
        original_text: request.text,
        analyzed,
        findings,
        total_findings,
        truncated_findings: additional_findings_count > 0,
        additional_findings_count,
        skipped_protected_lines,
        protected_spans: protected,
        analyzed_bytes,
        skipped_protected_bytes,
        analysis_language: language,
        language_supported,
        applied_categories: if enabled { categories } else { Vec::new() },
        preferences,
        review_required: true,
        warnings,
        limitations: vec![
            "Finite English phrase matching only. This does not detect all ambiguity, sarcasm, figurative language, or implied instructions, and it does not verify comprehension.".into(),
            "The original text is unchanged. Suggested interpretations are conditional and require human or host review; a literal occurrence is not an error.".into(),
            "No matches is not a pass, accessibility certification, safety check, or evidence that language is clear to a particular person.".into(),
            "Byte counts describe scanned source, not semantic understanding; protected spans and unsupported languages remain unassessed.".into(),
        ],
    })
}

fn whole_phrase(text: &str, start: usize, end: usize) -> bool {
    let is_word = |ch: char| ch.is_alphanumeric() || ch == '_';
    !text[..start].chars().next_back().is_some_and(is_word)
        && !text[end..].chars().next().is_some_and(is_word)
}
