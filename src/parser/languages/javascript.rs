// src/parser/languages/javascript.rs

//! JavaScript language parser implementation (JS + JSX)

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct JavaScriptParser;

impl JavaScriptParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "JavaScript".to_string(),
            extensions: vec!["js".to_string(), "jsx".to_string()],
            language_fn: || tree_sitter_javascript::LANGUAGE.into(),
            function_kinds: vec![
                "function_declaration".to_string(),
                "function_expression".to_string(),
                "arrow_function".to_string(),
                "method_definition".to_string(),
                "generator_function_declaration".to_string(),
            ],
            import_kinds: vec![
                "import_statement".to_string(),
                "export_statement".to_string(),
            ],
            type_kinds: vec!["class_declaration".to_string()],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "JavaScript",
            function_kinds: vec![
                "function_declaration",
                "function_expression",
                "arrow_function",
                "method_definition",
                "generator_function_declaration",
            ],
            import_kinds: vec!["import_statement", "export_statement"],
            type_kinds: vec!["class_declaration"],
            branch_kinds: vec![
                "if_statement",
                "for_statement",
                "for_in_statement",
                "while_statement",
                "do_statement",
                "switch_case",
                "catch_clause",
                "ternary_expression",
            ],
            has_attributes: false,
            go_export_rules: false,
            has_decorators: false,
            has_export_statements: true,
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
