// src/analysis/roots/csharp.rs

//! C#-specific root detection

use crate::analysis::roots::{common::*, FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct CSharpRootDetector;

impl LanguageRootDetector for CSharpRootDetector {
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

            // Entry points
            if config.include_exports {
                if func.file.ends_with("Program.cs")
                    || func.file.ends_with("Startup.cs")
                    || func.name == "Main"
                {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // ASP.NET Core attributes
            if config.include_framework {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("httpget")
                                || d.contains("httppost")
                                || d.contains("httpput")
                                || d.contains("httpdelete")
                                || d.contains("route")
                                || d.contains("apicontroller")
                                || d.contains("authorize")
                                || d.contains("fact") // xUnit test
                                || d.contains("test")
                            // NUnit test
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
