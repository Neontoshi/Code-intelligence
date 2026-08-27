// src/resolution/languages/go.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct GoResolver;

impl LanguageResolver for GoResolver {
    fn language(&self) -> Language {
        Language::Go
    }
}
