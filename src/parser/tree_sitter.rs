// src/parser/tree_sitter.rs

use crate::parser::languages::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::Parser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub path: String,
    pub language: String,
    pub functions: Vec<FunctionInfo>,
    pub imports: Vec<ImportInfo>,
    pub types: Vec<TypeInfo>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub params: Vec<ParamInfo>,
    pub return_type: Option<String>,
    pub doc_comment: Option<String>,
    pub calls: Vec<CallSite>,
    pub body_range: (usize, usize),
    pub body_start_line: usize,
    pub body_end_line: usize,
    pub complexity: usize,
    pub container: Option<String>,
    pub role: FunctionRole,
    pub purpose: String,
    pub trait_impl: Option<String>,
    pub decorators: Vec<String>,
    pub is_test: bool,
    pub is_trait_method: bool,
    pub is_trait_default: bool,
    pub variables: Vec<VariableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub type_hint: Option<String>,
    pub initializer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CallSite {
    SelfMethod(String),
    Qualified(String, String),
    OnReceiver(String, String),
    Chained(String),
    Bare(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FunctionRole {
    EntryPoint,
    Handler,
    Service,
    Repository,
    Utility,
    Validator,
    Factory,
    Converter,
    Middleware,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub type_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module: String,
    pub items: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
    Impl,
    TypeAlias,
    Interface,
    Class,
}

// PARSER IMPLEMENTATION

/// Main Tree-sitter parser
pub struct TreeSitterParser {
    languages: HashMap<String, LanguageConfig>,
}

impl std::fmt::Display for CallSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallSite::SelfMethod(m) => write!(f, "self.{}", m),
            CallSite::Qualified(q, m) => write!(f, "{}::{}", q, m),
            CallSite::OnReceiver(r, m) => write!(f, "{}.{}", r, m),
            CallSite::Chained(m) => write!(f, "().{}", m),
            CallSite::Bare(n) => write!(f, "{}", n),
        }
    }
}

impl TreeSitterParser {
    pub fn new() -> Self {
        Self {
            languages: get_all_language_configs(),
        }
    }

    fn detect_language(&self, path: &Path) -> Option<LanguageConfig> {
        let ext = path.extension()?.to_str()?;
        self.languages.get(ext).cloned()
    }

    pub fn parse_file(&self, path: &Path) -> Result<ParsedFile, String> {
        let config = self
            .detect_language(path)
            .ok_or_else(|| format!("Unsupported file: {:?}", path))?;

        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

        let mut parser = Parser::new();
        let language = (config.language_fn)();
        parser
            .set_language(&language)
            .map_err(|e| format!("Failed to set language: {}", e))?;

        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| "Failed to parse".to_string())?;

        let (functions, imports, types) = match config.name.as_str() {
            "Rust" => (
                rust::RustParser::extract_functions(&tree, &source),
                rust::RustParser::extract_imports(&tree, &source),
                rust::RustParser::extract_types(&tree, &source),
            ),
            "Python" => (
                python::PythonParser::extract_functions(&tree, &source),
                python::PythonParser::extract_imports(&tree, &source),
                python::PythonParser::extract_types(&tree, &source),
            ),
            "JavaScript" => (
                javascript::JavaScriptParser::extract_functions(&tree, &source),
                javascript::JavaScriptParser::extract_imports(&tree, &source),
                javascript::JavaScriptParser::extract_types(&tree, &source),
            ),
            "TypeScript" => (
                typescript::TypeScriptParser::extract_functions(&tree, &source),
                typescript::TypeScriptParser::extract_imports(&tree, &source),
                typescript::TypeScriptParser::extract_types(&tree, &source),
            ),
            "Go" => (
                go::GoParser::extract_functions(&tree, &source),
                go::GoParser::extract_imports(&tree, &source),
                go::GoParser::extract_types(&tree, &source),
            ),
            "Java" => (
                java::JavaParser::extract_functions(&tree, &source),
                java::JavaParser::extract_imports(&tree, &source),
                java::JavaParser::extract_types(&tree, &source),
            ),
            "Dart" => (
                dart::DartParser::extract_functions(&tree, &source),
                dart::DartParser::extract_imports(&tree, &source),
                dart::DartParser::extract_types(&tree, &source),
            ),
            "PHP" => (
                php::PhpParser::extract_functions(&tree, &source),
                php::PhpParser::extract_imports(&tree, &source),
                php::PhpParser::extract_types(&tree, &source),
            ),
            "CPP" => (
                cpp::CppParser::extract_functions(&tree, &source),
                cpp::CppParser::extract_imports(&tree, &source),
                cpp::CppParser::extract_types(&tree, &source),
            ),
            "CSharp" => (
                csharp::CSharpParser::extract_functions(&tree, &source),
                csharp::CSharpParser::extract_imports(&tree, &source),
                csharp::CSharpParser::extract_types(&tree, &source),
            ),
            _ => {
                return Err(format!("Unsupported language: {}", config.name));
            }
        };

        Ok(ParsedFile {
            path: path.to_string_lossy().to_string(),
            language: config.name,
            functions,
            imports,
            types,
            source,
        })
    }
}
