// src/parser/languages/python.rs

//! Python language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::Tree;

pub struct PythonParser;

impl PythonParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "Python".to_string(),
            extensions: vec!["py".to_string()],
            language_fn: || tree_sitter_python::LANGUAGE.into(),
            function_kinds: vec![
                "function_definition".to_string(),
                "async_function_definition".to_string(),
                "method_definition".to_string(),
            ],
            import_kinds: vec![
                "import_statement".to_string(),
                "import_from_statement".to_string(),
            ],
            type_kinds: vec!["class_definition".to_string()],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "Python",
            function_kinds: vec![
                "function_definition",
                "async_function_definition",
                "method_definition",
            ],
            import_kinds: vec!["import_statement", "import_from_statement"],
            type_kinds: vec!["class_definition"],
            has_attributes: false,
            go_export_rules: false,
            has_decorators: true,
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
