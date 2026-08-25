// src/analysis/dead_code/filters/java.rs

//! Java-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct JavaFilter;

impl LanguageFilter for JavaFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Java standard methods
        let standard_methods = [
            "equals",
            "hashCode",
            "toString",
            "compareTo",
            "close",
            "destroy",
        ];
        if standard_methods.contains(&func.name.as_str()) {
            return ProtectionLevel::Protected;
        }

        // 2. LIKELY ALIVE

        // Framework-decorated functions
        if let Some(doc) = &func.doc_comment {
            let decorator_patterns = [
                "@GetMapping",
                "@PostMapping",
                "@PutMapping",
                "@DeleteMapping",
                "@PatchMapping",
                "@RequestMapping",
                "@RestController",
                "@Controller",
                "@Service",
                "@Repository",
                "@Component",
            ];
            for pattern in decorator_patterns {
                if doc.contains(pattern) {
                    return ProtectionLevel::LikelyAlive;
                }
            }
        }

        // Functions with callers are likely alive
        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // 3. CANDIDATE
        ProtectionLevel::Candidate
    }
}
