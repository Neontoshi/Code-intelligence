// src/resolution/languages/javascript.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct JavaScriptResolver;

impl LanguageResolver for JavaScriptResolver {
    fn language(&self) -> Language {
        Language::JavaScript
    }
}
