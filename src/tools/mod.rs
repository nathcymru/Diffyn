//! Local, bounded tools that preserve source content and expose their limits.

mod checklist;
mod clarity;
mod layout;
mod profile;

pub use checklist::{ChecklistOutput, ChecklistRequest, generate_executive_function_checklist};
pub use clarity::{ClarityOutput, ClarityRequest, verify_clarity_and_literalism};
pub use layout::{ReformatOutput, ReformatRequest, reformat_for_cognitive_load};
pub use profile::{NeurotypeProfile, ResolvedPreferences};

#[cfg(test)]
use profile::{Neurotype, Preferences};

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
