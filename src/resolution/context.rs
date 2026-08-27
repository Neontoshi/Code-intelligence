// src/resolution/context.rs

use crate::resolution::scope::ScopeChain;
use crate::resolution::symbol::{FileId, ModuleId, ScopeId, SymbolId, SymbolIndex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Dart,
    Php,
    Cpp,
    CSharp,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Go => "go",
            Language::Java => "java",
            Language::Dart => "dart",
            Language::Php => "php",
            Language::Cpp => "cpp",
            Language::CSharp => "csharp",
        }
    }

    pub fn from_file_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "js" | "jsx" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "dart" => Some(Language::Dart),
            "php" => Some(Language::Php),
            "cpp" | "cc" | "cxx" | "hpp" | "h" => Some(Language::Cpp),
            "cs" => Some(Language::CSharp),
            _ => None,
        }
    }
}

pub struct ResolutionContext<'a> {
    pub file: FileId,
    pub function: SymbolId,
    pub scope: ScopeId,
    pub language: Language,
    pub module: ModuleId,
    pub index: &'a SymbolIndex,
    pub scopes: &'a ScopeChain,
}
