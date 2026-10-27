//! Extract source steps and apply explicitly supplied progress without inventing a plan.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

use super::{
    NeurotypeProfile, ResolvedPreferences, layout::sentence_separators, resolve_profile,
    text_spans::protected_spans, validate_text,
};

const MAX_STEPS: usize = 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Completed,
    Blocked,
    Skipped,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StepProgress {
    pub step_id: String,
    pub status: StepStatus,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChecklistRequest {
    pub complex_task: String,
    #[serde(default)]
    pub profile: Option<NeurotypeProfile>,
    /// Required when supplying progress or a start_at_step_id; copy from the prior result.
    #[serde(default)]
    pub source_revision: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 1024))]
    pub progress: Vec<StepProgress>,
    /// Begin at this source step, then show unfinished steps in source order.
    #[serde(default)]
    pub start_at_step_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ExtractedStep {
    pub step_id: String,
    pub source_order: usize,
    pub text: String,
    pub source_text: String,
    /// Original source checkbox only. Caller progress never changes this evidence.
    pub completed: Option<bool>,
    pub status: StepStatus,
    /// source_checkbox, unspecified, or caller_reported.
    pub status_origin: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ChecklistOutput {
    pub original_task: String,
    pub source_revision: String,
    pub extraction_method: String,
    pub context: String,
    /// Active unfinished display window; includes blocked items for review.
    pub extracted_steps: Vec<ExtractedStep>,
    /// Every source item outside the active window, including completed/skipped items.
    pub deferred_steps: Vec<ExtractedStep>,
    pub all_steps: Vec<ExtractedStep>,
    pub active_start_step_id: Option<String>,
    pub suggested_steps: Vec<String>,
    pub checklist_text: String,
    pub next_action: Option<String>,
    pub pause_after_item: Option<usize>,
    pub missing_information: Vec<String>,
    pub preferences: ResolvedPreferences,
    pub review_required: bool,
    pub warnings: Vec<String>,
    pub limitations: Vec<String>,
}

pub fn generate_executive_function_checklist(
    request: ChecklistRequest,
) -> Result<ChecklistOutput, String> {
    validate_text(&request.complex_task)?;
    let preferences = resolve_profile(request.profile.as_ref())?;
    let source_revision = format!("sha256:{}", source_digest(&request.complex_task));
    if request.progress.len() > MAX_STEPS {
        return Err("Provide no more than 1024 progress entries.".into());
    }
    let carries_state = !request.progress.is_empty() || request.start_at_step_id.is_some();
    if (carries_state && request.source_revision.is_none())
        || request
            .source_revision
            .as_ref()
            .is_some_and(|revision| revision != &source_revision)
    {
        return Err("Progress or window state requires the matching source_revision from an unchanged source.".into());
    }
    let mut context = String::new();
    let mut method = "insufficient_structure";
    let (mut steps, preamble) = extract_list(&request.complex_task);
    if !steps.is_empty() {
        context = preamble;
        method = "explicit_list";
    } else if protected_spans(&request.complex_task).is_empty() {
        let sequential = extract_sequence(&request.complex_task);
        if sequential.len() >= 2 {
            steps = sequential;
            method = "explicit_sequence";
        }
    }
    if steps.len() > MAX_STEPS {
        return Err(
            "Provide no more than 1024 explicit source steps; no steps were truncated.".into(),
        );
    }
    if !steps.is_empty() && steps.iter().all(|step| step.text.trim().is_empty()) {
        steps.clear();
        method = "insufficient_structure";
    }
    let missing_information = if steps.is_empty() {
        vec!["What is the first concrete action you already know is needed? You can also provide an ordered list of the steps you want to use.".into()]
    } else if steps.iter().any(|step| step.text.trim().is_empty()) {
        vec![
            "Some marked source items contain no action. What belongs in those blank items?".into(),
        ]
    } else {
        Vec::new()
    };

    let mut seen = HashSet::new();
    for progress in &request.progress {
        if !seen.insert(&progress.step_id) {
            return Err("Each step may have only one supplied progress entry.".into());
        }
        let Some(step) = steps
            .iter_mut()
            .find(|step| step.step_id == progress.step_id)
        else {
            return Err("A supplied progress step ID does not belong to this source.".into());
        };
        step.status = progress.status;
        step.status_origin = "caller_reported".into();
    }
    let start = match request.start_at_step_id {
        Some(ref id) => steps
            .iter()
            .position(|step| &step.step_id == id)
            .ok_or("The requested window step ID does not belong to this source.")?,
        None => 0,
    };
    let is_unfinished =
        |step: &ExtractedStep| matches!(step.status, StepStatus::Pending | StepStatus::Blocked);
    let mut warnings = Vec::new();
    if steps[..start].iter().any(is_unfinished) {
        warnings.push("Earlier unfinished source steps remain outside this window; moving the display does not verify dependencies or authorize skipping them.".into());
    }
    let active_indices: Vec<usize> = steps
        .iter()
        .enumerate()
        .skip(start)
        .filter(|(_, step)| is_unfinished(step))
        .take(preferences.max_checklist_items)
        .map(|(index, _)| index)
        .collect();
    let active_set: HashSet<usize> = active_indices.iter().copied().collect();
    let active: Vec<_> = active_indices
        .iter()
        .map(|index| steps[*index].clone())
        .collect();
    let deferred_steps = steps
        .iter()
        .enumerate()
        .filter(|(index, _)| !active_set.contains(index))
        .map(|(_, step)| step.clone())
        .collect();
    // A blocked first meaningful item must not silently promote a dependent later one.
    let first_action = active.iter().find(|step| !step.text.trim().is_empty());
    let next_action = first_action
        .filter(|step| step.status == StepStatus::Pending)
        .map(|step| step.text.clone());
    if first_action.is_some_and(|step| step.status == StepStatus::Blocked) {
        warnings.push("The first active action is reported blocked. No later action was promoted because dependencies have not been verified.".into());
    }
    let pause_after_item = (!active.is_empty()).then_some(active.len());
    let active_start_step_id = active.first().map(|step| step.step_id.clone());
    let mut checklist_text = String::new();
    if !context.is_empty() {
        checklist_text.push_str(&context);
        checklist_text.push_str("\n\n");
    }
    if !active.is_empty() {
        checklist_text.push_str(if preferences.use_bold {
            "**Source steps to review**\n\n"
        } else {
            "Source steps to review\n\n"
        });
        for step in &active {
            let status = if step.status == StepStatus::Blocked {
                "[blocked] "
            } else if step.status_origin == "unspecified" {
                ""
            } else if preferences.plain_text {
                "[not completed] "
            } else {
                "[ ] "
            };
            checklist_text.push_str(&format!("{}. {status}{}\n", step.source_order, step.text));
        }
    } else if steps.is_empty() {
        checklist_text.push_str(
            "A reliable checklist needs explicit steps. The original task is retained unchanged.",
        );
    } else {
        checklist_text.push_str("No unfinished source steps are in this window. Review the full sequence and reported progress before treating the task as complete.");
    }
    if !steps.is_empty() {
        let outside = steps.len() - active.len();
        checklist_text.push_str(&format!("\n{outside} source step(s) are outside this display window. All source steps, including conditions and reported progress, remain in all_steps. A display boundary is not a safe stopping instruction.\n"));
    }
    Ok(ChecklistOutput {
        original_task: request.complex_task,
        source_revision,
        extraction_method: method.into(),
        context,
        extracted_steps: active,
        deferred_steps,
        all_steps: steps,
        active_start_step_id,
        suggested_steps: Vec::new(),
        checklist_text,
        next_action,
        pause_after_item,
        missing_information,
        preferences,
        review_required: true,
        warnings,
        limitations: vec![
            "Extraction only: source order is preserved, but chronology, completeness, safety, and one-action granularity have not been verified.".into(),
            "No durations, prerequisites, equipment, dependencies, or inferred subtasks are invented. Conditions and continuation lines stay with their source item; source context and all_steps remain available across windows.".into(),
            "Freeform multiline text is not assumed to contain independent actions. Provide a marked list or explicit First/Then/Finally sequence when clarification is needed.".into(),
            "The active window and pause marker are presentation suggestions, not verified safe stopping points. Skipped or completed status is evidence reported by the source or caller, not independent verification.".into(),
            "completed retains only original checkbox evidence. A pending status with status_origin unspecified means eligible for display, not verified incompletion.".into(),
            "Step IDs and source revisions are stable for unchanged source. The caller must retain and resubmit progress; Diffyn stores no state.".into(),
        ],
    })
}

fn source_digest(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn make_step(
    order: usize,
    text: String,
    source_text: String,
    completed: Option<bool>,
) -> ExtractedStep {
    ExtractedStep {
        step_id: format!("step-{order}-{}", &source_digest(&source_text)[..32]),
        source_order: order,
        text,
        source_text,
        completed,
        status: if completed == Some(true) {
            StepStatus::Completed
        } else {
            StepStatus::Pending
        },
        status_origin: if completed.is_some() {
            "source_checkbox"
        } else {
            "unspecified"
        }
        .into(),
    }
}

fn extract_list(text: &str) -> (Vec<ExtractedStep>, String) {
    let protected = protected_spans(text);
    let mut steps: Vec<ExtractedStep> = Vec::new();
    let mut preamble = Vec::new();
    let mut base_indent = None;
    let mut offset = 0;
    for line in text.split('\n') {
        let display_line = line.strip_suffix('\r').unwrap_or(line);
        let indent = display_line.len() - display_line.trim_start().len();
        let marker = list_marker(line.trim_start());
        // Check the marker itself, not linked/quoted content within an explicit action.
        let marker_start = offset + indent;
        let bare_checkbox = ["[ ] ", "[x] ", "[X] "]
            .iter()
            .any(|prefix| display_line.trim_start().starts_with(prefix));
        let protected_marker = protected.iter().any(|span| {
            span.start_byte <= marker_start
                && span.end_byte > marker_start
                && span.kind != "mathematical_or_currency_line"
                && !(bare_checkbox
                    && span.kind == "bracket_or_link"
                    && span.start_byte == marker_start
                    && span.end_byte - span.start_byte == 3)
        });
        if let Some((content, completed)) = marker
            && !protected_marker
            && base_indent.is_none_or(|base| indent <= base)
            && !(steps.is_empty() && (indent >= 4 || line.starts_with('\t')))
        {
            base_indent = Some(indent);
            steps.push(make_step(
                steps.len() + 1,
                content.to_owned(),
                line.to_owned(),
                completed,
            ));
        } else if let Some(step) = steps.last_mut() {
            step.text.push('\n');
            step.text.push_str(line);
            step.source_text.push('\n');
            step.source_text.push_str(line);
        } else {
            preamble.push(line);
        }
        offset += line.len() + 1;
    }
    // Continuation content participates in the stable source ID.
    for step in &mut steps {
        step.step_id = format!(
            "step-{}-{}",
            step.source_order,
            &source_digest(&step.source_text)[..32]
        );
    }
    (steps, preamble.join("\n"))
}

fn list_marker(text: &str) -> Option<(&str, Option<bool>)> {
    let content = if let Some(rest) = text
        .strip_prefix("- ")
        .or_else(|| text.strip_prefix("* "))
        .or_else(|| text.strip_prefix("+ "))
    {
        rest
    } else if let Some((prefix, rest)) = text.split_once(['.', ')'])
        && !prefix.is_empty()
        && prefix.len() <= 6
        && prefix.chars().all(|ch| ch.is_ascii_digit())
        && rest.starts_with(char::is_whitespace)
    {
        rest.trim_start()
    } else if text.starts_with("[ ] ") || text.starts_with("[x] ") || text.starts_with("[X] ") {
        text
    } else {
        return None;
    };
    if let Some(rest) = content.strip_prefix("[ ] ") {
        Some((rest, Some(false)))
    } else if let Some(rest) = content
        .strip_prefix("[x] ")
        .or_else(|| content.strip_prefix("[X] "))
    {
        Some((rest, Some(true)))
    } else {
        Some((content, None))
    }
}

fn extract_sequence(text: &str) -> Vec<ExtractedStep> {
    // Only explicit sequencing words permit splitting. Never split a bare
    // conjunction or arbitrary newline: both can carry conditions or prose.
    let mut boundaries = sentence_separators(text);
    for (index, ch) in text.char_indices() {
        if ch == ';' || ch == '\n' {
            boundaries.push((index + ch.len_utf8(), index + ch.len_utf8()));
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    let mut slices = Vec::new();
    let mut previous = 0;
    for (end, next) in boundaries {
        let slice = text[previous..end].trim();
        if !slice.is_empty() {
            slices.push(slice);
        }
        previous = next;
    }
    let last = text[previous..].trim();
    if !last.is_empty() {
        slices.push(last);
    }
    if slices.len() < 2
        || slices
            .iter()
            .any(|slice| !starts_with_sequence_marker(slice))
    {
        return Vec::new();
    }
    slices
        .into_iter()
        .enumerate()
        .map(|(index, text)| make_step(index + 1, text.to_owned(), text.to_owned(), None))
        .collect()
}

fn starts_with_sequence_marker(text: &str) -> bool {
    let folded = text.to_lowercase();
    [
        "first",
        "then",
        "next",
        "finally",
        "afterwards",
        "yn gyntaf",
        "wedyn",
        "yna",
        "nesaf",
        "yn olaf",
    ]
    .iter()
    .any(|marker| {
        folded.strip_prefix(marker).is_some_and(|tail| {
            tail.chars()
                .next()
                .is_some_and(|ch| ch.is_whitespace() || ch == ',')
        })
    })
}
