// src/resolution/traits.rs

use crate::resolution::call_site::CallSite;
use crate::resolution::context::{Language, ResolutionContext};
use crate::resolution::result::ResolutionResult;
use crate::resolution::symbol::{ImportBinding, SymbolId, TypeId};

pub trait LanguageResolver: Send + Sync {
    fn language(&self) -> Language;

    fn resolve_import(
        &self,
        _import: &ImportBinding,
        _context: &ResolutionContext,
    ) -> Vec<SymbolId> {
        Vec::new()
    }

    fn resolve_call(&self, call: &CallSite, context: &ResolutionContext) -> ResolutionResult {
        crate::resolution::generic::resolve_call(call, context)
    }

    fn resolve_type(&self, _name: &str, _context: &ResolutionContext) -> Vec<TypeId> {
        Vec::new()
    }

    fn resolve_module(&self, _module: &str, _context: &ResolutionContext) -> Vec<String> {
        Vec::new()
    }
}
