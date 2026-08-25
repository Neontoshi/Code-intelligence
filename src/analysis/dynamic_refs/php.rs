// src/analysis/dynamic_refs/php.rs

//! PHP-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::DynamicReference, traits::DynamicRefDetector, DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct PhpDetector;

impl DynamicRefDetector for PhpDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect dynamic function calls
        if source.contains("call_user_func")
            || source.contains("call_user_func_array")
            || source.contains("forward_static_call")
            || source.contains("ReflectionClass")
            || source.contains("ReflectionMethod")
        {
            refs.push(DynamicReference::new_reflection(
                file.path.clone(),
                None,
                "call_user_func".to_string(),
                None,
                "call_user_func".to_string(),
                0.90,
                "PHP dynamic call dispatch".to_string(),
            ));
        }

        refs
    }
}
