// src/resolution/languages/typescript.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct TypeScriptResolver;

impl LanguageResolver for TypeScriptResolver {
    fn language(&self) -> Language {
        Language::TypeScript
    }
}
