// src/parser/languages/rust.rs

//! Rust language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct RustParser;

impl RustParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "Rust".to_string(),
            extensions: vec!["rs".to_string()],
            language_fn: || tree_sitter_rust::LANGUAGE.into(),
            function_kinds: vec!["function_item".to_string(), "method_item".to_string()],
            import_kinds: vec!["use_declaration".to_string()],
            type_kinds: vec![
                "struct_item".to_string(),
                "enum_item".to_string(),
                "trait_item".to_string(),
                "impl_item".to_string(),
                "type_alias".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "Rust",
            function_kinds: vec!["function_item", "method_item"],
            import_kinds: vec!["use_declaration"],
            type_kinds: vec![
                "struct_item",
                "enum_item",
                "trait_item",
                "impl_item",
                "type_alias",
            ],
            has_attributes: false,
            go_export_rules: false,
            has_decorators: false,
            has_export_statements: false,
        }
    }

    pub fn extract_functions(tree: &Tree, source: &str) -> Vec<FunctionInfo> {
        let config = Self::parser_config();
        SharedParser::extract_functions(tree, source, &config)
    }

    pub fn extract_imports(tree: &Tree, source: &str) -> Vec<ImportInfo> {
        let config = Self::parser_config();
        SharedParser::extract_imports(tree, source, &config)
    }

    pub fn extract_types(tree: &Tree, source: &str) -> Vec<TypeInfo> {
        let config = Self::parser_config();
        SharedParser::extract_types(tree, source, &config)
    }
}
