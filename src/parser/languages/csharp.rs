// src/parser/languages/csharp.rs

//! C# language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct CSharpParser;

impl CSharpParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "CSharp".to_string(),
            extensions: vec!["cs".to_string()],
            language_fn: || tree_sitter_c_sharp::LANGUAGE.into(),
            function_kinds: vec![
                "method_declaration".to_string(),
                "constructor_declaration".to_string(),
                "destructor_declaration".to_string(),
                "local_function_statement".to_string(),
                "operator_declaration".to_string(),
                "conversion_operator_declaration".to_string(),
                "property_declaration".to_string(),
                "indexer_declaration".to_string(),
                "arrow_expression_clause".to_string(),
            ],
            import_kinds: vec!["using_directive".to_string()],
            type_kinds: vec![
                "class_declaration".to_string(),
                "interface_declaration".to_string(),
                "struct_declaration".to_string(),
                "enum_declaration".to_string(),
                "record_declaration".to_string(),
                "record_struct_declaration".to_string(),
                "namespace_declaration".to_string(),
                "file_scoped_namespace_declaration".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "CSharp",
            function_kinds: vec![
                "method_declaration",
                "constructor_declaration",
                "destructor_declaration",
                "local_function_statement",
                "operator_declaration",
                "conversion_operator_declaration",
                "property_declaration",
                "indexer_declaration",
                "arrow_expression_clause",
            ],
            import_kinds: vec!["using_directive"],
            type_kinds: vec![
                "class_declaration",
                "interface_declaration",
                "struct_declaration",
                "enum_declaration",
                "record_declaration",
                "record_struct_declaration",
                "namespace_declaration",
                "file_scoped_namespace_declaration",
            ],
            branch_kinds: vec![
                "if_statement",
                "for_statement",
                "foreach_statement",
                "while_statement",
                "do_statement",
                "switch_section",
                "catch_clause",
                "conditional_expression",
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
