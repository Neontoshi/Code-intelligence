// src/analysis/roots/php.rs

//! PHP-specific root detection

use crate::analysis::roots::{common::*, FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct PhpRootDetector;

impl LanguageRootDetector for PhpRootDetector {
    fn detect_roots(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
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

            // Framework routes
            if config.include_framework {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("route")
                                || d.contains("get")
                                || d.contains("post")
                                || d.contains("livewire")
                            {
                                roots.insert(func.full_path.clone());
                                break;
                            }
                        }
                    }
                }
            }

            // Entry points
            if config.include_exports {
                if func.file.ends_with("index.php")
                    || func.file.ends_with("artisan")
                    || func.file.contains("/bin/console")
                {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }
        }

        roots
    }
}
