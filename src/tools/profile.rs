//! Explicit preferences take precedence over neutral product defaults.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Optional self-description only. No diagnosis, support level or ability is inferred.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Neurotype {
    Autism,
    Adhd,
    Dyslexia,
    Dyspraxia,
    #[serde(rename = "audhd", alias = "AuDHD")]
    Audhd,
}

/// Every accommodation is available without disclosing a neurotype.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct NeurotypeProfile {
    /// Self-declared labels are optional and never change the chosen settings.
    #[schemars(length(max = 5))]
    pub self_declared_neurotypes: Vec<Neurotype>,
    pub preferences: Preferences,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    /// Product default: 2. Allowed range: 1 through 5.
    #[schemars(range(min = 1, max = 5))]
    pub max_sentences_per_chunk: Option<usize>,
    /// Active checklist window, not a deletion cap. Product default: 7; range 1–20.
    #[schemars(range(min = 1, max = 20))]
    pub max_checklist_items: Option<usize>,
    pub use_bold: Option<bool>,
    /// Controls generated decoration only; never strips source Markdown or code.
    pub plain_text: Option<bool>,
    /// Requests optional literal alternatives, never automatic semantic rewriting.
    pub literal_language: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResolvedPreferences {
    pub max_sentences_per_chunk: usize,
    pub max_checklist_items: usize,
    pub use_bold: bool,
    pub plain_text: bool,
    pub literal_language: bool,
}

impl NeurotypeProfile {
    /// Resolve only explicit settings; labels are not a validated preference model.
    pub fn resolve(&self) -> Result<ResolvedPreferences, String> {
        if self.self_declared_neurotypes.len() > 5 {
            return Err("Provide no more than five optional self-declared labels.".into());
        }
        let chunk = self.preferences.max_sentences_per_chunk.unwrap_or(2);
        let checklist = self.preferences.max_checklist_items.unwrap_or(7);
        if !(1..=5).contains(&chunk) {
            return Err("max_sentences_per_chunk must be between 1 and 5.".into());
        }
        if !(1..=20).contains(&checklist) {
            return Err("max_checklist_items must be between 1 and 20.".into());
        }
        let plain_text = self.preferences.plain_text.unwrap_or(false);
        Ok(ResolvedPreferences {
            max_sentences_per_chunk: chunk,
            max_checklist_items: checklist,
            // Plain text wins a direct formatting conflict without changing words.
            use_bold: self.preferences.use_bold.unwrap_or(false) && !plain_text,
            plain_text,
            literal_language: self.preferences.literal_language.unwrap_or(true),
        })
    }
}
