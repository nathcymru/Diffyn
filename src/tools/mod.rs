//! Local, bounded tools that preserve source content and expose their limits.

mod action_plan;
mod checklist;
mod clarity;
mod communication_preferences;
mod contract;
mod integrity;
mod layout;
mod numeric;
mod profile;
mod resume;
mod text_spans;

pub use action_plan::{
    Completion, PlanFinding, PlanStep, SourceReference, StepOrigin, ValidatePlanOutput,
    ValidatePlanRequest, validate_action_plan,
};
pub use checklist::{
    ChecklistOutput, ChecklistRequest, ExtractedStep, StepProgress, StepStatus,
    generate_executive_function_checklist,
};
pub use clarity::{
    ClarityCategory, ClarityFinding, ClarityOutput, ClarityRequest, verify_clarity_and_literalism,
};
pub use communication_preferences::{
    AnswerOrder, CommunicationSettings, Detail, HostPreferences, PreferenceConflict,
    PreferenceSource, QuestionStyle, ResolvePreferencesOutput, ResolvePreferencesRequest, Tone,
    resolve_communication_preferences,
};
pub use contract::{
    ContractFinding, ResponseContractOutput, ResponseContractRequest, ResponsePreferences,
    check_response_contract,
};
pub use integrity::{
    CompareDetailsOutput, CompareDetailsRequest, DetailComparison, DetailSpan,
    compare_protected_details,
};
pub use layout::{ReformatOutput, ReformatRequest, reformat_for_cognitive_load};
pub use numeric::{NumericOperation, NumericOutput, NumericRequest, explain_numeric_values};
pub use profile::{Neurotype, NeurotypeProfile, Preferences, ResolvedPreferences};
pub use resume::{ResumeCardOutput, ResumeCardRequest, ResumeStep, build_resume_card};
pub use text_spans::ProtectedSpan;

/// A product resource limit, not a clinical cognitive-load threshold.
pub const MAX_TEXT_BYTES: usize = 32 * 1024;

fn validate_text(text: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT_BYTES {
        return Err("Text must contain no more than 32768 UTF-8 bytes.".into());
    }
    if text.trim().is_empty() {
        return Err("Provide some text; whitespace-only input cannot be processed.".into());
    }
    Ok(())
}

fn resolve_profile(profile: Option<&NeurotypeProfile>) -> Result<ResolvedPreferences, String> {
    profile.cloned().unwrap_or_default().resolve()
}

#[cfg(test)]
mod tests;
