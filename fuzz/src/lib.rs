//! Invariants exercise the production tools; no alternative parser is substituted.

use diffyn::tools::*;

fn accepts(text: &str, profile: &Option<NeurotypeProfile>) -> bool {
    !text.trim().is_empty()
        && text.len() <= MAX_TEXT_BYTES
        && profile
            .as_ref()
            .is_none_or(|profile| profile.resolve().is_ok())
}

fn check_preferences(preferences: &ResolvedPreferences) {
    assert!((1..=5).contains(&preferences.max_sentences_per_chunk));
    assert!((1..=20).contains(&preferences.max_checklist_items));
    assert!(!(preferences.plain_text && preferences.use_bold));
}

pub fn check_reformat(request: ReformatRequest) {
    let expected_ok = accepts(&request.text, &request.profile);
    let original = request.text.clone();
    let result = reformat_for_cognitive_load(request);
    assert_eq!(result.is_ok(), expected_ok);
    if let Ok(output) = result {
        assert_eq!(output.original_text, original);
        // A layout tool must not drop or change words, numbers or punctuation.
        assert!(
            original.chars().filter(|ch| !ch.is_whitespace()).eq(output
                .formatted_text
                .chars()
                .filter(|ch| !ch.is_whitespace()))
        );
        for span in &output.protected_spans {
            let protected = original
                .get(span.start_byte..span.end_byte)
                .expect("valid source span");
            assert!(output.formatted_text.contains(protected));
        }
        check_preferences(&output.preferences);
        assert!(serde_json::to_vec(&output).is_ok());
    }
}

pub fn check_checklist(request: ChecklistRequest) {
    let expected_ok = accepts(&request.complex_task, &request.profile);
    let original = request.complex_task.clone();
    let result = generate_executive_function_checklist(request);
    if !expected_ok {
        assert!(result.is_err());
    }
    if let Ok(output) = result {
        assert_eq!(output.original_task, original);
        assert!(output.suggested_steps.is_empty());
        assert!(output.extracted_steps.len() <= output.preferences.max_checklist_items);
        let all_ids: std::collections::HashSet<_> =
            output.all_steps.iter().map(|s| &s.step_id).collect();
        let visible_ids: std::collections::HashSet<_> = output
            .extracted_steps
            .iter()
            .chain(&output.deferred_steps)
            .map(|s| &s.step_id)
            .collect();
        assert_eq!(all_ids, visible_ids);
        assert_eq!(all_ids.len(), output.all_steps.len());
        assert_eq!(
            output.all_steps.len(),
            output.extracted_steps.len() + output.deferred_steps.len()
        );
        // Every extracted or deferred item must retain its source in order.
        // This catches lost conditions, invented steps and cap-related deletion.
        let mut offset = 0;
        for (index, step) in output.all_steps.iter().enumerate() {
            assert_eq!(step.source_order, index + 1);
            let relative = original[offset..]
                .find(&step.source_text)
                .expect("step must be retained from the original, in order");
            offset += relative + step.source_text.len();
            assert!(step.source_text.contains(&step.text));
        }
        if let Some(next) = &output.next_action {
            assert!(
                output
                    .extracted_steps
                    .iter()
                    .any(|step| step.status == StepStatus::Pending && &step.text == next)
            );
        }
        check_preferences(&output.preferences);
        assert!(serde_json::to_vec(&output).is_ok());
    }
}

pub fn check_clarity(request: ClarityRequest) {
    let expected_ok = accepts(&request.text, &request.profile);
    let default_rules = request.categories.is_none() && request.language.is_none();
    let original = request.text.clone();
    let result = verify_clarity_and_literalism(request);
    if default_rules {
        assert_eq!(result.is_ok(), expected_ok);
    } else if !expected_ok {
        assert!(result.is_err());
    }
    if let Ok(output) = result {
        assert_eq!(output.original_text, original);
        assert!(output.findings.len() <= 128);
        assert_eq!(
            output.total_findings,
            output.findings.len() + output.additional_findings_count
        );
        assert_eq!(
            output.truncated_findings,
            output.additional_findings_count > 0
        );
        let mut previous_start = 0;
        for finding in &output.findings {
            // get() checks both bounds and UTF-8 character boundaries.
            assert_eq!(
                original.get(finding.start_byte..finding.end_byte),
                Some(finding.phrase.as_str())
            );
            assert!(finding.start_byte >= previous_start);
            assert!(finding.start_byte < finding.end_byte);
            assert!(!output.protected_spans.iter().any(
                |span| span.start_byte < finding.end_byte && span.end_byte > finding.start_byte
            ));
            previous_start = finding.start_byte;
        }
        check_preferences(&output.preferences);
        assert!(serde_json::to_vec(&output).is_ok());
    }
}

pub fn check_text(data: &[u8]) {
    // Lossy conversion deliberately turns arbitrary bytes into valid UTF-8, so
    // this target spends its time inside the tools rather than rejecting UTF-8.
    let text = String::from_utf8_lossy(data).into_owned();
    let selector = data.first().copied().unwrap_or(0);
    let mut profile = NeurotypeProfile::default();
    profile.preferences.max_sentences_per_chunk = Some(usize::from(selector % 5) + 1);
    profile.preferences.max_checklist_items = Some(usize::from(selector % 20) + 1);
    profile.preferences.plain_text = Some(selector & 1 != 0);
    profile.preferences.use_bold = Some(selector & 2 != 0);
    profile.preferences.literal_language = Some(selector & 4 != 0);
    check_reformat(ReformatRequest {
        text: text.clone(),
        profile: Some(profile.clone()),
    });
    check_checklist(ChecklistRequest {
        complex_task: text.clone(),
        profile: Some(profile.clone()),
        ..Default::default()
    });
    // Construct a known list independently of the parser. Random quantities
    // and window sizes must never lose a condition or silently drop later items.
    let expected_steps: Vec<_> = data
        .iter()
        .take(64)
        .enumerate()
        .map(|(index, value)| format!("If condition {value} applies, keep {} items.", index + 1))
        .collect();
    if !expected_steps.is_empty() {
        let task = expected_steps
            .iter()
            .map(|step| format!("- {step}"))
            .collect::<Vec<_>>()
            .join("\n");
        let output = generate_executive_function_checklist(ChecklistRequest {
            complex_task: task,
            profile: Some(profile.clone()),
            ..Default::default()
        })
        .expect("a bounded explicit list and valid preferences must be accepted");
        let actual: Vec<_> = output.all_steps.iter().map(|step| &step.text).collect();
        assert_eq!(actual, expected_steps.iter().collect::<Vec<_>>());
    }
    check_clarity(ClarityRequest {
        text,
        profile: Some(profile),
        ..Default::default()
    });
}

pub fn check_json(data: &[u8]) {
    // Typed deserialization also exercises malformed JSON, unknown fields,
    // invalid enums, nesting and numeric limits before any domain processing.
    if let Ok(request) = serde_json::from_slice::<ReformatRequest>(data) {
        check_reformat(request);
    }
    if let Ok(request) = serde_json::from_slice::<ChecklistRequest>(data) {
        check_checklist(request);
    }
    if let Ok(request) = serde_json::from_slice::<ClarityRequest>(data) {
        check_clarity(request);
    }
    if let Ok(request) = serde_json::from_slice::<ResolvePreferencesRequest>(data)
        && let Ok(output) = resolve_communication_preferences(request)
    {
        check_preferences(&output.tool_preferences);
        assert!(serde_json::to_vec(&output).is_ok());
    }
    if let Ok(request) = serde_json::from_slice::<CompareDetailsRequest>(data) {
        let original = request.original_text.clone();
        let candidate = request.candidate_text.clone();
        if let Ok(output) = compare_protected_details(request) {
            assert!(!output.semantic_equivalence_checked);
            assert!(output.details.len() <= 128);
            assert_eq!(
                output.total_detail_groups,
                output.details.len() + output.omitted_detail_groups
            );
            for detail in output.details {
                for span in detail.original_spans {
                    assert_eq!(
                        original.get(span.start_byte..span.end_byte),
                        Some(detail.text.as_str())
                    );
                }
                for span in detail.candidate_spans {
                    assert_eq!(
                        candidate.get(span.start_byte..span.end_byte),
                        Some(detail.text.as_str())
                    );
                }
            }
        }
    }
    if let Ok(request) = serde_json::from_slice::<ValidatePlanRequest>(data)
        && let Ok(output) = validate_action_plan(request)
    {
        assert!(output.steps.len() <= 100);
        assert!(output.review_required);
        assert!(serde_json::to_vec(&output).is_ok());
    }
    if let Ok(request) = serde_json::from_slice::<ResumeCardRequest>(data)
        && let Ok(output) = build_resume_card(request)
    {
        assert!(!output.saved);
        assert!(output.steps.len() <= 100);
        assert!(serde_json::to_vec(&output).is_ok());
    }
    if let Ok(request) = serde_json::from_slice::<NumericRequest>(data)
        && let Ok(output) = explain_numeric_values(request)
    {
        assert!(!output.result.is_empty());
        assert!(output.result.len() < 128);
        assert!(output.exact_fraction.contains('/'));
        assert!(serde_json::to_vec(&output).is_ok());
    }
    if let Ok(request) = serde_json::from_slice::<ResponseContractRequest>(data) {
        let text = request.text.clone();
        if let Ok(output) = check_response_contract(request) {
            assert!(output.findings.len() <= 128);
            assert_eq!(
                output.total_findings,
                output.findings.len() + output.omitted_findings
            );
            for finding in output.findings {
                if let (Some(start), Some(end)) = (finding.start_byte, finding.end_byte) {
                    assert!(text.get(start..end).is_some());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{check_json, check_text};
    use diffyn::tools::{ChecklistRequest, ClarityRequest, ReformatRequest};

    #[test]
    fn boundary_inputs_reach_real_tools() {
        for text in [
            "",
            " \t\n",
            "First, gather 2 items. Then, wait if blocked.",
            "🧠 Cymraeg. Break a leg!",
        ] {
            check_text(text.as_bytes());
        }
        check_text(&vec![b'a'; 32768]);
        check_text(&vec![b'a'; 32769]);
        check_text(&[0xff, 0xfe, b'.', b' ', b'A']);
    }

    #[test]
    fn invalid_profiles_and_envelopes_do_not_bypass_validation() {
        for input in [
            r#"{"text":"Keep 2 items.","profile":{"preferences":{"max_checklist_items":0}}}"#,
            r#"{"text":"Keep 2 items.","profile":{"preferences":{"max_sentences_per_chunk":6}}}"#,
            r#"{"text":"Keep 2 items.","unknown":"field"}"#,
            r#"{"text":"Keep 2 items.","profile":{"self_declared_neurotypes":["unknown"]}}"#,
            r#"{"complex_task":"- [x] Done\n- [ ] Keep the condition if needed."}"#,
        ] {
            check_json(input.as_bytes());
        }
    }

    #[test]
    fn malformed_and_unknown_request_fields_are_rejected() {
        for input in [
            r#"{"text":"Keep 2 items.","unknown":"field"}"#,
            r#"{"text":"Keep 2 items.","profile":{"self_declared_neurotypes":["unknown"]}}"#,
            r#"{"text":"Keep 2 items.","profile":{"preferences":{"unknown":true}}}"#,
            r#"{"text": [true, null, {"#,
        ] {
            assert!(serde_json::from_str::<ReformatRequest>(input).is_err());
            assert!(serde_json::from_str::<ClarityRequest>(input).is_err());
        }
        assert!(
            serde_json::from_str::<ChecklistRequest>(
                r#"{"complex_task":"- Keep 2 items.","unknown":"field"}"#
            )
            .is_err()
        );
    }
}
