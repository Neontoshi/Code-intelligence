// src/analysis/dead_code/filters/cpp.rs

//! C++-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct CppFilter;

impl LanguageFilter for CppFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Special member functions and destructors
        if (func.file.ends_with(".cpp")
            || func.file.ends_with(".cc")
            || func.file.ends_with(".hpp")
            || func.file.ends_with(".h"))
            && (func.name.starts_with('~') || func.name == "main" || func.name == "operator=")
        {
            return ProtectionLevel::Protected;
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
