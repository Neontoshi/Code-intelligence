// src/resolution/languages/php.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct PhpResolver;

impl LanguageResolver for PhpResolver {
    fn language(&self) -> Language {
        Language::Php
    }
}
