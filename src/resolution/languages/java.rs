// src/resolution/languages/java.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct JavaResolver;

impl LanguageResolver for JavaResolver {
    fn language(&self) -> Language {
        Language::Java
    }
}
