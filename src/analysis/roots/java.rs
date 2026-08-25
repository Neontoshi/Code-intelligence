// src/analysis/roots/java.rs

//! Java-specific root detection

use crate::analysis::roots::{common::*, FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct JavaRootDetector;

impl LanguageRootDetector for JavaRootDetector {
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
                if func.name == "main" {
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

            // Framework annotations (Spring)
            if config.include_framework {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("getmapping")
                                || d.contains("postmapping")
                                || d.contains("putmapping")
                                || d.contains("deletemapping")
                                || d.contains("requestmapping")
                                || d.contains("restcontroller")
                                || d.contains("controller")
                                || d.contains("service")
                                || d.contains("repository")
                                || d.contains("component")
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
