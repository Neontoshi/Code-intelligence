// src/resolution/languages/mod.rs

pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod javascript;
pub mod php;
pub mod python;
pub mod rust;
pub mod typescript;

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;
use std::collections::HashMap;
use std::sync::Arc;

pub fn get_all_resolvers() -> HashMap<Language, Arc<dyn LanguageResolver>> {
    let mut resolvers: HashMap<Language, Arc<dyn LanguageResolver>> = HashMap::new();

    resolvers.insert(Language::Rust, Arc::new(rust::RustResolver));
    resolvers.insert(Language::Python, Arc::new(python::PythonResolver));
    resolvers.insert(
        Language::JavaScript,
        Arc::new(javascript::JavaScriptResolver),
    );
    resolvers.insert(
        Language::TypeScript,
        Arc::new(typescript::TypeScriptResolver),
    );
    resolvers.insert(Language::Go, Arc::new(go::GoResolver));
    resolvers.insert(Language::Java, Arc::new(java::JavaResolver));
    resolvers.insert(Language::Dart, Arc::new(dart::DartResolver));
    resolvers.insert(Language::Php, Arc::new(php::PhpResolver));
    resolvers.insert(Language::Cpp, Arc::new(cpp::CppResolver));
    resolvers.insert(Language::CSharp, Arc::new(csharp::CSharpResolver));

    resolvers
}
