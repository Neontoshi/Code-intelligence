// src/resolution/languages/dart.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct DartResolver;

impl LanguageResolver for DartResolver {
    fn language(&self) -> Language {
        Language::Dart
    }
}
