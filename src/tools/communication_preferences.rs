//! Resolve only supplied preferences, with explicit scope and provenance.
use super::ResolvedPreferences;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Detail {
    Concise,
    Standard,
    Detailed,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Neutral,
    Warm,
    Direct,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerOrder {
    AnswerFirst,
    ContextFirst,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionStyle {
    OneAtATime,
    Grouped,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct CommunicationSettings {
    #[schemars(range(min = 1, max = 5))]
    pub max_sentences_per_chunk: Option<usize>,
    #[schemars(range(min = 1, max = 20))]
    pub max_checklist_items: Option<usize>,
    pub use_bold: Option<bool>,
    pub plain_text: Option<bool>,
    pub literal_language: Option<bool>,
    #[schemars(length(min = 1, max = 64))]
    pub language: Option<String>,
    pub detail: Option<Detail>,
    pub tone: Option<Tone>,
    pub answer_order: Option<AnswerOrder>,
    pub question_style: Option<QuestionStyle>,
}
#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ResolvePreferencesRequest {
    pub general: CommunicationSettings,
    pub task: CommunicationSettings,
    pub current_request: CommunicationSettings,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PreferenceSource {
    pub field: String,
    pub scope: String,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PreferenceConflict {
    pub field: String,
    pub lower_scope: String,
    pub winning_scope: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct HostPreferences {
    pub language: Option<String>,
    pub detail: Detail,
    pub tone: Tone,
    pub answer_order: AnswerOrder,
    pub question_style: QuestionStyle,
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ResolvePreferencesOutput {
    pub tool_preferences: ResolvedPreferences,
    pub host_preferences: HostPreferences,
    pub sources: Vec<PreferenceSource>,
    pub conflicts: Vec<PreferenceConflict>,
    pub precedence: Vec<String>,
    pub limitations: Vec<String>,
}

fn select<T: Clone + PartialEq>(
    field: &str,
    default: T,
    values: [Option<&T>; 3],
    sources: &mut Vec<PreferenceSource>,
    conflicts: &mut Vec<PreferenceConflict>,
) -> T {
    let mut result = default;
    let mut scope = "default";
    for (name, candidate) in ["general", "task", "current_request"]
        .into_iter()
        .zip(values)
    {
        if let Some(value) = candidate {
            if scope != "default" && *value != result {
                conflicts.push(PreferenceConflict {
                    field: field.into(),
                    lower_scope: scope.into(),
                    winning_scope: name.into(),
                    reason: "The more specific explicit scope takes precedence.".into(),
                });
            }
            result = value.clone();
            scope = name;
        }
    }
    sources.push(PreferenceSource {
        field: field.into(),
        scope: scope.into(),
    });
    result
}

pub fn resolve_communication_preferences(
    request: ResolvePreferencesRequest,
) -> Result<ResolvePreferencesOutput, String> {
    for p in [&request.general, &request.task, &request.current_request] {
        if p.max_sentences_per_chunk
            .is_some_and(|v| !(1..=5).contains(&v))
            || p.max_checklist_items
                .is_some_and(|v| !(1..=20).contains(&v))
            || p.language
                .as_ref()
                .is_some_and(|s| s.trim().is_empty() || s.len() > 64)
        {
            return Err("Communication preferences exceed the published limits.".into());
        }
    }
    let mut sources = Vec::new();
    let mut conflicts = Vec::new();
    macro_rules! pick {
        ($field:ident, $default:expr) => {
            select(
                stringify!($field),
                $default,
                [
                    request.general.$field.as_ref(),
                    request.task.$field.as_ref(),
                    request.current_request.$field.as_ref(),
                ],
                &mut sources,
                &mut conflicts,
            )
        };
    }
    let mut tool_preferences = ResolvedPreferences {
        max_sentences_per_chunk: pick!(max_sentences_per_chunk, 2),
        max_checklist_items: pick!(max_checklist_items, 7),
        use_bold: pick!(use_bold, false),
        plain_text: pick!(plain_text, false),
        literal_language: pick!(literal_language, true),
    };
    if tool_preferences.plain_text && tool_preferences.use_bold {
        conflicts.push(PreferenceConflict {
            field: "use_bold".into(),
            lower_scope: "effective_preferences".into(),
            winning_scope: "plain_text".into(),
            reason: "Plain text suppresses generated bold decoration.".into(),
        });
        tool_preferences.use_bold = false;
    }
    let language = pick!(language, String::new());
    let language = if language.is_empty() {
        None
    } else {
        Some(language)
    };
    let host_preferences = HostPreferences {
        language,
        detail: pick!(detail, Detail::Standard),
        tone: pick!(tone, Tone::Neutral),
        answer_order: pick!(answer_order, AnswerOrder::AnswerFirst),
        question_style: pick!(question_style, QuestionStyle::OneAtATime),
    };
    Ok(ResolvePreferencesOutput {
        tool_preferences,
        host_preferences,
        sources,
        conflicts,
        precedence: ["default", "general", "task", "current_request"]
            .map(str::to_owned)
            .to_vec(),
        limitations: vec![
            "Host preferences are guidance, not settings implemented by the local text tools. Each tool applies only its documented settings.".into(),
            "Preferences are caller-supplied and are not stored. No diagnosis or ability is inferred.".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn precedence_and_plain_text_are_explicit() -> Result<(), String> {
        let r: ResolvePreferencesRequest = serde_json::from_value(serde_json::json!({
            "general": {
                "detail": "detailed",
                "use_bold": true
            },
            "task": {
                "detail": "concise"
            },
            "current_request": {
                "plain_text": true,
                "language": "cy"
            }
        }))
        .map_err(|e| e.to_string())?;
        let out = resolve_communication_preferences(r)?;
        assert_eq!(out.host_preferences.detail, Detail::Concise);
        assert_eq!(out.host_preferences.language.as_deref(), Some("cy"));
        assert!(!out.tool_preferences.use_bold);
        assert_eq!(out.conflicts.len(), 2);
        Ok(())
    }
    #[test]
    fn invalid_lower_scope_is_not_hidden() {
        let mut r = ResolvePreferencesRequest::default();
        r.general.max_checklist_items = Some(100);
        r.task.max_checklist_items = Some(2);
        assert!(resolve_communication_preferences(r).is_err());
        assert!(
            serde_json::from_str::<ResolvePreferencesRequest>(
                r#"{"general":{"diagnosis":"private"}}"#
            )
            .is_err()
        );
    }
}
