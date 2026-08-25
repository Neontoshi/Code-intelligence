// src/analysis/dynamic_refs/rust.rs

//! Rust-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::{DynamicRefType, DynamicReference},
    traits::DynamicRefDetector,
    DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct RustDetector;

impl DynamicRefDetector for RustDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect dyn Trait usage
        if source.contains("dyn ") || source.contains("Box<dyn") || source.contains("&dyn") {
            refs.push(DynamicReference {
                source_file: file.path.clone(),
                source_function: None,
                target_function: Some("dyn".to_string()),
                target_full_path: None,
                target_pattern: "dyn Trait".to_string(),
                reference_type: DynamicRefType::DynamicDispatch,
                confidence: 0.85,
                context: "Rust trait object dynamic dispatch".to_string(),
                resolved: false,
            });
        }

        // Detect FFI exports
        if source.contains("#[no_mangle]")
            || source.contains("extern \"C\"")
            || source.contains("extern \"system\"")
        {
            refs.push(DynamicReference::new_framework(
                file.path.clone(),
                None,
                "ffi_export".to_string(),
                None,
                "extern \"C\"".to_string(),
                0.95,
                "Rust FFI exported function".to_string(),
            ));
        }

        refs
    }
}
