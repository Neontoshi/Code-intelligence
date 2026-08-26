// src/parser/languages/php.rs

//! PHP language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct PhpParser;

impl PhpParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "PHP".to_string(),
            extensions: vec!["php".to_string()],
            language_fn: || tree_sitter_php::LANGUAGE_PHP.into(),
            function_kinds: vec![
                "function_definition".to_string(),
                "method_declaration".to_string(),
                "arrow_function".to_string(),
                "anonymous_function_creation_expression".to_string(),
            ],
            import_kinds: vec![
                "namespace_use_declaration".to_string(),
                "require_expression".to_string(),
                "include_expression".to_string(),
            ],
            type_kinds: vec![
                "class_declaration".to_string(),
                "interface_declaration".to_string(),
                "trait_declaration".to_string(),
                "enum_declaration".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "PHP",
            function_kinds: vec![
                "function_definition",
                "method_declaration",
                "arrow_function",
                "anonymous_function_creation_expression",
            ],
            import_kinds: vec![
                "namespace_use_declaration",
                "require_expression",
                "include_expression",
            ],
            type_kinds: vec![
                "class_declaration",
                "interface_declaration",
                "trait_declaration",
                "enum_declaration",
            ],
            branch_kinds: vec![
                "if_statement",
                "for_statement",
                "foreach_statement",
                "while_statement",
                "do_statement",
                "switch_statement",
                "catch_clause",
                "conditional_expression",
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
