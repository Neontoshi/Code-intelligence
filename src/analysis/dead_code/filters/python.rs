// src/analysis/dead_code/filters/python.rs

//! Python-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct PythonFilter;

impl LanguageFilter for PythonFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Python dunder methods are protected
        if func.file.ends_with(".py") && func.name.starts_with("__") && func.name.ends_with("__") {
            return ProtectionLevel::Protected;
        }

        // Framework-decorated functions are likely alive
        if let Some(doc) = &func.doc_comment {
            let decorator_patterns = [
                "@app.route",
                "@router.",
                "@blueprint.",
                "@get",
                "@post",
                "@put",
                "@delete",
                "@patch",
            ];
            for pattern in decorator_patterns {
                if doc.contains(pattern) {
                    return ProtectionLevel::LikelyAlive;
                }
            }
        }

        // 2. LIKELY ALIVE

        // Functions with callers are likely alive
        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // 3. CANDIDATE
        ProtectionLevel::Candidate
    }
}
