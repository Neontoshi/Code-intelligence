// src/analysis/dead_code/filters/rust.rs

//! Rust-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct RustFilter;

impl LanguageFilter for RustFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED - Mathematically/semantically guaranteed alive

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Trait default methods are protected
        if func.is_trait_default {
            return ProtectionLevel::Protected;
        }

        // Trait methods are protected
        if func.is_trait_method {
            return ProtectionLevel::Protected;
        }

        // Trait implementations are protected
        if func.trait_impl.is_some() {
            return ProtectionLevel::Protected;
        }

        // FFI functions are protected
        if let Some(doc) = &func.doc_comment {
            if doc.contains("extern \"C\"")
                || doc.contains("#[no_mangle]")
                || doc.contains("#[export_name]")
                || doc.contains("#[link_name]")
            {
                return ProtectionLevel::Protected;
            }
        }

        // Entry points are protected
        let entry_points = ["main", "async_main", "run", "start", "init", "setup"];
        if entry_points.contains(&func.name.as_str()) {
            return ProtectionLevel::Protected;
        }

        // 2. LIKELY ALIVE - High confidence alive

        // Public API functions are likely alive (especially in libraries)
        if func.is_public && (func.file.contains("lib.rs") || func.file.contains("mod.rs")) {
            return ProtectionLevel::LikelyAlive;
        }

        // Functions with callers are likely alive
        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // 3. CANDIDATE - May be dead, needs analysis
        ProtectionLevel::Candidate
    }
}
