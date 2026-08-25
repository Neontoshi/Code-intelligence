// src/analysis/roots/go.rs

//! Go-specific root detection

use crate::analysis::roots::{common::*, FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct GoRootDetector;

impl LanguageRootDetector for GoRootDetector {
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
                if func.name == "main" {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // init functions are called by Go runtime
            if config.include_framework {
                if func.name == "init" {
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

            // Exported functions in libraries
            if config.include_exports {
                let is_exported = func
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                if is_exported && func.fan_in == 0 {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }
        }

        roots
    }
}
