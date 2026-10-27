//! Exact surface-detail comparisons, never a semantic-equivalence certificate.
use super::{MAX_TEXT_BYTES, validate_text};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompareDetailsRequest {
    #[schemars(length(min = 1, max = 32768))]
    pub original_text: String,
    #[schemars(length(min = 1, max = 32768))]
    pub candidate_text: String,
    #[serde(default)]
    #[schemars(length(max = 64))]
    pub protected_text: Vec<String>,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct DetailSpan {
    pub start_byte: usize,
    pub end_byte: usize,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct DetailComparison {
    pub category: String,
    pub text: String,
    pub original_count: usize,
    pub candidate_count: usize,
    pub original_spans: Vec<DetailSpan>,
    pub candidate_spans: Vec<DetailSpan>,
    pub counts_match: bool,
    pub spans_truncated: bool,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct CompareDetailsOutput {
    pub details: Vec<DetailComparison>,
    pub total_detail_groups: usize,
    pub omitted_detail_groups: usize,
    pub changed_detail_groups: usize,
    pub review_required: bool,
    pub semantic_equivalence_checked: bool,
    pub coverage: Vec<String>,
    pub warnings: Vec<String>,
}
type Ledger = BTreeMap<(String, String), Vec<(usize, usize)>>;
fn add(map: &mut Ledger, category: &str, text: &str, start: usize, end: usize) {
    if let Some(value) = text.get(start..end) {
        map.entry((category.into(), value.into()))
            .or_default()
            .push((start, end));
    }
}
fn is_month(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "january"
            | "february"
            | "march"
            | "april"
            | "may"
            | "june"
            | "july"
            | "august"
            | "september"
            | "october"
            | "november"
            | "december"
            | "ionawr"
            | "chwefror"
            | "mawrth"
            | "ebrill"
            | "mai"
            | "mehefin"
            | "gorffennaf"
            | "awst"
            | "medi"
            | "hydref"
            | "tachwedd"
            | "rhagfyr"
    )
}
fn unit(word: &str) -> bool {
    matches!(
        word,
        "mm" | "cm"
            | "m"
            | "km"
            | "mg"
            | "g"
            | "kg"
            | "ml"
            | "mL"
            | "l"
            | "L"
            | "s"
            | "ms"
            | "min"
            | "h"
            | "Hz"
            | "kHz"
            | "MHz"
            | "W"
            | "kW"
            | "kWh"
            | "V"
            | "A"
            | "°C"
            | "°F"
            | "%"
            | "GBP"
            | "USD"
            | "EUR"
            | "percent"
            | "per cent"
    )
}
fn next_word(text: &str, mut i: usize) -> (usize, usize) {
    let b = text.as_bytes();
    while let Some(c) = text.get(i..).and_then(|s| s.chars().next()) {
        if !horizontal_space(c) {
            break;
        }
        i += c.len_utf8();
    }
    let start = i;
    while i < b.len() {
        let Some(c) = text[i..].chars().next() else {
            break;
        };
        if c.is_alphabetic() || c == '°' || c == '%' {
            i += c.len_utf8()
        } else {
            break;
        }
    }
    (start, i)
}
fn horizontal_space(c: char) -> bool {
    c.is_whitespace() && !matches!(c, '\r' | '\n' | '\u{2028}' | '\u{2029}')
}
fn numeric_prefix(text: &str, start: usize) -> Option<(usize, bool)> {
    let mut i = start;
    let sign_width = |at: usize| {
        text.get(at..)
            .and_then(|s| s.chars().next())
            .filter(|c| matches!(c, '+' | '-' | '−'))
            .map_or(0, char::len_utf8)
    };
    i += sign_width(i);
    let currency = text
        .get(i..)
        .and_then(|s| s.chars().next())
        .filter(|c| matches!(c, '£' | '$' | '€' | '¥'));
    if let Some(c) = currency {
        i += c.len_utf8();
        while let Some(c) = text.get(i..).and_then(|s| s.chars().next()) {
            if !horizontal_space(c) {
                break;
            }
            i += c.len_utf8();
        }
        if i > start && sign_width(start) == 0 {
            i += sign_width(i);
        }
    }
    let bytes = text.as_bytes();
    if bytes.get(i).is_some_and(u8::is_ascii_digit)
        || (bytes.get(i) == Some(&b'.') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
    {
        Some((i, currency.is_some()))
    } else {
        None
    }
}
fn ledger(text: &str, protected: &[String]) -> (Ledger, bool) {
    let mut map = Ledger::new();
    let b = text.as_bytes();
    let mut i = 0;
    let mut unmatched_code = false;
    while i < b.len() {
        if b[i] == b'`' || (b[i] == b'~' && b.get(i..i + 3) == Some(b"~~~")) {
            let start = i;
            let marker = b[i];
            while i < b.len() && b[i] == marker {
                i += 1;
            }
            let width = i - start;
            let mut end = None;
            let mut j = i;
            while j < b.len() {
                if b[j] == marker {
                    let run = j;
                    while j < b.len() && b[j] == marker {
                        j += 1;
                    }
                    if j - run == width {
                        end = Some(j);
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            let close = end.unwrap_or(b.len());
            unmatched_code |= end.is_none();
            add(&mut map, "code", text, start, close);
            i = close;
            continue;
        }
        if text
            .get(i..)
            .is_some_and(|s| s.starts_with("https://") || s.starts_with("http://"))
        {
            let start = i;
            while let Some(c) = text.get(i..).and_then(|s| s.chars().next()) {
                if c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'') {
                    break;
                }
                i += c.len_utf8();
            }
            // Preserve ambiguous terminal punctuation: !, ?, ), etc. can be legal
            // URL characters. A false-positive difference is safer than erasure.
            add(&mut map, "url", text, start, i);
            continue;
        }
        if let Some((digits_start, has_currency)) = numeric_prefix(text, i) {
            let start = i;
            i = digits_start;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            while i + 1 < b.len()
                && matches!(b[i], b'.' | b',' | b'/' | b'-' | b':')
                && b[i + 1].is_ascii_digit()
            {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let number_end = i;
            if b.get(i) == Some(&b'%') {
                i += 1;
            }
            let (word_start, word_end) = next_word(text, i);
            let word = &text[word_start..word_end];
            let mut category = if has_currency {
                "number_and_unit"
            } else {
                "numeric_value"
            };
            if unit(word) {
                i = word_end;
                category = "number_and_unit";
            } else if is_month(word) {
                i = word_end;
                category = "date";
                let mut end = i;
                while end < b.len() && b[end] == b' ' {
                    end += 1;
                }
                let year = end;
                while end < b.len() && b[end].is_ascii_digit() {
                    end += 1;
                }
                if end - year == 4 {
                    i = end;
                }
            } else if text[start..number_end].contains('/')
                || text[start..number_end]
                    .bytes()
                    .filter(|c| *c == b'-')
                    .count()
                    >= 2
            {
                category = "date_like";
            }
            add(&mut map, category, text, start, i);
            continue;
        }
        // Also recognize month-first dates without interpreting date order.
        if let Some(c) = text[i..].chars().next() {
            if c.is_alphabetic() {
                let start = i;
                while i < b.len() {
                    let Some(c) = text[i..].chars().next() else {
                        break;
                    };
                    if c.is_alphabetic() {
                        i += c.len_utf8()
                    } else {
                        break;
                    }
                }
                if is_month(&text[start..i]) {
                    let mut end = i;
                    while end < b.len() && b[end] == b' ' {
                        end += 1;
                    }
                    let day = end;
                    while end < b.len() && b[end].is_ascii_digit() {
                        end += 1;
                    }
                    if (1..=2).contains(&(end - day)) {
                        if b.get(end) == Some(&b',') {
                            end += 1;
                        }
                        while end < b.len() && b[end] == b' ' {
                            end += 1;
                        }
                        let year = end;
                        while end < b.len() && b[end].is_ascii_digit() {
                            end += 1;
                        }
                        if end - year == 4 {
                            add(&mut map, "date", text, start, end);
                            i = end;
                        }
                    }
                }
            } else {
                i += c.len_utf8();
            }
        } else {
            break;
        }
    }
    for value in protected.iter().collect::<BTreeSet<_>>() {
        for (start, _) in text.match_indices(value) {
            add(&mut map, "user_protected", text, start, start + value.len());
        }
    }
    (map, unmatched_code)
}
pub fn compare_protected_details(
    request: CompareDetailsRequest,
) -> Result<CompareDetailsOutput, String> {
    validate_text(&request.original_text)?;
    validate_text(&request.candidate_text)?;
    if request.protected_text.len() > 64
        || request
            .protected_text
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 1024)
        || request
            .protected_text
            .iter()
            .map(String::len)
            .sum::<usize>()
            > MAX_TEXT_BYTES
    {
        return Err("Protected text exceeds the published limits.".into());
    }
    let (mut original, original_warning) = ledger(&request.original_text, &request.protected_text);
    let (mut candidate, candidate_warning) =
        ledger(&request.candidate_text, &request.protected_text);
    // Include explicitly protected strings even when neither text contains them.
    for s in &request.protected_text {
        original
            .entry(("user_protected".into(), s.clone()))
            .or_default();
    }
    let mut keys: Vec<_> = original.keys().chain(candidate.keys()).cloned().collect();
    keys.sort();
    keys.dedup();
    let total_detail_groups = keys.len();
    let mut changed_detail_groups = 0;
    let mut details = Vec::new();
    for (category, text) in keys {
        let a = original
            .remove(&(category.clone(), text.clone()))
            .unwrap_or_default();
        let b = candidate
            .remove(&(category.clone(), text.clone()))
            .unwrap_or_default();
        let counts_match = a.len() == b.len();
        if !counts_match {
            changed_detail_groups += 1;
        }
        if details.len() < 128 {
            let spans_truncated = a.len() > 16 || b.len() > 16;
            details.push(DetailComparison {
                category,
                text,
                original_count: a.len(),
                candidate_count: b.len(),
                original_spans: a
                    .into_iter()
                    .take(16)
                    .map(|(start_byte, end_byte)| DetailSpan {
                        start_byte,
                        end_byte,
                    })
                    .collect(),
                candidate_spans: b
                    .into_iter()
                    .take(16)
                    .map(|(start_byte, end_byte)| DetailSpan {
                        start_byte,
                        end_byte,
                    })
                    .collect(),
                counts_match,
                spans_truncated,
            });
        }
    }
    let mut warnings = vec![
        "Exact occurrence counts do not verify semantic equivalence, negation, attribution, context or safety. Unchanged details can still be used incorrectly.".into(),
        "Detection covers a finite numeric/date/unit syntax, HTTP(S) URLs and backtick/tilde-delimited code; other representations need review. Spans are UTF-8 byte offsets.".into(),
    ];
    warnings.push(
        "URL candidates retain terminal punctuation because prose punctuation and legal URL characters can be ambiguous; inspect URL-boundary differences.".into(),
    );
    if original_warning || candidate_warning {
        warnings.push(
            "An unmatched code delimiter conservatively protected the remaining text as code."
                .into(),
        );
    }
    if total_detail_groups > details.len() {
        warnings.push("Only the first 128 detail groups are expanded; changed-group counts include all detected groups.".into());
    }
    if request
        .protected_text
        .iter()
        .any(|s| !request.original_text.contains(s))
    {
        warnings.push("Some user-protected text was absent from the original; no source occurrence can be verified for it.".into());
    }
    Ok(CompareDetailsOutput {
        omitted_detail_groups: total_detail_groups - details.len(),
        total_detail_groups,
        changed_detail_groups,
        details,
        review_required: true,
        semantic_equivalence_checked: false,
        coverage: vec![
            "exact numeric and recognized unit text".into(),
            "finite date forms without date interpretation".into(),
            "HTTP(S) URLs".into(),
            "delimited code".into(),
            "user-supplied protected text".into(),
        ],
        warnings,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_counts_not_semantics() -> Result<(), String> {
        let out = compare_protected_details(CompareDetailsRequest {
            original_text: "Do not take 10 mg on 27 October 2026. `a=1` https://example.org/x"
                .into(),
            candidate_text: "Take 20 mg on 27 October 2026. `a=2` https://example.org/y".into(),
            protected_text: vec!["Do not".into()],
        })?;
        assert!(out.changed_detail_groups >= 7);
        assert!(!out.semantic_equivalence_checked);
        assert!(
            out.details
                .iter()
                .any(|d| d.text == "27 October 2026" && d.counts_match)
        );
        Ok(())
    }
    #[test]
    fn utf8_spans_and_duplicate_counts() -> Result<(), String> {
        let a = "é 5 kg ☃ 5 kg";
        let out = compare_protected_details(CompareDetailsRequest {
            original_text: a.into(),
            candidate_text: "5 kg".into(),
            protected_text: vec![],
        })?;
        let detail = out
            .details
            .iter()
            .find(|d| d.text == "5 kg")
            .ok_or("missing")?;
        assert_eq!(detail.original_count, 2);
        for s in &detail.original_spans {
            assert_eq!(a.get(s.start_byte..s.end_byte), Some("5 kg"));
        }
        Ok(())
    }
    #[test]
    fn currency_and_sign_changes_are_protected() -> Result<(), String> {
        for (original, candidate) in [
            ("£48", "$48"),
            ("-£48", "£48"),
            ("£-48", "£48"),
            ("−€48.50", "€48.50"),
            ("£ 48", "$ 48"),
        ] {
            let output = compare_protected_details(CompareDetailsRequest {
                original_text: original.into(),
                candidate_text: candidate.into(),
                protected_text: vec![],
            })?;
            assert_eq!(output.changed_detail_groups, 2, "{original} vs {candidate}");
            assert!(
                output
                    .details
                    .iter()
                    .all(|d| d.category == "number_and_unit")
            );
        }
        Ok(())
    }
    #[test]
    fn repeated_protection_does_not_inflate_occurrences() -> Result<(), String> {
        let output = compare_protected_details(CompareDetailsRequest {
            original_text: "keep keep".into(),
            candidate_text: "keep".into(),
            protected_text: vec!["keep".into(), "keep".into()],
        })?;
        let detail = output
            .details
            .iter()
            .find(|d| d.category == "user_protected")
            .ok_or("missing")?;
        assert_eq!(detail.original_count, 2);
        assert_eq!(detail.candidate_count, 1);
        Ok(())
    }
    #[test]
    fn unchanged_numbers_never_establish_meaning() -> Result<(), String> {
        let output = compare_protected_details(CompareDetailsRequest {
            original_text: "Do not take 5 mg.".into(),
            candidate_text: "Take 5 mg.".into(),
            protected_text: vec![],
        })?;
        assert_eq!(output.changed_detail_groups, 0);
        assert!(!output.semantic_equivalence_checked);
        assert!(output.review_required);
        Ok(())
    }
    #[test]
    fn group_caps_retain_complete_change_counts() -> Result<(), String> {
        let output = compare_protected_details(CompareDetailsRequest {
            original_text: (0..200)
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            candidate_text: "none".into(),
            protected_text: vec![],
        })?;
        assert_eq!(output.total_detail_groups, 200);
        assert_eq!(output.changed_detail_groups, 200);
        assert_eq!(output.details.len(), 128);
        assert_eq!(output.omitted_detail_groups, 72);
        Ok(())
    }
    #[test]
    fn preserves_decimal_prefixes_unicode_spaces_and_url_punctuation() -> Result<(), String> {
        for (original, candidate) in [
            (".5 mg", "5 mg"),
            ("-.5 mg", ".5 mg"),
            ("£\u{a0}48", "$\u{a0}48"),
            ("5\u{a0}mg", "5\u{a0}kg"),
            ("https://example.org/?a=!", "https://example.org/?a="),
            ("https://example.org/?", "https://example.org/"),
        ] {
            let output = compare_protected_details(CompareDetailsRequest {
                original_text: original.into(),
                candidate_text: candidate.into(),
                protected_text: vec![],
            })?;
            assert_eq!(output.changed_detail_groups, 2, "{original} vs {candidate}");
        }
        Ok(())
    }
}
