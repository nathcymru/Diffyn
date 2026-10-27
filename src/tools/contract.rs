//! Measurable presentation checks, with explicit unassessed areas.
use super::{MAX_TEXT_BYTES, validate_text};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ResponsePreferences {
    #[schemars(range(min = 20, max = 10000))]
    pub max_paragraph_chars: Option<usize>,
    #[schemars(range(min = 3, max = 200))]
    pub max_sentence_words: Option<usize>,
    #[schemars(length(max = 20))]
    pub required_headings: Vec<String>,
    #[schemars(length(max = 50))]
    pub forbidden_terms: Vec<String>,
    pub plain_text: bool,
    /// Checks for a caller-specified literal marker only; never infers an actionable next step.
    #[schemars(length(min = 1, max = 128))]
    pub next_action_marker: Option<String>,
}
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResponseContractRequest {
    #[schemars(length(min = 1, max = 32768))]
    pub text: String,
    pub preferences: ResponsePreferences,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ContractFinding {
    pub category: String,
    pub message: String,
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
    pub observed: Option<usize>,
    pub requested_max: Option<usize>,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ResponseContractOutput {
    pub findings: Vec<ContractFinding>,
    pub total_findings: usize,
    pub omitted_findings: usize,
    pub checked: Vec<String>,
    pub unassessed: Vec<String>,
    pub review_required: bool,
}
fn push(
    findings: &mut Vec<ContractFinding>,
    total: &mut usize,
    category: &str,
    message: &str,
    span: Option<(usize, usize)>,
    observed: Option<usize>,
    max: Option<usize>,
) {
    *total += 1;
    if findings.len() < 128 {
        findings.push(ContractFinding {
            category: category.into(),
            message: message.into(),
            start_byte: span.map(|s| s.0),
            end_byte: span.map(|s| s.1),
            observed,
            requested_max: max,
        });
    }
}
pub fn check_response_contract(
    request: ResponseContractRequest,
) -> Result<ResponseContractOutput, String> {
    validate_text(&request.text)?;
    let p = &request.preferences;
    if p.max_paragraph_chars
        .is_some_and(|n| !(20..=10000).contains(&n))
        || p.max_sentence_words
            .is_some_and(|n| !(3..=200).contains(&n))
        || p.required_headings.len() > 20
        || p.forbidden_terms.len() > 50
        || p.required_headings
            .iter()
            .chain(&p.forbidden_terms)
            .any(|s| s.trim().is_empty() || s.len() > 128 || s.contains(['\n', '\r']))
        || p.next_action_marker
            .as_ref()
            .is_some_and(|s| s.trim().is_empty() || s.len() > 128 || s.contains(['\n', '\r']))
        || p.required_headings
            .iter()
            .chain(&p.forbidden_terms)
            .map(String::len)
            .sum::<usize>()
            + request.text.len()
            + p.next_action_marker.as_ref().map_or(0, String::len)
            > MAX_TEXT_BYTES
    {
        return Err("Response contract preferences exceed the published limits.".into());
    }
    let mut findings = Vec::new();
    let mut total = 0;
    let mut checked = Vec::new();
    let text = &request.text;
    if let Some(max) = p.max_paragraph_chars {
        checked.push("Unicode scalar count per blank-line-delimited paragraph".into());
        let mut start = 0;
        let mut end = 0;
        for line in text.split_inclusive('\n') {
            if line.trim().is_empty() {
                if end > start {
                    let count = text[start..end].trim().chars().count();
                    if count > max {
                        push(
                            &mut findings,
                            &mut total,
                            "paragraph_length",
                            "A paragraph exceeds the requested character count.",
                            Some((start, end)),
                            Some(count),
                            Some(max),
                        );
                    }
                }
                start = end + line.len();
            }
            end += line.len();
        }
        if end > start {
            let count = text[start..end].trim().chars().count();
            if count > max {
                push(
                    &mut findings,
                    &mut total,
                    "paragraph_length",
                    "A paragraph exceeds the requested character count.",
                    Some((start, end)),
                    Some(count),
                    Some(max),
                );
            }
        }
    }
    if let Some(max) = p.max_sentence_words {
        checked.push("Whitespace-separated words per punctuation-delimited segment (not linguistic sentence detection)".into());
        let mut start = 0;
        for (index, c) in text.char_indices() {
            if matches!(c, '.' | '!' | '?' | '\n') {
                let end = index + c.len_utf8();
                let count = text[start..end].split_whitespace().count();
                if count > max {
                    push(
                        &mut findings,
                        &mut total,
                        "sentence_length_heuristic",
                        "A punctuation-delimited segment exceeds the requested word count; abbreviations and code need review.",
                        Some((start, end)),
                        Some(count),
                        Some(max),
                    );
                }
                start = end;
            }
        }
        if start < text.len() {
            let count = text[start..].split_whitespace().count();
            if count > max {
                push(
                    &mut findings,
                    &mut total,
                    "sentence_length_heuristic",
                    "A final segment exceeds the requested word count.",
                    Some((start, text.len())),
                    Some(count),
                    Some(max),
                );
            }
        }
    }
    if !p.required_headings.is_empty() {
        checked.push("Exact requested heading text on a standalone line, optionally prefixed by Markdown # markers".into());
        for heading in &p.required_headings {
            let found = text.lines().any(|line| {
                let l = line.trim();
                l == heading
                    || l.strip_prefix('#')
                        .is_some_and(|_| l.trim_start_matches('#').trim() == heading)
            });
            if !found {
                push(
                    &mut findings,
                    &mut total,
                    "missing_heading",
                    &format!("The explicitly required heading {heading:?} is absent."),
                    None,
                    None,
                    None,
                );
            }
        }
    }
    if !p.forbidden_terms.is_empty() {
        checked.push(
            "Case-sensitive literal occurrences of user-specified terms (including substrings)"
                .into(),
        );
        for term in &p.forbidden_terms {
            for (start, _) in text.match_indices(term) {
                push(
                    &mut findings,
                    &mut total,
                    "specified_term",
                    "A user-specified term occurs here.",
                    Some((start, start + term.len())),
                    None,
                    None,
                );
            }
        }
    }
    if p.plain_text {
        checked.push("Common Markdown decoration markers: heading prefixes, strong emphasis, inline/fenced code and Markdown links".into());
        let mut offset = 0;
        for line in text.split_inclusive('\n') {
            if line.trim_start().starts_with('#')
                || line.contains("**")
                || line.contains("__")
                || line.contains('`')
                || line.contains("](")
            {
                push(
                    &mut findings,
                    &mut total,
                    "markdown_marker",
                    "A common Markdown marker occurs in text requested as plain text; inspect context before changing it.",
                    Some((offset, offset + line.len())),
                    None,
                    None,
                );
            }
            offset += line.len();
        }
    }
    if let Some(marker) = &p.next_action_marker {
        checked.push("Presence of the exact user-specified next-action marker".into());
        if !text.contains(marker) {
            push(
                &mut findings,
                &mut total,
                "missing_next_action_marker",
                "The requested next-action marker is absent; action quality is not assessed.",
                None,
                None,
                None,
            );
        }
    }
    Ok(ResponseContractOutput {
        omitted_findings: total - findings.len(),
        total_findings: total,
        findings,
        checked,
        unassessed: vec![
            "Meaning, factual accuracy, completeness, tone, real actionability and understanding are not assessed.".into(),
            "No cognitive-load score, diagnostic conclusion or accessibility certification is produced.".into(),
            "Offsets use UTF-8 bytes. Code and quotations can contain intentional matches; no automatic rewrite is performed.".into(),
        ],
        review_required: true,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_requested_checks_run() -> Result<(), String> {
        let o = check_response_contract(ResponseContractRequest {
            text: "Complex text with **bold**.".into(),
            preferences: ResponsePreferences::default(),
        })?;
        assert!(o.checked.is_empty());
        assert_eq!(o.total_findings, 0);
        assert!(o.review_required);
        Ok(())
    }
    #[test]
    fn spans_and_exact_markers() -> Result<(), String> {
        let text = "☃ **Soon**\n\nNext action: read.";
        let o = check_response_contract(ResponseContractRequest {
            text: text.into(),
            preferences: ResponsePreferences {
                plain_text: true,
                forbidden_terms: vec!["Soon".into()],
                required_headings: vec!["Summary".into()],
                next_action_marker: Some("Next action:".into()),
                ..Default::default()
            },
        })?;
        assert_eq!(o.total_findings, 3);
        let f = o
            .findings
            .iter()
            .find(|f| f.category == "specified_term")
            .ok_or("missing")?;
        assert_eq!(
            text.get(f.start_byte.ok_or("start")?..f.end_byte.ok_or("end")?),
            Some("Soon")
        );
        Ok(())
    }
    #[test]
    fn finding_output_is_bounded() -> Result<(), String> {
        let o = check_response_contract(ResponseContractRequest {
            text: "x ".repeat(200),
            preferences: ResponsePreferences {
                forbidden_terms: vec!["x".into()],
                ..Default::default()
            },
        })?;
        assert_eq!(o.total_findings, 200);
        assert_eq!(o.findings.len(), 128);
        Ok(())
    }
    #[test]
    fn unicode_paragraphs_and_crlf_use_scalar_counts() -> Result<(), String> {
        let text = format!("{}\r\n\r\nshort", "☃".repeat(21));
        let output = check_response_contract(ResponseContractRequest {
            text: text.clone(),
            preferences: ResponsePreferences {
                max_paragraph_chars: Some(20),
                ..Default::default()
            },
        })?;
        assert_eq!(output.total_findings, 1);
        assert_eq!(output.findings[0].observed, Some(21));
        let end = output.findings[0].end_byte.ok_or("missing end")?;
        assert!(text.is_char_boundary(end));
        Ok(())
    }
    #[test]
    fn marker_counts_toward_combined_byte_limit() {
        let request = ResponseContractRequest {
            text: "x".repeat(MAX_TEXT_BYTES),
            preferences: ResponsePreferences {
                next_action_marker: Some("x".into()),
                ..Default::default()
            },
        };
        assert!(check_response_contract(request).is_err());
        let request = ResponseContractRequest {
            text: "x".repeat(MAX_TEXT_BYTES - 1),
            preferences: ResponsePreferences {
                next_action_marker: Some("x".into()),
                ..Default::default()
            },
        };
        assert!(check_response_contract(request).is_ok());
    }
}
