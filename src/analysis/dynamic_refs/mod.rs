// src/analysis/dynamic_refs/mod.rs

//! Dynamic reference detection - language-specific implementations

pub mod common;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod javascript;
pub mod php;
pub mod python;
pub mod rust;
pub mod traits;
pub mod typescript;

// Re-export types from common
pub use common::{DynamicRefType, DynamicReference, ExtractedDynamicCall};
pub use traits::DynamicRefDetector;

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

/// Main dynamic reference detector that orchestrates language-specific detectors
pub struct DynamicRefOrchestrator {
    detectors: HashMap<String, Box<dyn DynamicRefDetector>>,
}

impl DynamicRefOrchestrator {
    pub fn new() -> Self {
        let mut detectors: HashMap<String, Box<dyn DynamicRefDetector>> = HashMap::new();

        // Register all language detectors
        detectors.insert("rust".to_string(), Box::new(rust::RustDetector));
        detectors.insert("python".to_string(), Box::new(python::PythonDetector));
        detectors.insert(
            "javascript".to_string(),
            Box::new(javascript::JavaScriptDetector),
        );
        detectors.insert(
            "typescript".to_string(),
            Box::new(typescript::TypeScriptDetector),
        );
        detectors.insert("go".to_string(), Box::new(go::GoDetector));
        detectors.insert("java".to_string(), Box::new(java::JavaDetector));
        detectors.insert("dart".to_string(), Box::new(dart::DartDetector));
        detectors.insert("php".to_string(), Box::new(php::PhpDetector));
        detectors.insert("cpp".to_string(), Box::new(cpp::CppDetector));
        detectors.insert("csharp".to_string(), Box::new(csharp::CSharpDetector));

        Self { detectors }
    }

    pub fn generate_report(&self, refs: &[DynamicReference]) -> String {
        let mut output = String::new();
        output.push_str("## 🔄 Dynamic Reference Detection\n\n");

        if refs.is_empty() {
            output.push_str("✅ No dynamic references detected.\n");
            return output;
        }

        output.push_str(&format!("Found **{}** dynamic references:\n\n", refs.len()));

        let mut by_type: HashMap<DynamicRefType, Vec<&DynamicReference>> = HashMap::new();
        for r in refs {
            by_type.entry(r.reference_type.clone()).or_default().push(r);
        }

        for (ref_type, refs_by_type) in &by_type {
            output.push_str(&format!("### {:?} ({})\n\n", ref_type, refs_by_type.len()));
            for r in refs_by_type {
                let resolved_info = if let Some(path) = &r.target_full_path {
                    format!(" → resolved to `{}`", path)
                } else {
                    " ⚠️ unresolved".to_string()
                };
                output.push_str(&format!(
                    "- 🟢 **{}** (confidence: {:.0}%){}\n  - File: `{}`\n  - Context: {}\n",
                    r.target_pattern,
                    r.confidence * 100.0,
                    resolved_info,
                    r.source_file,
                    r.context
                ));
            }
            output.push('\n');
        }

        output
    }

    /// Detect all dynamic references across all files
    pub fn detect_all(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Vec<DynamicReference> {
        let mut all_refs = Vec::new();
        let mut context = DetectionContext::new(call_graph, files);

        for file in files {
            if let Some(detector) = self.detectors.get(&file.language.to_lowercase()) {
                let refs = detector.detect(file, &mut context);
                all_refs.extend(refs);
            }
        }

        // Deduplicate references
        let mut seen = std::collections::HashSet::new();
        all_refs.retain(|r| {
            let key = (
                r.source_file.clone(),
                r.target_full_path.clone().unwrap_or_default(),
                r.target_pattern.clone(),
                r.reference_type.clone(),
            );
            seen.insert(key)
        });

        all_refs
    }
}

impl Default for DynamicRefOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Context passed to detectors
pub struct DetectionContext<'a> {
    pub call_graph: &'a CallGraph,
    pub files: &'a [ParsedFile],
    pub name_to_paths: HashMap<String, Vec<String>>,
    pub lower_name_to_paths: HashMap<String, Vec<String>>,
    pub unqualified_to_paths: HashMap<String, Vec<String>>,
}

impl<'a> DetectionContext<'a> {
    pub fn new(call_graph: &'a CallGraph, files: &'a [ParsedFile]) -> Self {
        let mut name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        let mut lower_name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        let mut unqualified_to_paths: HashMap<String, Vec<String>> = HashMap::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            name_to_paths
                .entry(func.name.clone())
                .or_default()
                .push(func.full_path.clone());

            lower_name_to_paths
                .entry(func.name.to_lowercase())
                .or_default()
                .push(func.full_path.clone());

            if let Some(short_name) = func.full_path.rsplit("::").next() {
                unqualified_to_paths
                    .entry(short_name.to_string())
                    .or_default()
                    .push(func.full_path.clone());
            }
        }

        Self {
            call_graph,
            files,
            name_to_paths,
            lower_name_to_paths,
            unqualified_to_paths,
        }
    }

    /// Resolve a symbol to a full path
    pub fn resolve_symbol(&self, name: &str) -> Option<String> {
        if let Some(paths) = self.name_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        if let Some(paths) = self.unqualified_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        let lower = name.to_lowercase();
        if let Some(paths) = self.lower_name_to_paths.get(&lower) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        None
    }
}
