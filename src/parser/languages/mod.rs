// src/parser/languages/mod.rs

//! Language-specific parser implementations for Tree-sitter

pub mod common;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod javascript;
pub mod php;
pub mod python;
pub mod rust;
pub mod shared;
pub mod typescript;

pub use common::*;
pub use cpp::CppParser;
pub use csharp::CSharpParser;
pub use dart::DartParser;
pub use go::GoParser;
pub use java::JavaParser;
pub use javascript::JavaScriptParser;
pub use php::PhpParser;
pub use python::PythonParser;
pub use rust::RustParser;
pub use shared::LanguageParserConfig;
pub use typescript::TypeScriptParser;

use std::collections::HashMap;
use tree_sitter::Language;

/// Configuration for a language
#[derive(Clone)]
pub struct LanguageConfig {
    pub name: String,
    pub extensions: Vec<String>,
    pub language_fn: fn() -> Language,
    pub function_kinds: Vec<String>,
    pub import_kinds: Vec<String>,
    pub type_kinds: Vec<String>,
}

/// Get all language configurations
pub fn get_all_language_configs() -> HashMap<String, LanguageConfig> {
    let mut configs = HashMap::new();

    // Rust
    configs.insert("rs".to_string(), rust::RustParser::config());

    // Python
    configs.insert("py".to_string(), python::PythonParser::config());

    // JavaScript
    configs.insert("js".to_string(), javascript::JavaScriptParser::config());
    configs.insert("jsx".to_string(), javascript::JavaScriptParser::config());

    // TypeScript
    configs.insert("ts".to_string(), typescript::TypeScriptParser::config());
    configs.insert("tsx".to_string(), typescript::TypeScriptParser::config());

    // Go
    configs.insert("go".to_string(), go::GoParser::config());

    // Java
    configs.insert("java".to_string(), java::JavaParser::config());

    // Dart
    configs.insert("dart".to_string(), dart::DartParser::config());

    // PHP
    configs.insert("php".to_string(), php::PhpParser::config());

    // C++
    configs.insert("cpp".to_string(), cpp::CppParser::config());
    configs.insert("cc".to_string(), cpp::CppParser::config());
    configs.insert("cxx".to_string(), cpp::CppParser::config());
    configs.insert("hpp".to_string(), cpp::CppParser::config());
    configs.insert("h".to_string(), cpp::CppParser::config());

    // C#
    configs.insert("cs".to_string(), csharp::CSharpParser::config());

    configs
}

/// Get a language config by extension
pub fn get_language_config(ext: &str) -> Option<LanguageConfig> {
    get_all_language_configs().get(ext).cloned()
}

/// Detect language from file path
pub fn detect_language(path: &std::path::Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    get_language_config(ext).map(|config| config.name)
}
