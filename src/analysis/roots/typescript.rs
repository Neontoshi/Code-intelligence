// src/analysis/roots/typescript.rs

//! TypeScript-specific root detection

use crate::analysis::roots::{FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct TypeScriptRootDetector;

impl LanguageRootDetector for TypeScriptRootDetector {
    fn detect_roots(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        // TypeScript shares some rules with JavaScript
        // But we'll implement directly instead of trying to delegate

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
                let is_ts_family =
                    is_jsx_file || func.file.ends_with(".ts") || func.file.ends_with(".js");

                if is_ts_family {
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
                let is_entry_barrel = func.file.ends_with("/index.ts")
                    || func.file.ends_with("/index.tsx")
                    || func.file.ends_with("/main.ts")
                    || func.file.ends_with("/mod.ts")
                    || func.file.ends_with("/lib.ts");

                if func.is_public && (is_entry_barrel || func.fan_in == 0) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // TypeScript-specific: decorators
            if config.include_framework {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("controller")
                                || d.contains("get")
                                || d.contains("post")
                                || d.contains("put")
                                || d.contains("delete")
                                || d.contains("injectable")
                                || d.contains("module")
                            {
                                roots.insert(func.full_path.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }

        roots
    }
}

// Helper function - moved from common
fn is_test_function_name(name: &str) -> bool {
    name.starts_with("test_")
        || name.starts_with("Test")
        || name.starts_with("bench_")
        || name.starts_with("Benchmark")
        || name.starts_with("Example")
}
