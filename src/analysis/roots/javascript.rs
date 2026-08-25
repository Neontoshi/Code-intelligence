// src/analysis/roots/javascript.rs

//! JavaScript-specific root detection

use crate::analysis::roots::{FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct JavaScriptRootDetector;

impl LanguageRootDetector for JavaScriptRootDetector {
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

            // Framework components and hooks
            if config.include_framework {
                let is_jsx_file = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
                let is_js_family =
                    is_jsx_file || func.file.ends_with(".ts") || func.file.ends_with(".js");

                if is_js_family {
                    let is_component = is_jsx_file
                        && (func
                            .name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                            || func.file.contains("/pages/")
                            || func.file.contains("/components/"));
                    let is_hook = func.name.starts_with("use")
                        && func
                            .name
                            .chars()
                            .nth(3)
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false);

                    let in_framework_dir = func.file.contains("/component")
                        || func.file.contains("/page")
                        || func.file.contains("/hooks/")
                        || func.file.contains("/stores/")
                        || func.file.contains("/services/");

                    if is_hook || is_component || (func.is_public && in_framework_dir) {
                        roots.insert(func.full_path.clone());
                        continue;
                    }
                }
            }

            // Exports - barrel files
            if config.include_exports {
                let is_entry_barrel = func.file.ends_with("/index.js")
                    || func.file.ends_with("/index.jsx")
                    || func.file.ends_with("/main.js")
                    || func.file.ends_with("/mod.js");

                if func.is_public && (is_entry_barrel || func.fan_in == 0) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }
        }

        roots
    }
}

// Helper function
fn is_test_function_name(name: &str) -> bool {
    name.starts_with("test_")
        || name.starts_with("Test")
        || name.starts_with("bench_")
        || name.starts_with("Benchmark")
        || name.starts_with("Example")
}
