// src/analysis/dead_code/filters/typescript.rs

//! TypeScript-specific dead code filters

use super::javascript::JavaScriptFilter;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct TypeScriptFilter;

impl LanguageFilter for TypeScriptFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // TypeScript shares most rules with JavaScript
        let js_filter = JavaScriptFilter;
        let level = js_filter.get_protection_level(func);

        // TypeScript-specific: interface methods are protected
        if level == ProtectionLevel::Candidate {
            if func.is_trait_method {
                return ProtectionLevel::Protected;
            }

            // TypeScript decorators
            if let Some(doc) = &func.doc_comment {
                let decorator_patterns = [
                    "@Get",
                    "@Post",
                    "@Put",
                    "@Delete",
                    "@Patch",
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
        }

        level
    }
}
