// src/resolution/languages/python.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct PythonResolver;

impl LanguageResolver for PythonResolver {
    fn language(&self) -> Language {
        Language::Python
    }
}
