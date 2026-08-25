// src/analysis/dead_code/filters/protection.rs

//! Main protection logic that combines all filters

use super::{FilterOrchestrator, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

/// Check if a function should never be considered dead
pub fn is_never_dead(func: &FunctionNode) -> bool {
    get_protection_level(func) == ProtectionLevel::Protected
}

/// Check if a function is likely alive (not a good candidate for deletion)
pub fn is_likely_alive(func: &FunctionNode) -> bool {
    matches!(
        get_protection_level(func),
        ProtectionLevel::Protected | ProtectionLevel::LikelyAlive
    )
}

/// Check if a function is actionable (can be considered for deletion)
pub fn is_actionable(func: &FunctionNode) -> bool {
    get_protection_level(func).is_actionable()
}

/// Get the protection level for a function
pub fn get_protection_level(func: &FunctionNode) -> ProtectionLevel {
    let orchestrator = FilterOrchestrator::new();
    orchestrator.get_protection_level(func)
}

/// Get a human-readable reason why a function is filtered
pub fn filter_reason(func: &FunctionNode) -> Option<&'static str> {
    match get_protection_level(func) {
        ProtectionLevel::Protected => {
            if func.is_test {
                Some("protected: test_function")
            } else if func.is_trait_default {
                Some("protected: trait_default_method")
            } else if func.is_trait_method {
                Some("protected: trait_method")
            } else if func.trait_impl.is_some() {
                Some("protected: trait_implementation")
            } else {
                Some("protected: ffi_or_framework")
            }
        }
        ProtectionLevel::LikelyAlive => {
            if func.is_public {
                Some("likely_alive: public_api")
            } else if func.fan_in > 0 {
                Some("likely_alive: has_callers")
            } else {
                Some("likely_alive: entry_point")
            }
        }
        ProtectionLevel::Candidate => None,
    }
}

/// Get the protection level as a color/emoji for display
pub fn protection_level_emoji(level: ProtectionLevel) -> &'static str {
    match level {
        ProtectionLevel::Protected => "🛡️",
        ProtectionLevel::LikelyAlive => "🟢",
        ProtectionLevel::Candidate => "🟠",
    }
}

/// Get a human-readable description of the protection level
pub fn protection_level_description(level: ProtectionLevel) -> &'static str {
    match level {
        ProtectionLevel::Protected => {
            "Protected - Never considered dead (trait impl, FFI, test, etc.)"
        }
        ProtectionLevel::LikelyAlive => {
            "Likely Alive - High confidence alive (public API, entry point, has callers)"
        }
        ProtectionLevel::Candidate => "Candidate - May be dead, needs analysis",
    }
}

/// Check if a function should be prioritized for review
pub fn should_prioritize_review(func: &FunctionNode) -> bool {
    let level = get_protection_level(func);
    match level {
        ProtectionLevel::Candidate => func.is_public || func.complexity > 10.0,
        _ => false,
    }
}
