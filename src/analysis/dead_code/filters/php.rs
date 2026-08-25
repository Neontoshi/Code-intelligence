// src/analysis/dead_code/filters/php.rs

//! PHP-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct PhpFilter;

impl LanguageFilter for PhpFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Magic methods
        if func.file.ends_with(".php") && func.name.starts_with("__") {
            return ProtectionLevel::Protected;
        }

        // 2. LIKELY ALIVE

        // Framework methods
        let php_framework = [
            "handle",
            "boot",
            "register",
            "authorize",
            "rules",
            "up",
            "down",
            "index",
            "show",
            "store",
            "update",
            "destroy",
        ];
        if php_framework.contains(&func.name.as_str()) {
            return ProtectionLevel::LikelyAlive;
        }

        // Functions with callers are likely alive
        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // 3. CANDIDATE
        ProtectionLevel::Candidate
    }
}
