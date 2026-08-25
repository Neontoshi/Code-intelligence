// src/analysis/dead_code/filters/mod.rs

//! Language-specific dead code filters
//!
//! These filters determine which functions should never be considered dead
//! based on language-specific patterns and conventions.

pub mod common;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod javascript;
pub mod php;
pub mod protection;
pub mod python;
pub mod rust;
pub mod typescript;

pub use common::*;
pub use protection::*;

use crate::graph::call_graph::FunctionNode;
use std::collections::HashMap;

/// Protection level for a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtectionLevel {
    /// Protected - never considered dead, safe from automatic deletion
    Protected,
    /// Likely Alive - high confidence these are alive, but not mathematically guaranteed
    LikelyAlive,
    /// Candidate - may be dead, needs review
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

/// Trait for language-specific protection filters
pub trait LanguageFilter: Send + Sync {
    /// Get the protection level for a function
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel;
}

/// Main filter orchestrator
pub struct FilterOrchestrator {
    filters: HashMap<String, Box<dyn LanguageFilter>>,
}

impl FilterOrchestrator {
    pub fn new() -> Self {
        let mut filters: HashMap<String, Box<dyn LanguageFilter>> = HashMap::new();

        filters.insert("rust".to_string(), Box::new(rust::RustFilter));
        filters.insert("python".to_string(), Box::new(python::PythonFilter));
        filters.insert(
            "javascript".to_string(),
            Box::new(javascript::JavaScriptFilter),
        );
        filters.insert(
            "typescript".to_string(),
            Box::new(typescript::TypeScriptFilter),
        );
        filters.insert("go".to_string(), Box::new(go::GoFilter));
        filters.insert("java".to_string(), Box::new(java::JavaFilter));
        filters.insert("dart".to_string(), Box::new(dart::DartFilter));
        filters.insert("php".to_string(), Box::new(php::PhpFilter));
        filters.insert("cpp".to_string(), Box::new(cpp::CppFilter));
        filters.insert("csharp".to_string(), Box::new(csharp::CSharpFilter));

        Self { filters }
    }

    /// Get protection level for a function based on its language
    pub fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // First, check language-specific filters
        if let Some(filter) = self.filters.get(&func.file_language()) {
            return filter.get_protection_level(func);
        }

        // Fallback to generic protection
        get_generic_protection_level(func)
    }
}

impl Default for FilterOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Get protection level using generic rules (language-agnostic)
pub fn get_generic_protection_level(func: &FunctionNode) -> ProtectionLevel {
    // Test functions are protected
    if func.is_test {
        return ProtectionLevel::Protected;
    }

    // Trait methods are protected
    if func.is_trait_method || func.is_trait_default {
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

    // Functions with callers are likely alive
    if func.fan_in > 0 {
        return ProtectionLevel::LikelyAlive;
    }

    // Public API functions are likely alive (especially in libraries)
    if func.is_public && (func.file.contains("lib.rs") || func.file.contains("mod.rs")) {
        return ProtectionLevel::LikelyAlive;
    }

    // Default: candidate for dead code analysis
    ProtectionLevel::Candidate
}

/// Extension trait for FunctionNode to get language from file
trait FileLanguage {
    fn file_language(&self) -> String;
}

impl FileLanguage for FunctionNode {
    fn file_language(&self) -> String {
        // Detect language from file extension
        if self.file.ends_with(".rs") {
            "rust".to_string()
        } else if self.file.ends_with(".py") {
            "python".to_string()
        } else if self.file.ends_with(".js") || self.file.ends_with(".jsx") {
            "javascript".to_string()
        } else if self.file.ends_with(".ts") || self.file.ends_with(".tsx") {
            "typescript".to_string()
        } else if self.file.ends_with(".go") {
            "go".to_string()
        } else if self.file.ends_with(".java") {
            "java".to_string()
        } else if self.file.ends_with(".dart") {
            "dart".to_string()
        } else if self.file.ends_with(".php") {
            "php".to_string()
        } else if self.file.ends_with(".cpp")
            || self.file.ends_with(".cc")
            || self.file.ends_with(".cxx")
            || self.file.ends_with(".hpp")
            || self.file.ends_with(".h")
        {
            "cpp".to_string()
        } else if self.file.ends_with(".cs") {
            "csharp".to_string()
        } else {
            "unknown".to_string()
        }
    }
}
