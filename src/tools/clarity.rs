//! A small English idiom linter, not a claim to understand intent or sarcasm.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    NeurotypeProfile, ResolvedPreferences,
    layout::{has_structured_block, has_verbatim_syntax},
    resolve_profile, validate_text,
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

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClarityRequest {
    pub text: String,
    #[serde(default)]
    pub profile: Option<NeurotypeProfile>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ClarityFinding {
    /// UTF-8 byte offsets in original_text. These are not character indices.
    pub start_byte: usize,
    pub end_byte: usize,
    pub phrase: String,
    pub category: String,
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
    pub preferences: ResolvedPreferences,
    pub review_required: bool,
    pub warnings: Vec<String>,
    pub limitations: Vec<String>,
}

pub fn verify_clarity_and_literalism(request: ClarityRequest) -> Result<ClarityOutput, String> {
    validate_text(&request.text)?;
    let preferences = resolve_profile(request.profile.as_ref())?;
    let mut candidates = Vec::new();
    let mut skipped_protected_lines = 0;
    let mut warnings = Vec::new();
    let analyzed = !has_structured_block(&request.text);
    if analyzed {
        let mut line_offset = 0;
        for line in request.text.split_inclusive('\n') {
            if has_verbatim_syntax(line) {
                skipped_protected_lines += 1;
            } else {
                // ASCII-only case folding preserves byte positions in Unicode
                // text. Other languages are never translated or normalized.
                let folded = line.to_ascii_lowercase();
                for (pattern_index, (phrase, _)) in IDIOMS.iter().enumerate() {
                    for (start, _) in folded.match_indices(phrase) {
                        let end = start + phrase.len();
                        if whole_phrase(line, start, end) {
                            candidates.push((
                                line_offset + start,
                                line_offset + end,
                                pattern_index,
                            ));
                        }
                    }
                }
            }
            line_offset += line.len();
        }
    } else {
        warnings.push("Structured or verbatim blocks were retained without idiom analysis.".into());
    }
    if skipped_protected_lines > 0 {
        warnings.push("Quoted, code, mathematical, or linked lines were skipped to avoid misleading suggestions.".into());
    }
    candidates.sort_unstable();
    let total_findings = candidates.len();
    candidates.truncate(MAX_FINDINGS);
    let additional_findings_count = total_findings.saturating_sub(candidates.len());
    if additional_findings_count > 0 {
        warnings.push("Only the first 128 findings are expanded; total_findings and additional_findings_count report the remaining matches.".into());
    }
    let findings = candidates
        .into_iter()
        .map(|(start, end, pattern_index)| ClarityFinding {
            start_byte: start,
            end_byte: end,
            phrase: request.text[start..end].to_owned(),
            category: "possible_english_idiom".into(),
            suggested_interpretation: preferences.literal_language.then(|| IDIOMS[pattern_index].1.to_owned()),
            note: "This phrase may be literal, technical, creative, or appropriate in context. Confirm intended meaning before making any change.".into(),
        })
        .collect();
    Ok(ClarityOutput {
        original_text: request.text,
        analyzed,
        findings,
        total_findings,
        truncated_findings: additional_findings_count > 0,
        additional_findings_count,
        skipped_protected_lines,
        preferences,
        review_required: true,
        warnings,
        limitations: vec![
            "Finite English phrase matching only. This does not detect all ambiguity, sarcasm, figurative language, or implied instructions, and it does not verify comprehension.".into(),
            "The original text is unchanged. Suggested interpretations are conditional and require human or host review; a literal occurrence is not an error.".into(),
            "No matches is not a pass, accessibility certification, safety check, or evidence that language is clear to a particular person.".into(),
        ],
    })
}

fn whole_phrase(text: &str, start: usize, end: usize) -> bool {
    let is_word = |ch: char| ch.is_alphanumeric() || ch == '_';
    !text[..start].chars().next_back().is_some_and(is_word)
        && !text[end..].chars().next().is_some_and(is_word)
}
