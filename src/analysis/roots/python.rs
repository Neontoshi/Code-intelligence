// src/analysis/roots/python.rs

//! Python-specific root detection

use crate::analysis::roots::{common::*, FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct PythonRootDetector;

impl LanguageRootDetector for PythonRootDetector {
    fn detect_roots(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Application entry points
            if config.include_exports {
                if func.name == "main" || func.name == "__main__" {
                    roots.insert(func.full_path.clone());
                    continue;
                }
                if func.file.ends_with("__main__.py")
                    || func.file.ends_with("manage.py")
                    || func.file.ends_with("wsgi.py")
                {
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

            // Framework decorators (Flask, FastAPI, Django)
            if config.include_framework {
                // Check decorators from parser
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("route")
                                || d.contains("get")
                                || d.contains("post")
                                || d.contains("put")
                                || d.contains("delete")
                                || d.contains("patch")
                                || d.contains("router.")
                                || d.contains("blueprint.")
                                || d.contains("command")
                                || d.contains("fixture")
                                || d.contains("pytest")
                                || d.contains("task")
                                || d.contains("celery")
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
