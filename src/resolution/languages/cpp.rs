// src/resolution/languages/cpp.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct CppResolver;

impl LanguageResolver for CppResolver {
    fn language(&self) -> Language {
        Language::Cpp
    }
}
