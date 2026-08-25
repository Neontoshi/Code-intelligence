// src/analysis/dynamic_refs/cpp.rs

//! C++-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::{DynamicRefType, DynamicReference},
    traits::DynamicRefDetector,
    DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct CppDetector;

impl DynamicRefDetector for CppDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect FFI exports
        if source.contains("extern \"C\"")
            || source.contains("Q_INVOKABLE")
            || source.contains("EMSCRIPTEN_KEEPALIVE")
        {
            refs.push(DynamicReference::new_framework(
                file.path.clone(),
                None,
                "extern_c".to_string(),
                None,
                "extern \"C\"".to_string(),
                0.95,
                "C++ FFI / Native exported entry point".to_string(),
            ));
        }

        // Detect virtual dispatch
        if source.contains("virtual ") || source.contains("override") {
            refs.push(DynamicReference {
                source_file: file.path.clone(),
                source_function: None,
                target_function: Some("virtual".to_string()),
                target_full_path: None,
                target_pattern: "virtual method".to_string(),
                reference_type: DynamicRefType::DynamicDispatch,
                confidence: 0.85,
                context: "C++ virtual polymorphic dispatch".to_string(),
                resolved: false,
            });
        }

        refs
    }
}
