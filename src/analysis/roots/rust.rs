// src/analysis/roots/rust.rs

//! Rust-specific root detection

use crate::analysis::roots::{common::*, FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct RustRootDetector;

impl LanguageRootDetector for RustRootDetector {
    fn detect_roots(
        &self,
        call_graph: &CallGraph,
        _files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Application entry points
            if config.include_exports {
                if func.name == "main" || func.name == "async_main" {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // Test functions
            if config.include_tests {
                if func.is_test || is_test_function_name(&func.name) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // FFI exports
            if config.include_ffi && has_ffi_attributes(func) {
                roots.insert(func.full_path.clone());
                continue;
            }

            // Public API exports
            if config.include_exports && func.is_public {
                // In Rust, public functions in lib.rs or mod.rs are API exports
                if func.file.ends_with("lib.rs")
                    || func.file.ends_with("mod.rs")
                    || func.file.contains("/lib/")
                {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // Framework callbacks - Rust-specific
            if config.include_framework {
                // React-like components in Rust (Yew, Dioxus)
                if func.file.ends_with(".rs") && func.is_public {
                    let is_component = func
                        .name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    if is_component
                        && (func.file.contains("/components/") || func.file.contains("/pages/"))
                    {
                        roots.insert(func.full_path.clone());
                        continue;
                    }
                }
            }
        }

        roots
    }
}
