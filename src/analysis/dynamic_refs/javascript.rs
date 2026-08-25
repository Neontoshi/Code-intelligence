// src/analysis/dynamic_refs/javascript.rs

//! JavaScript-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::DynamicReference, traits::DynamicRefDetector, DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct JavaScriptDetector;

impl DynamicRefDetector for JavaScriptDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect dynamic imports
        if source.contains("import(") || source.contains("require(") {
            refs.push(DynamicReference::new_dynamic_import(
                file.path.clone(),
                "dynamic_import".to_string(),
                None,
                "import()".to_string(),
                0.85,
                "Dynamic ES module import".to_string(),
            ));
        }

        refs
    }
}
