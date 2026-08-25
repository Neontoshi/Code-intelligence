// src/parser/languages/go.rs

//! Go language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct GoParser;

impl GoParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "Go".to_string(),
            extensions: vec!["go".to_string()],
            language_fn: || tree_sitter_go::LANGUAGE.into(),
            function_kinds: vec![
                "function_declaration".to_string(),
                "method_declaration".to_string(),
            ],
            import_kinds: vec!["import_declaration".to_string()],
            type_kinds: vec!["type_declaration".to_string()],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "Go",
            function_kinds: vec!["function_declaration", "method_declaration"],
            import_kinds: vec!["import_declaration"],
            type_kinds: vec!["type_declaration"],
            has_attributes: false,
            go_export_rules: true,
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
