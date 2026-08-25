// src/analysis/dead_code/filters/go.rs

//! Go-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct GoFilter;

impl LanguageFilter for GoFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED

        // Test and benchmark functions
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }
        if func.name.starts_with("Test")
            || func.name.starts_with("Benchmark")
            || func.name.starts_with("Example")
        {
            return ProtectionLevel::Protected;
        }

        // init functions are called by Go runtime
        if func.name == "init" {
            return ProtectionLevel::Protected;
        }

        // 2. LIKELY ALIVE

        // Exported functions (capitalized) in libraries
        let is_exported = func
            .name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

        if is_exported && (func.file.contains("/internal/") || func.fan_in == 0) {
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
