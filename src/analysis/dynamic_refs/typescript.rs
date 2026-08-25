// src/analysis/dynamic_refs/typescript.rs

//! TypeScript-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::DynamicReference, traits::DynamicRefDetector, DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct TypeScriptDetector;

impl DynamicRefDetector for TypeScriptDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect dynamic imports
        if source.contains("import(") {
            refs.push(DynamicReference::new_dynamic_import(
                file.path.clone(),
                "dynamic_import".to_string(),
                None,
                "import()".to_string(),
                0.85,
                "Dynamic ES module import".to_string(),
            ));
        }

        // Detect React components and hooks (same as JS)
        if file.path.ends_with(".tsx") || file.path.ends_with(".jsx") {
            // Components are detected through the parser/filters
            // We add a framework reference for safety
            refs.push(DynamicReference::new_framework(
                file.path.clone(),
                None,
                "jsx_component".to_string(),
                None,
                "JSXComponent".to_string(),
                0.90,
                "React/JSX component - framework entry point".to_string(),
            ));
        }

        refs
    }
}
