// src/analysis/dynamic_refs/java.rs

//! Java-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::DynamicReference, traits::DynamicRefDetector, DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct JavaDetector;

impl DynamicRefDetector for JavaDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect reflection
        if source.contains(".getMethod(")
            || source.contains(".invoke(")
            || source.contains("Class.forName(")
        {
            refs.push(DynamicReference::new_reflection(
                file.path.clone(),
                None,
                "getMethod".to_string(),
                None,
                "java.lang.reflect".to_string(),
                0.90,
                "Java reflection invocation".to_string(),
            ));
        }

        // Detect Spring annotations
        if source.contains("@GetMapping")
            || source.contains("@PostMapping")
            || source.contains("@RequestMapping")
            || source.contains("@RestController")
        {
            refs.push(DynamicReference::new_framework(
                file.path.clone(),
                None,
                "RestController".to_string(),
                None,
                "@RestController".to_string(),
                0.95,
                "Spring / Jakarta Web controller endpoint".to_string(),
            ));
        }

        refs
    }
}
