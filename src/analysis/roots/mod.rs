// src/analysis/roots/mod.rs

//! Language-specific root detection

pub mod common;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod javascript;
pub mod php;
pub mod python;
pub mod root_set;
pub mod rust;
pub mod typescript;

pub use common::*;
pub use root_set::{
    ReachabilityAnalyzer, ReachabilityMap, RootDetectionConfig, RootDetector, RootSet,
};

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet};

pub type FunctionId = String;

/// Trait for language-specific root detectors
pub trait LanguageRootDetector: Send + Sync {
    /// Detect roots for a specific language
    fn detect_roots(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> HashSet<FunctionId>;
}

/// Main root detector orchestrator
pub struct RootDetectorOrchestrator {
    detectors: HashMap<String, Box<dyn LanguageRootDetector>>,
}

impl RootDetectorOrchestrator {
    pub fn new() -> Self {
        let mut detectors: HashMap<String, Box<dyn LanguageRootDetector>> = HashMap::new();

        detectors.insert("rust".to_string(), Box::new(rust::RustRootDetector));
        detectors.insert("python".to_string(), Box::new(python::PythonRootDetector));
        detectors.insert(
            "javascript".to_string(),
            Box::new(javascript::JavaScriptRootDetector),
        );
        detectors.insert(
            "typescript".to_string(),
            Box::new(typescript::TypeScriptRootDetector),
        );
        detectors.insert("go".to_string(), Box::new(go::GoRootDetector));
        detectors.insert("java".to_string(), Box::new(java::JavaRootDetector));
        detectors.insert("dart".to_string(), Box::new(dart::DartRootDetector));
        detectors.insert("php".to_string(), Box::new(php::PhpRootDetector));
        detectors.insert("cpp".to_string(), Box::new(cpp::CppRootDetector));
        detectors.insert("csharp".to_string(), Box::new(csharp::CSharpRootDetector));

        Self { detectors }
    }

    /// Detect all roots across all languages
    pub fn detect_all_roots(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> RootSet {
        let mut root_set = RootSet::default();

        // Detect roots for each file individually
        for file in files {
            let language = file.language.to_lowercase();
            if let Some(detector) = self.detectors.get(&language) {
                // Create a slice with just this file
                let file_slice = [file.clone()];
                let roots = detector.detect_roots(call_graph, &file_slice, config);
                // Add roots to the appropriate categories
                root_set.application.extend(roots);
            }
        }

        // Add generic roots (language-agnostic)
        root_set
            .application
            .extend(Self::detect_generic_roots(call_graph, files, config));

        root_set
    }

    /// Detect generic roots that apply to all languages
    fn detect_generic_roots(
        call_graph: &CallGraph,
        _files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        // Test functions
        if config.include_tests {
            for idx in call_graph.node_indices() {
                let func = &call_graph[idx];
                if func.is_test {
                    roots.insert(func.full_path.clone());
                }
            }
        }

        // Entry points by name
        let entry_names = ["main", "async_main", "run", "start", "init", "setup"];
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if entry_names.contains(&func.name.as_str()) {
                // Check if it's likely an entry point
                if common::is_likely_entry_point(func, call_graph) {
                    roots.insert(func.full_path.clone());
                }
            }
        }

        roots
    }
}

impl Default for RootDetectorOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
