//! Validate supplied plan structure; never invent or certify a procedure.
use super::{MAX_TEXT_BYTES, validate_text};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepOrigin {
    Source,
    User,
    Proposal,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Completion {
    #[default]
    Unknown,
    NotStarted,
    Pending,
    InProgress,
    Completed,
    Blocked,
    Skipped,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceReference {
    pub start_byte: usize,
    pub end_byte: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanStep {
    #[schemars(length(min = 1, max = 64))]
    pub id: String,
    #[schemars(length(min = 1, max = 4096))]
    pub text: String,
    pub origin: StepOrigin,
    #[serde(default)]
    pub source_ref: Option<SourceReference>,
    #[serde(default)]
    #[schemars(length(max = 100))]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub completion: Completion,
}
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidatePlanRequest {
    #[serde(default)]
    #[schemars(length(max = 32768))]
    pub source_text: Option<String>,
    #[schemars(length(min = 1, max = 100))]
    pub steps: Vec<PlanStep>,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PlanFinding {
    pub category: String,
    pub step_id: Option<String>,
    pub related_id: Option<String>,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ValidatePlanOutput {
    pub steps: Vec<PlanStep>,
    pub findings: Vec<PlanFinding>,
    pub total_findings: usize,
    pub omitted_findings: usize,
    pub structurally_valid: bool,
    pub cycle_step_ids: Vec<String>,
    pub review_required: bool,
    pub limitations: Vec<String>,
}
fn finding(category: &str, step: &str, related: Option<&str>, message: &str) -> PlanFinding {
    PlanFinding {
        category: category.into(),
        step_id: Some(step.into()),
        related_id: related.map(str::to_owned),
        message: message.into(),
    }
}
pub fn validate_action_plan(request: ValidatePlanRequest) -> Result<ValidatePlanOutput, String> {
    if request.steps.is_empty() || request.steps.len() > 100 {
        return Err("Supply between 1 and 100 plan steps.".into());
    }
    if let Some(source) = &request.source_text {
        validate_text(source)?;
    }
    let mut total = request.source_text.as_ref().map_or(0, String::len);
    for step in &request.steps {
        if step.id.trim().is_empty()
            || step.id.len() > 64
            || step.text.trim().is_empty()
            || step.text.len() > 4096
            || step.depends_on.len() > 100
            || step
                .depends_on
                .iter()
                .any(|id| id.trim().is_empty() || id.len() > 64)
        {
            return Err("Plan fields exceed the published limits.".into());
        }
        total += step.id.len()
            + step.text.len()
            + step.depends_on.iter().map(String::len).sum::<usize>();
    }
    if total > MAX_TEXT_BYTES {
        return Err("Combined plan text exceeds 32768 UTF-8 bytes.".into());
    }
    let mut findings = Vec::new();
    let mut ids = BTreeMap::new();
    for (index, step) in request.steps.iter().enumerate() {
        if ids.insert(step.id.as_str(), index).is_some() {
            findings.push(finding(
                "duplicate_id",
                &step.id,
                None,
                "Step identifiers must be unique.",
            ));
        }
        if step.origin == StepOrigin::Source && step.source_ref.is_none() {
            findings.push(finding(
                "missing_source_reference",
                &step.id,
                None,
                "A source step needs an exact source byte range.",
            ));
        }
        if let Some(span) = &step.source_ref {
            let exact = request.source_text.as_ref().and_then(|s| {
                if span.start_byte < span.end_byte {
                    s.get(span.start_byte..span.end_byte)
                } else {
                    None
                }
            });
            if exact != Some(step.text.as_str()) {
                findings.push(finding(
                    "source_mismatch",
                    &step.id,
                    None,
                    "The source range must be valid UTF-8 boundaries and equal the step text exactly.",
                ));
            }
        }
        if step.origin == StepOrigin::Proposal && step.completion == Completion::Completed {
            findings.push(finding(
                "unconfirmed_completion",
                &step.id,
                None,
                "A proposal alone cannot establish completed work; supply user or source provenance.",
            ));
        }
    }
    for step in &request.steps {
        let mut seen = BTreeSet::new();
        for dep in &step.depends_on {
            if !seen.insert(dep) {
                findings.push(finding(
                    "duplicate_dependency",
                    &step.id,
                    Some(dep),
                    "A dependency was repeated.",
                ));
            }
            if !ids.contains_key(dep.as_str()) {
                findings.push(finding(
                    "missing_dependency",
                    &step.id,
                    Some(dep),
                    "The referenced prerequisite is absent.",
                ));
            }
        }
    }
    // Bounded graph traversal reports only nodes actually in a cycle, not descendants blocked by it.
    let mut cycle_step_ids = Vec::new();
    for step in &request.steps {
        let mut pending = step.depends_on.clone();
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if id == step.id {
                cycle_step_ids.push(step.id.clone());
                break;
            }
            if seen.insert(id.clone())
                && let Some(index) = ids.get(id.as_str())
            {
                pending.extend(request.steps[*index].depends_on.iter().cloned());
            }
        }
    }
    cycle_step_ids.sort();
    cycle_step_ids.dedup();
    for id in &cycle_step_ids {
        findings.push(finding(
            "dependency_cycle",
            id,
            None,
            "This step belongs to a dependency cycle.",
        ));
    }
    let total_findings = findings.len();
    findings.truncate(128);
    Ok(ValidatePlanOutput {
        structurally_valid: total_findings == 0,
        total_findings,
        omitted_findings: total_findings - findings.len(),
        steps: request.steps,
        findings,
        cycle_step_ids,
        review_required: true,
        limitations: vec![
            "Checks supplied identifiers, exact source references and dependency structure only; does not establish completeness, chronology, feasibility or procedure safety.".into(),
            "Completion and origin are caller declarations, not independently verified facts. No state is retained.".into(),
        ],
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graph_and_utf8_provenance() -> Result<(), String> {
        let r: ValidatePlanRequest = serde_json::from_value(serde_json::json!({
            "source_text": "Cau'r drws. ☃",
            "steps": [
                {
                    "id": "a",
                    "text": "Cau'r drws.",
                    "origin": "source",
                    "source_ref": {
                        "start_byte": 0,
                        "end_byte": 11
                    },
                    "depends_on": [
                        "b"
                    ]
                },
                {
                    "id": "b",
                    "text": "Wait",
                    "origin": "proposal",
                    "depends_on": [
                        "a"
                    ]
                },
                {
                    "id": "c",
                    "text": "Continue",
                    "origin": "user",
                    "depends_on": [
                        "a"
                    ]
                }
            ]
        }))
        .map_err(|e| e.to_string())?;
        let o = validate_action_plan(r)?;
        assert_eq!(o.cycle_step_ids, ["a", "b"]);
        assert!(!o.findings.iter().any(|f| f.category == "source_mismatch"));
        Ok(())
    }
    #[test]
    fn cannot_complete_a_proposal() -> Result<(), String> {
        let r: ValidatePlanRequest = serde_json::from_value(serde_json::json!({
            "steps": [
                {
                    "id": "a",
                    "text": "Start",
                    "origin": "proposal",
                    "completion": "completed",
                    "depends_on": [
                        "missing"
                    ]
                }
            ]
        }))
        .map_err(|e| e.to_string())?;
        let o = validate_action_plan(r)?;
        assert_eq!(o.findings.len(), 2);
        assert!(!o.structurally_valid);
        Ok(())
    }
    #[test]
    fn invalid_utf8_source_ranges_are_findings_not_panics() -> Result<(), String> {
        let request: ValidatePlanRequest = serde_json::from_value(serde_json::json!({
            "source_text": "☃",
            "steps": [
                {
                    "id": "a",
                    "text": "☃",
                    "origin": "source",
                    "source_ref": {
                        "start_byte": 1,
                        "end_byte": 3
                    }
                }
            ]
        }))
        .map_err(|e| e.to_string())?;
        let output = validate_action_plan(request)?;
        assert!(!output.structurally_valid);
        assert_eq!(output.findings[0].category, "source_mismatch");
        Ok(())
    }
    #[test]
    fn expanded_findings_are_bounded() -> Result<(), String> {
        let step = PlanStep {
            id: "a".into(),
            text: "Read".into(),
            origin: StepOrigin::User,
            source_ref: None,
            depends_on: vec!["missing".into(); 100],
            completion: Completion::Unknown,
        };
        let output = validate_action_plan(ValidatePlanRequest {
            source_text: None,
            steps: vec![step],
        })?;
        assert_eq!(output.total_findings, 199);
        assert_eq!(output.findings.len(), 128);
        assert_eq!(output.omitted_findings, 71);
        assert!(!output.structurally_valid);
        Ok(())
    }
}
