// src/analysis/dead_code/filters/mod.rs

pub mod common;
pub mod protection;

pub use common::*;
pub use protection::*;

use crate::analysis::framework_registry::FrameworkRegistry;
use crate::analysis::roots::{detect_language_from_file, has_ffi_attributes};
use crate::graph::call_graph::FunctionNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtectionLevel {
    Protected,
    LikelyAlive,
    Candidate,
}

impl ProtectionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtectionLevel::Protected => "protected",
            ProtectionLevel::LikelyAlive => "likely_alive",
            ProtectionLevel::Candidate => "candidate",
        }
    }

    pub fn is_actionable(&self) -> bool {
        matches!(self, ProtectionLevel::Candidate)
    }

    pub fn is_safe_to_delete(&self) -> bool {
        matches!(self, ProtectionLevel::Candidate)
    }

    pub fn needs_review(&self) -> bool {
        matches!(
            self,
            ProtectionLevel::LikelyAlive | ProtectionLevel::Candidate
        )
    }
}

pub struct UnifiedFilter {
    framework_registry: FrameworkRegistry,
}

impl UnifiedFilter {
    pub fn new() -> Self {
        Self {
            framework_registry: FrameworkRegistry::new(),
        }
    }

    pub fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        let language = detect_language_from_file(&func.file);

        // 1. PROTECTED - Guaranteed alive

        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        if func.is_trait_method || func.is_trait_default || func.trait_impl.is_some() {
            return ProtectionLevel::Protected;
        }

        if has_ffi_attributes(func) {
            return ProtectionLevel::Protected;
        }

        let entry_points = ["main", "async_main", "run", "start", "init", "setup"];
        if entry_points.contains(&func.name.as_str()) {
            return ProtectionLevel::Protected;
        }

        if self
            .framework_registry
            .is_framework_root(&language, &func.file, &func.name)
        {
            return ProtectionLevel::Protected;
        }

        if language == "python" && func.name.starts_with("__") && func.name.ends_with("__") {
            return ProtectionLevel::Protected;
        }

        // 2. LIKELY ALIVE - High confidence

        if let Some(doc) = &func.doc_comment {
            if self.framework_registry.is_dynamic_behavior(&language, doc) {
                return ProtectionLevel::LikelyAlive;
            }
        }

        if language == "javascript" || language == "typescript" {
            let is_jsx = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
            let is_component = is_jsx
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

            if func.name.starts_with("handle") || func.name.starts_with("on") {
                return ProtectionLevel::LikelyAlive;
            }
        }

        if func.is_public
            && (func.file.contains("lib.rs")
                || func.file.contains("mod.rs")
                || func.file.contains("lib/")
                || func.file.contains("src/"))
        {
            return ProtectionLevel::LikelyAlive;
        }

        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // 3. CANDIDATE
        ProtectionLevel::Candidate
    }
}

impl Default for UnifiedFilter {
    fn default() -> Self {
        Self::new()
    }
}
