// src/analysis/dead_code/filters/javascript.rs

//! JavaScript-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct JavaScriptFilter;

impl LanguageFilter for JavaScriptFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // React components and hooks
        let is_jsx_file = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
        let is_ts_family = is_jsx_file || func.file.ends_with(".ts") || func.file.ends_with(".js");

        if is_ts_family {
            let is_component = is_jsx_file
                && func
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
            let is_hook = func.name.starts_with("use")
                && func
                    .name
                    .chars()
                    .nth(3)
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);

            if is_component || is_hook {
                return ProtectionLevel::Protected;
            }

            // UI event handlers
            if func.name.starts_with("handle") || func.name.starts_with("on") {
                return ProtectionLevel::LikelyAlive;
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
