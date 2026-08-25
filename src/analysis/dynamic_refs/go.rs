// src/analysis/dynamic_refs/go.rs

//! Go-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::DynamicReference, traits::DynamicRefDetector, DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct GoDetector;

impl DynamicRefDetector for GoDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect reflection
        if source.contains("reflect.")
            || source.contains("\"reflect\"")
            || source.contains("MethodByName")
        {
            refs.push(DynamicReference::new_reflection(
                file.path.clone(),
                None,
                "reflect".to_string(),
                None,
                "reflect.ValueOf".to_string(),
                0.90,
                "Go reflect package usage".to_string(),
            ));
        }

        refs
    }
}
