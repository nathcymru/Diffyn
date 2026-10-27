//! Conservative whitespace-only chunking, with verbatim fallbacks.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    NeurotypeProfile, ResolvedPreferences, resolve_profile,
    text_spans::{ProtectedSpan, overlaps, protected_spans},
    validate_text,
};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReformatRequest {
    pub text: String,
    #[serde(default)]
    pub profile: Option<NeurotypeProfile>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ReformatOutput {
    pub original_text: String,
    pub formatted_text: String,
    /// Protected source bytes are copied exactly; offsets refer to original_text.
    pub protected_spans: Vec<ProtectedSpan>,
    pub preferences: ResolvedPreferences,
    pub changes: Vec<String>,
    pub warnings: Vec<String>,
    pub review_required: bool,
    pub limitations: Vec<String>,
}

pub fn reformat_for_cognitive_load(request: ReformatRequest) -> Result<ReformatOutput, String> {
    validate_text(&request.text)?;
    let preferences = resolve_profile(request.profile.as_ref())?;
    let mut warnings = Vec::new();
    let mut changes = Vec::new();
    let mut breaks = 0;
    let protected = protected_spans(&request.text);
    let mut formatted_text = String::with_capacity(request.text.len());
    let mut line_offset = 0;
    let paragraph_break = if request.text.contains("\r\n") {
        "\r\n\r\n"
    } else {
        "\n\n"
    };
    for line in request.text.split_inclusive('\n') {
        if is_structural_line(line) {
            formatted_text.push_str(line);
            line_offset += line.len();
            continue;
        }
        let mut previous_end = 0;
        let mut eligible_count = 0;
        for (start, end) in sentence_separators(line) {
            // Include the sentence punctuation: a URL/code/quote ending in a
            // period must not be mistaken for a prose sentence boundary.
            if overlaps(&protected, line_offset + start - 1, line_offset + end) {
                continue;
            }
            eligible_count += 1;
            if eligible_count % preferences.max_sentences_per_chunk == 0 {
                formatted_text.push_str(&line[previous_end..start]);
                formatted_text.push_str(paragraph_break);
                previous_end = end;
                breaks += 1;
            }
        }
        formatted_text.push_str(&line[previous_end..]);
        line_offset += line.len();
    }
    if !protected.is_empty() {
        warnings.push("Recognized source syntax was preserved byte-for-byte; eligible surrounding prose may have layout changes. Unclosed constructs are protected conservatively.".into());
    }
    if breaks > 0 {
        changes.push(format!("Replaced {breaks} inter-sentence whitespace gap(s) with paragraph breaks; all non-whitespace characters were preserved."));
    }
    if preferences.use_bold {
        warnings.push(
            "Bold emphasis was not added: selecting important words requires semantic review."
                .into(),
        );
    }
    Ok(ReformatOutput {
        original_text: request.text,
        formatted_text,
        protected_spans: protected,
        preferences,
        changes,
        review_required: breaks > 0 || !warnings.is_empty(),
        warnings,
        limitations: vec![
            "Layout only: this tool does not summarize, simplify vocabulary, verify understanding, or measure cognitive load.".into(),
            "Sentence boundaries are conservative heuristics; protected structures remain unchanged. Review presentation in the receiving client.".into(),
            "Plain-text preference suppresses generated decoration; existing source formatting is never stripped.".into(),
        ],
    })
}

fn is_structural_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with(['#', '>', '-', '*', '+'])
        || line.split_once(['.', ')']).is_some_and(|(prefix, tail)| {
            !prefix.is_empty()
                && prefix.chars().all(|ch| ch.is_ascii_digit())
                && tail.starts_with(char::is_whitespace)
        })
}

/// Return UTF-8 byte ranges of whitespace after likely sentence endings.
/// Ranges are boundaries from char_indices; slicing never crosses a code point.
pub(super) fn sentence_separators(text: &str) -> Vec<(usize, usize)> {
    let mut separators = Vec::new();
    for (index, ch) in text.char_indices() {
        if !matches!(ch, '.' | '!' | '?') {
            continue;
        }
        let start = index + ch.len_utf8();
        let mut end = start;
        for (offset, next) in text[start..].char_indices() {
            if next == ' ' || next == '\t' {
                end = start + offset + next.len_utf8();
            } else {
                break;
            }
        }
        // A following uppercase letter makes this deliberately conservative for
        // abbreviations, decimals, numbered data and unsupported writing systems.
        if end > start
            && text[end..].chars().next().is_some_and(char::is_uppercase)
            && !(ch == '.' && is_abbreviation(&text[..index]))
        {
            separators.push((start, end));
        }
    }
    separators
}

fn is_abbreviation(before_period: &str) -> bool {
    let token = before_period.split_whitespace().next_back().unwrap_or("");
    let token = token.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '.');
    if token.contains('.') {
        // A decimal followed by sentence punctuation is a complete value,
        // whereas a dotted abbreviation needs conservative preservation.
        return token.parse::<f64>().is_err();
    }
    if token.chars().count() == 1 && token.chars().all(char::is_alphabetic) {
        return true;
    }
    let lower = token.to_ascii_lowercase();
    [
        "mr", "mrs", "ms", "mx", "dr", "prof", "sr", "jr", "st", "vs", "etc", "eg", "ie", "fig",
        "no", "approx", "dept", "inc", "ltd", "rev", "vol", "jan", "feb", "mar", "apr", "jun",
        "jul", "aug", "sep", "sept", "oct", "nov", "dec", "e.e", "h.y", "ayb", "cyf", "tud", "gol",
        "llaw", "rhif", "mg", "ml", "cm", "mm",
    ]
    .contains(&lower.as_str())
}
