//! Extract source steps without inventing a plan or claiming dependency analysis.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    NeurotypeProfile, ResolvedPreferences,
    layout::{has_structured_block, has_verbatim_syntax, sentence_separators},
    resolve_profile, validate_text,
};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChecklistRequest {
    pub complex_task: String,
    #[serde(default)]
    pub profile: Option<NeurotypeProfile>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ExtractedStep {
    pub source_order: usize,
    pub text: String,
    pub source_text: String,
    /// Only explicit source checkbox status is copied; otherwise this is unknown.
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ChecklistOutput {
    pub original_task: String,
    pub extraction_method: String,
    pub context: String,
    pub extracted_steps: Vec<ExtractedStep>,
    /// Steps beyond the active window remain available and are never discarded.
    pub deferred_steps: Vec<ExtractedStep>,
    pub suggested_steps: Vec<String>,
    pub checklist_text: String,
    pub next_action: Option<String>,
    pub pause_after_item: Option<usize>,
    pub missing_information: Vec<String>,
    pub preferences: ResolvedPreferences,
    pub review_required: bool,
    pub limitations: Vec<String>,
}

pub fn generate_executive_function_checklist(
    request: ChecklistRequest,
) -> Result<ChecklistOutput, String> {
    validate_text(&request.complex_task)?;
    let preferences = resolve_profile(request.profile.as_ref())?;
    let mut context = String::new();
    let mut method = "insufficient_structure";
    let mut steps = Vec::new();

    if !has_structured_block(&request.complex_task) {
        let (listed, preamble) = extract_list(&request.complex_task);
        if !listed.is_empty() {
            steps = listed;
            context = preamble;
            method = "explicit_list";
        } else if !request.complex_task.lines().any(has_verbatim_syntax) {
            let sequential = extract_sequence(&request.complex_task);
            if sequential.len() >= 2 {
                steps = sequential;
                method = "explicit_sequence";
            }
        }
    }

    if !steps.is_empty() && steps.iter().all(|step| step.text.trim().is_empty()) {
        // Marker-only items are not actions. The original still retains them.
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
    let deferred_steps = if steps.len() > preferences.max_checklist_items {
        steps.split_off(preferences.max_checklist_items)
    } else {
        Vec::new()
    };
    let next_action = steps
        .iter()
        .chain(deferred_steps.iter())
        .find(|step| step.completed != Some(true) && !step.text.trim().is_empty())
        .map(|step| step.text.clone());
    let pause_after_item = (!steps.is_empty()).then_some(steps.len());
    let mut checklist_text = String::new();
    if !context.is_empty() {
        checklist_text.push_str(&context);
        checklist_text.push_str("\n\n");
    }
    if !steps.is_empty() {
        checklist_text.push_str(if preferences.use_bold {
            "**Source steps to review**\n\n"
        } else {
            "Source steps to review\n\n"
        });
        for step in &steps {
            // Keep the source order and conditions. Numbering is presentation;
            // a copied source item may still contain more than one action.
            let status = match (step.completed, preferences.plain_text) {
                (Some(true), true) => "[completed] ",
                (Some(false), true) => "[not completed] ",
                (Some(true), false) => "[x] ",
                (Some(false), false) => "[ ] ",
                (None, _) => "",
            };
            checklist_text.push_str(&format!("{}. {status}{}\n", step.source_order, step.text));
        }
        if !deferred_steps.is_empty() {
            checklist_text.push_str(&format!(
                "\n{} additional source step(s) are retained in deferred_steps. This is a display window, not permission to omit later steps.\n",
                deferred_steps.len()
            ));
        }
    } else {
        checklist_text.push_str(
            "A reliable checklist needs explicit steps. The original task is retained unchanged.",
        );
    }

    Ok(ChecklistOutput {
        original_task: request.complex_task,
        extraction_method: method.into(),
        context,
        extracted_steps: steps,
        deferred_steps,
        suggested_steps: Vec::new(),
        checklist_text,
        next_action,
        pause_after_item,
        missing_information,
        preferences,
        review_required: true,
        limitations: vec![
            "Extraction only: source order is preserved, but chronology, completeness, safety, and one-action granularity have not been verified.".into(),
            "No durations, prerequisites, equipment, dependencies, or inferred subtasks are invented. Conditions and continuation lines stay with their source item.".into(),
            "Freeform multiline text is not assumed to contain independent actions. Provide a marked list or explicit First/Then/Finally sequence when clarification is needed.".into(),
            "The active window and pause marker are presentation suggestions. Complete any safety-critical sequence as required; a display boundary is not a safe stopping instruction.".into(),
            "Completion status is copied only from explicit source checkboxes; other completion status is unknown.".into(),
        ],
    })
}

fn extract_list(text: &str) -> (Vec<ExtractedStep>, String) {
    let mut steps: Vec<ExtractedStep> = Vec::new();
    let mut preamble = Vec::new();
    let mut base_indent = None;
    for line in text.split('\n') {
        let indent = line.len() - line.trim_start().len();
        let marker = list_marker(line.trim_start());
        // Nested bullets are context within their parent, not new peer actions.
        if let Some((content, completed)) = marker
            && base_indent.is_none_or(|base| indent <= base)
            // Four-space/tab indentation is Markdown code when no outer list
            // has established list context. Never promote code into actions.
            && !(steps.is_empty() && (indent >= 4 || line.starts_with('\t')))
        {
            base_indent = Some(indent);
            steps.push(ExtractedStep {
                source_order: steps.len() + 1,
                text: content.to_owned(),
                source_text: line.to_owned(),
                completed,
            });
        } else if let Some(step) = steps.last_mut() {
            step.text.push('\n');
            step.text.push_str(line);
            step.source_text.push('\n');
            step.source_text.push_str(line);
        } else {
            preamble.push(line);
        }
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
        slices.push(text[previous..end].trim());
        previous = next;
    }
    slices.push(text[previous..].trim());
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
        .map(|(index, text)| ExtractedStep {
            source_order: index + 1,
            text: text.to_owned(),
            source_text: text.to_owned(),
            completed: None,
        })
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
