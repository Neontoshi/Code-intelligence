// src/resolution/languages/csharp.rs

use crate::resolution::context::Language;
use crate::resolution::traits::LanguageResolver;

pub struct CSharpResolver;

impl LanguageResolver for CSharpResolver {
    fn language(&self) -> Language {
        Language::CSharp
    }
}
