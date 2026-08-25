// src/analysis/dynamic_refs/python.rs

//! Python-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::DynamicReference, traits::DynamicRefDetector, DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct PythonDetector;

impl DynamicRefDetector for PythonDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect reflection patterns
        if source.contains("getattr(") || source.contains("setattr(") || source.contains("hasattr(")
        {
            refs.push(DynamicReference::new_reflection(
                file.path.clone(),
                None,
                "getattr".to_string(),
                None,
                "getattr".to_string(),
                0.85,
                "Python reflection dispatch".to_string(),
            ));
        }

        // Detect importlib
        if source.contains("importlib") || source.contains("__import__") {
            refs.push(DynamicReference::new_reflection(
                file.path.clone(),
                None,
                "importlib".to_string(),
                None,
                "importlib".to_string(),
                0.85,
                "Python dynamic import".to_string(),
            ));
        }

        // Detect Flask/FastAPI routes
        if source.contains("@app.route") || source.contains("@router.") {
            refs.push(DynamicReference::new_framework(
                file.path.clone(),
                None,
                "app.route".to_string(),
                None,
                "@app.route".to_string(),
                0.95,
                "Flask/FastAPI route handler".to_string(),
            ));
        }

        refs
    }
}
