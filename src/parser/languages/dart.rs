// src/parser/languages/dart.rs

//! Dart language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct DartParser;

impl DartParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "Dart".to_string(),
            extensions: vec!["dart".to_string()],
            language_fn: || tree_sitter_dart::LANGUAGE.into(),
            function_kinds: vec![
                "function_signature".to_string(),
                "method_signature".to_string(),
                "function_declaration".to_string(),
                "method_declaration".to_string(),
                "getter_signature".to_string(),
                "setter_signature".to_string(),
            ],
            import_kinds: vec!["import_or_export".to_string(), "library_import".to_string()],
            type_kinds: vec![
                "class_definition".to_string(),
                "mixin_declaration".to_string(),
                "extension_declaration".to_string(),
                "enum_declaration".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "Dart",
            function_kinds: vec![
                "function_signature",
                "method_signature",
                "function_declaration",
                "method_declaration",
                "getter_signature",
                "setter_signature",
            ],
            import_kinds: vec!["import_or_export", "library_import"],
            type_kinds: vec![
                "class_definition",
                "mixin_declaration",
                "extension_declaration",
                "enum_declaration",
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
