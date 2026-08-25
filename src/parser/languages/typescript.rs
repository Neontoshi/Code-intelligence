// src/parser/languages/typescript.rs

//! TypeScript language parser implementation (TS + TSX)

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct TypeScriptParser;

impl TypeScriptParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "TypeScript".to_string(),
            extensions: vec!["ts".to_string(), "tsx".to_string()],
            language_fn: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            function_kinds: vec![
                "function_declaration".to_string(),
                "function_expression".to_string(),
                "arrow_function".to_string(),
                "method_definition".to_string(),
                "generator_function_declaration".to_string(),
                "function".to_string(),
                "lexical_declaration".to_string(),
                "variable_declaration".to_string(),
                "variable_declarator".to_string(),
                "export_statement".to_string(),
                "export_default".to_string(),
                "class_declaration".to_string(),
                "class".to_string(),
                "method_definition".to_string(),
            ],
            import_kinds: vec![
                "import_statement".to_string(),
                "import".to_string(),
                "export_statement".to_string(),
                "export".to_string(),
            ],
            type_kinds: vec![
                "class_declaration".to_string(),
                "interface_declaration".to_string(),
                "type_alias_declaration".to_string(),
                "enum_declaration".to_string(),
                "type_parameter".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "TypeScript",
            function_kinds: vec![
                "function_declaration",
                "function_expression",
                "arrow_function",
                "method_definition",
                "generator_function_declaration",
                "function",
                "lexical_declaration",
                "variable_declaration",
                "variable_declarator",
                "export_statement",
                "export_default",
                "class_declaration",
                "class",
                "method_definition",
            ],
            import_kinds: vec!["import_statement", "import", "export_statement", "export"],
            type_kinds: vec![
                "class_declaration",
                "interface_declaration",
                "type_alias_declaration",
                "enum_declaration",
                "type_parameter",
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
