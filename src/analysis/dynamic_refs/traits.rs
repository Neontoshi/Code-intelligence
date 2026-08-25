// src/analysis/dynamic_refs/traits.rs

//! Traits for dynamic reference detection

use crate::analysis::dynamic_refs::{DetectionContext, DynamicReference};
use crate::parser::tree_sitter::ParsedFile;

/// Trait for language-specific dynamic reference detectors
pub trait DynamicRefDetector: Send + Sync {
    /// Detect dynamic references in a file
    fn detect(&self, file: &ParsedFile, context: &mut DetectionContext) -> Vec<DynamicReference>;
}
