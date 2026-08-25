// src/parser/languages/cpp.rs

//! C++ language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct CppParser;

impl CppParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "CPP".to_string(),
            extensions: vec![
                "cpp".to_string(),
                "cc".to_string(),
                "cxx".to_string(),
                "hpp".to_string(),
                "h".to_string(),
            ],
            language_fn: || tree_sitter_cpp::LANGUAGE.into(),
            function_kinds: vec![
                "function_definition".to_string(),
                "declaration".to_string(),
                "template_declaration".to_string(),
                "field_declaration".to_string(),
            ],
            import_kinds: vec![
                "preproc_include".to_string(),
                "using_declaration".to_string(),
            ],
            type_kinds: vec![
                "class_specifier".to_string(),
                "struct_specifier".to_string(),
                "enum_specifier".to_string(),
                "type_definition".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "CPP",
            function_kinds: vec![
                "function_definition",
                "declaration",
                "template_declaration",
                "field_declaration",
            ],
            import_kinds: vec!["preproc_include", "using_declaration"],
            type_kinds: vec![
                "class_specifier",
                "struct_specifier",
                "enum_specifier",
                "type_definition",
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
