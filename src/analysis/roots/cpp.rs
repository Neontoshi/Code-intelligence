// src/analysis/roots/cpp.rs

//! C++-specific root detection

use crate::analysis::roots::{common::*, FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct CppRootDetector;

impl LanguageRootDetector for CppRootDetector {
    fn detect_roots(
        &self,
        call_graph: &CallGraph,
        _files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Test functions
            if config.include_tests {
                if func.is_test || is_test_function_name(&func.name) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // FFI exports
            if config.include_ffi {
                if has_ffi_attributes(func) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // Entry points
            if config.include_exports {
                if func.name == "main"
                    || matches!(
                        func.name.as_str(),
                        "WinMain" | "WndProc" | "DllMain" | "GetThisFromHandle"
                    )
                    || func.name.starts_with("my_application_")
                    || func.name.ends_with("_class_init")
                {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }
        }

        roots
    }
}
