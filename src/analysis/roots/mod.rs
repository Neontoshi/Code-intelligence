// src/analysis/roots/mod.rs

pub mod common;
pub mod root_set;

pub use common::*;
pub use root_set::{
    ReachabilityAnalyzer, ReachabilityMap, RootDetectionConfig, RootDetector, RootSet,
};

use crate::analysis::framework_registry::FrameworkRegistry;
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;

pub type FunctionId = String;

pub struct RootDetectorOrchestrator {
    framework_registry: FrameworkRegistry,
}

impl RootDetectorOrchestrator {
    pub fn new() -> Self {
        Self {
            framework_registry: FrameworkRegistry::new(),
        }
    }

    pub fn detect_all_roots_categorized(
        &self,
        call_graph: &CallGraph,
        _files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> RootSet {
        let mut root_set = RootSet::default();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let id = func.full_path.clone();
            let language = detect_language_from_file(&func.file);

            if config.include_tests && (func.is_test || common::is_test_function_name(&func.name)) {
                root_set.add_test(id.clone());
                continue;
            }

            if config.include_ffi && common::has_ffi_attributes(func) {
                root_set.add_ffi(id.clone());
                continue;
            }

            if config.include_framework
                && self
                    .framework_registry
                    .is_framework_root(&language, &func.file, &func.name)
            {
                root_set.add_framework(id.clone());
                continue;
            }

            if config.include_exports && func.is_public {
                root_set.add_export(id.clone());
                continue;
            }

            if common::is_likely_entry_point(func, call_graph) {
                root_set.add_application(id.clone());
            }
        }

        root_set
    }

    pub fn detect_all_roots(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> RootSet {
        self.detect_all_roots_categorized(call_graph, files, config)
    }
}

pub fn detect_language_from_file(file: &str) -> String {
    if file.ends_with(".rs") {
        "rust".to_string()
    } else if file.ends_with(".py") {
        "python".to_string()
    } else if file.ends_with(".js") || file.ends_with(".jsx") {
        "javascript".to_string()
    } else if file.ends_with(".ts") || file.ends_with(".tsx") {
        "typescript".to_string()
    } else if file.ends_with(".go") {
        "go".to_string()
    } else if file.ends_with(".java") {
        "java".to_string()
    } else if file.ends_with(".dart") {
        "dart".to_string()
    } else if file.ends_with(".php") {
        "php".to_string()
    } else if file.ends_with(".cpp")
        || file.ends_with(".cc")
        || file.ends_with(".cxx")
        || file.ends_with(".hpp")
        || file.ends_with(".h")
    {
        "cpp".to_string()
    } else if file.ends_with(".cs") {
        "csharp".to_string()
    } else {
        "unknown".to_string()
    }
}

impl Default for RootDetectorOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
