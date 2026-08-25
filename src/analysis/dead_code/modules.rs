// src/analysis/dead_code/modules.rs

use crate::graph::import_graph::{ImportEdge, ImportGraph};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DeadModuleReport {
    pub unused_modules: Vec<DeadModule>,
    pub unused_files: Vec<DeadFile>,
    pub unused_imports: Vec<DeadImport>,
}

#[derive(Debug, Clone)]
pub struct DeadModule {
    pub name: String,
    pub path: PathBuf,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct DeadFile {
    pub path: PathBuf,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct DeadImport {
    pub module: String,
    pub imported_by: String,
    pub line: usize,
    pub confidence: f64,
}

pub struct ModuleDeadCodeDetector;

impl ModuleDeadCodeDetector {
    pub fn detect_dead_modules(import_graph: &ImportGraph) -> DeadModuleReport {
        let mut unused_modules = Vec::new();
        let mut unused_files = Vec::new();
        let mut unused_imports = Vec::new();

        // 1. Find unused files (files that are never imported)
        let unimported_files = import_graph.find_unimported_files();

        for file in unimported_files {
            // Check if it's a module file (has exports)
            let exports = import_graph.get_exports(&file);

            // If it has exports but is never imported, it's dead
            if !exports.is_empty() {
                let confidence = Self::calculate_file_confidence(import_graph, &file, &exports);

                // Only report if confidence is high enough
                if confidence > 0.6 {
                    unused_files.push(DeadFile {
                        path: PathBuf::from(&file),
                        confidence,
                        reason: format!(
                            "File exports {} functions but is never imported",
                            exports.len()
                        ),
                    });
                }
            }
        }

        // 2. Find unused imports within files
        let all_edges: Vec<_> = import_graph.iter_edges().cloned().collect();
        for edge in all_edges {
            // Check if this import is used
            let is_used = Self::is_import_used(import_graph, &edge);

            if !is_used {
                let confidence = Self::calculate_import_confidence(import_graph, &edge);

                if confidence > 0.5 {
                    unused_imports.push(DeadImport {
                        module: edge.target_file.clone(),
                        imported_by: edge.source_file.clone(),
                        line: edge.import_info.line,
                        confidence,
                    });
                }
            }
        }

        // 3. Find unused modules (directories/mod.rs files with no exports)
        for node in import_graph.iter_nodes() {
            if node.path.ends_with("mod.rs") || node.path.ends_with("mod") {
                let exports = import_graph.get_exports(&node.path);
                if exports.is_empty() {
                    unused_modules.push(DeadModule {
                        name: node.name.clone(),
                        path: PathBuf::from(&node.path),
                        confidence: 0.8,
                        reason: "Module has no exports and is never imported".to_string(),
                    });
                }
            }
        }

        DeadModuleReport {
            unused_modules,
            unused_files,
            unused_imports,
        }
    }

    fn calculate_file_confidence(
        _import_graph: &ImportGraph,
        file: &str,
        exports: &[String],
    ) -> f64 {
        let mut confidence: f64 = 0.8;

        if exports.len() > 5 {
            confidence *= 0.9;
        }

        // Check if it's a test file
        if file.contains("/tests/") || file.contains("/test/") || file.ends_with("_test.rs") {
            confidence *= 0.5; // Test files are often run by cargo test
        }

        // Check if it's a bin file
        if file.contains("/bin/") || file.starts_with("src/bin/") {
            confidence *= 0.4; // Binary files might be entry points
        }

        // Check if it's a lib file (often not imported directly)
        if file.ends_with("lib.rs") {
            confidence *= 0.7; // lib.rs is often the root
        }

        confidence.max(0.0).min(1.0)
    }

    fn is_import_used(import_graph: &ImportGraph, edge: &ImportEdge) -> bool {
        // Get all functions exported by the imported module
        let exported_funcs = import_graph.get_imported_functions(&edge.target_file);

        // Check if any of these functions are used in the source file
        for func in &exported_funcs {
            if import_graph.is_function_used_in_file(func, &edge.source_file) {
                return true;
            }
        }

        false
    }

    fn calculate_import_confidence(import_graph: &ImportGraph, edge: &ImportEdge) -> f64 {
        let mut confidence: f64 = 0.7;

        // If the import is from a common module, it might be needed
        let import_count = import_graph.import_count(&edge.target_file);
        if import_count > 1 {
            confidence += 0.2;
        }

        // If the import is from the standard library, it might be needed
        if edge.target_file.contains("std::") || edge.target_file.contains("core::") {
            confidence += 0.1;
        }

        // If the import is from a test module, it might be less critical
        if edge.target_file.contains("test") || edge.target_file.contains("tests") {
            confidence -= 0.2;
        }

        // Check if the imported module has exports
        let exports = import_graph.get_exports(&edge.target_file);
        if exports.is_empty() {
            confidence -= 0.2;
        }

        confidence.max(0.0).min(1.0)
    }
}
