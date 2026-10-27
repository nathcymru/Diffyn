//! A caller-owned checkpoint, with no inferred progress or persistence.
use super::action_plan::Completion;
use super::{MAX_TEXT_BYTES, validate_text};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResumeStep {
    #[schemars(length(min = 1, max = 64))]
    pub id: String,
    #[schemars(length(min = 1, max = 4096))]
    pub text: String,
    pub status: Completion,
}
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResumeCardRequest {
    #[schemars(length(min = 1, max = 4096))]
    pub goal: String,
    #[schemars(length(max = 100))]
    pub steps: Vec<ResumeStep>,
    #[serde(default)]
    #[schemars(length(max = 64))]
    pub current_step_id: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 64))]
    pub next_step_id: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 20))]
    pub blockers: Vec<String>,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ResumeCardOutput {
    pub goal: String,
    pub steps: Vec<ResumeStep>,
    pub completed_step_ids: Vec<String>,
    pub current_step: Option<ResumeStep>,
    pub next_action: Option<ResumeStep>,
    pub blockers: Vec<String>,
    pub plain_text: String,
    pub provenance: String,
    pub saved: bool,
    pub limitations: Vec<String>,
}
pub fn build_resume_card(request: ResumeCardRequest) -> Result<ResumeCardOutput, String> {
    validate_text(&request.goal)?;
    if request.goal.len() > 4096 || request.steps.len() > 100 || request.blockers.len() > 20 {
        return Err("Checkpoint fields exceed the published limits.".into());
    }
    let mut ids = BTreeSet::new();
    let mut total = request.goal.len();
    for step in &request.steps {
        if step.id.trim().is_empty()
            || step.id.len() > 64
            || !ids.insert(step.id.as_str())
            || step.text.trim().is_empty()
            || step.text.len() > 4096
        {
            return Err(
                "Checkpoint steps need unique bounded identifiers and nonblank text.".into(),
            );
        }
        total += step.id.len() + step.text.len();
    }
    for blocker in &request.blockers {
        if blocker.trim().is_empty() || blocker.len() > 4096 {
            return Err("Checkpoint blockers exceed the published limits.".into());
        }
        total += blocker.len();
    }
    for id in [&request.current_step_id, &request.next_step_id]
        .into_iter()
        .flatten()
    {
        if id.len() > 64 || !ids.contains(id.as_str()) {
            return Err("A checkpoint reference does not identify a supplied step.".into());
        }
        total += id.len();
    }
    if total > MAX_TEXT_BYTES {
        return Err("Combined checkpoint text exceeds 32768 UTF-8 bytes.".into());
    }
    let find = |id: &Option<String>| {
        id.as_ref()
            .and_then(|id| request.steps.iter().find(|s| &s.id == id))
            .cloned()
    };
    let current_step = find(&request.current_step_id);
    let next_action = find(&request.next_step_id);
    if next_action.as_ref().is_some_and(|s| {
        matches!(
            s.status,
            Completion::Completed | Completion::Skipped | Completion::Blocked
        )
    }) || current_step
        .as_ref()
        .is_some_and(|s| matches!(s.status, Completion::Completed | Completion::Skipped))
    {
        return Err("Current and next steps cannot be completed or skipped; a blocked step cannot be selected as the next action.".into());
    }
    let completed_step_ids = request
        .steps
        .iter()
        .filter(|s| s.status == Completion::Completed)
        .map(|s| s.id.clone())
        .collect();
    let mut plain_text = format!("Goal: {}\n", request.goal);
    plain_text.push_str("Progress (supplied by the caller):\n");
    for step in &request.steps {
        let status = match step.status {
            Completion::Unknown => "unknown",
            Completion::NotStarted => "not started",
            Completion::Pending => "pending",
            Completion::InProgress => "in progress",
            Completion::Completed => "completed",
            Completion::Blocked => "blocked",
            Completion::Skipped => "skipped",
        };
        plain_text.push_str(&format!("{}: {} — {}\n", step.id, status, step.text));
    }
    plain_text.push_str(&format!(
        "Current step: {}\nNext action: {}\n",
        current_step
            .as_ref()
            .map_or("not supplied", |s| s.text.as_str()),
        next_action
            .as_ref()
            .map_or("not supplied", |s| s.text.as_str())
    ));
    if request.blockers.is_empty() {
        plain_text.push_str("Blockers: none supplied\n");
    } else {
        plain_text.push_str("Blockers:\n");
        for b in &request.blockers {
            plain_text.push_str(&format!("- {b}\n"));
        }
    }
    Ok(ResumeCardOutput {
        goal: request.goal,
        steps: request.steps,
        completed_step_ids,
        current_step,
        next_action,
        blockers: request.blockers,
        plain_text,
        provenance: "All goal, progress, blocker and next-action information was explicitly supplied by the caller.".into(),
        saved: false,
        limitations: vec![
            "No progress is inferred or independently verified. No safe stopping point is assessed.".into(),
            "This card is returned only; Diffyn does not save it, remember it across requests, or schedule reminders.".into(),
        ],
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_inferred_next_action() -> Result<(), String> {
        let r: ResumeCardRequest = serde_json::from_value(serde_json::json!({
            "goal": "Make tea",
            "steps": [
                {
                    "id": "a",
                    "text": "Get mug",
                    "status": "not_started"
                }
            ]
        }))
        .map_err(|e| e.to_string())?;
        let o = build_resume_card(r)?;
        assert!(o.next_action.is_none());
        assert!(!o.saved);
        assert!(o.completed_step_ids.is_empty());
        Ok(())
    }
    #[test]
    fn rejects_unknown_and_completed_references() -> Result<(), String> {
        for id in ["a", "absent"] {
            let r: ResumeCardRequest = serde_json::from_value(serde_json::json!({
                "goal": "Make tea",
                "steps": [
                    {
                        "id": "a",
                        "text": "Get mug",
                        "status": "completed"
                    }
                ],
                "next_step_id": id
            }))
            .map_err(|e| e.to_string())?;
            assert!(build_resume_card(r).is_err());
        }
        Ok(())
    }
    #[test]
    fn preserves_checklist_statuses_without_coercion() -> Result<(), String> {
        let r: ResumeCardRequest = serde_json::from_value(serde_json::json!({
            "goal": "Continue",
            "steps": [
                {
                    "id": "a",
                    "text": "Read",
                    "status": "pending"
                },
                {
                    "id": "b",
                    "text": "Wait",
                    "status": "blocked"
                },
                {
                    "id": "c",
                    "text": "Optional",
                    "status": "skipped"
                }
            ],
            "next_step_id": "a"
        }))
        .map_err(|e| e.to_string())?;
        let o = build_resume_card(r)?;
        assert_eq!(o.steps[0].status, Completion::Pending);
        assert_eq!(o.steps[1].status, Completion::Blocked);
        assert_eq!(o.steps[2].status, Completion::Skipped);
        assert!(o.completed_step_ids.is_empty());
        Ok(())
    }
    #[test]
    fn blocked_current_state_is_preserved_but_not_recommended_next() -> Result<(), String> {
        for (current, next) in [(Some("a"), None), (None, Some("a"))] {
            let request: ResumeCardRequest = serde_json::from_value(serde_json::json!({
                "goal": "Send report",
                "steps": [
                    {
                        "id": "a",
                        "text": "Send the report",
                        "status": "blocked"
                    }
                ],
                "current_step_id": current,
                "next_step_id": next,
                "blockers": [
                    "Waiting for approval"
                ]
            }))
            .map_err(|e| e.to_string())?;
            assert_eq!(build_resume_card(request).is_ok(), next.is_none());
        }
        Ok(())
    }
}
