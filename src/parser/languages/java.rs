// src/parser/languages/java.rs

//! Java language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct JavaParser;

impl JavaParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "Java".to_string(),
            extensions: vec!["java".to_string()],
            language_fn: || tree_sitter_java::LANGUAGE.into(),
            function_kinds: vec![
                "method_declaration".to_string(),
                "constructor_declaration".to_string(),
                "lambda_expression".to_string(),
            ],
            import_kinds: vec!["import_declaration".to_string()],
            type_kinds: vec![
                "class_declaration".to_string(),
                "interface_declaration".to_string(),
                "enum_declaration".to_string(),
                "record_declaration".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "Java",
            function_kinds: vec![
                "method_declaration",
                "constructor_declaration",
                "lambda_expression",
            ],
            import_kinds: vec!["import_declaration"],
            type_kinds: vec![
                "class_declaration",
                "interface_declaration",
                "enum_declaration",
                "record_declaration",
            ],
            has_attributes: true,
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
