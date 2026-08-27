// src/resolution/mod.rs

pub mod call_site;
pub mod context;
pub mod index_builder;
pub mod languages;
pub mod result;
pub mod scope;
pub mod symbol;
pub mod traits;

pub use call_site::{CallKind, CallSite, CalleeExpr, SourceLocation};
pub use context::{Language, ResolutionContext};
pub use index_builder::IndexBuilder;
pub use result::{
    ResolutionCandidate, ResolutionEvidence, ResolutionMethod, ResolutionResult, ResolutionStatus,
};
pub use scope::{Scope, ScopeChain};
pub use symbol::{
    FileId, ImportBinding, ImportKind, Module, ModuleId, ScopeId, Symbol, SymbolId, SymbolIndex,
    SymbolKind, TypeId, Visibility,
};
pub use traits::LanguageResolver;

use std::collections::HashMap;
use std::sync::Arc;

pub struct ResolutionEngine {
    pub index: SymbolIndex,
    pub scopes: ScopeChain,
    pub resolvers: HashMap<Language, Arc<dyn LanguageResolver>>,
}

impl ResolutionEngine {
    pub fn new() -> Self {
        Self {
            index: SymbolIndex::new(),
            scopes: ScopeChain::new(),
            resolvers: languages::get_all_resolvers(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.index.add_symbol(symbol);
    }

    pub fn add_scope(&mut self, scope: Scope) {
        self.scopes.add_scope(scope);
    }

    pub fn resolve_call(
        &self,
        call: &CallSite,
        file: &FileId,
        function: &SymbolId,
        scope: &ScopeId,
        language: &Language,
        module: &ModuleId,
    ) -> ResolutionResult {
        let context = ResolutionContext {
            file: file.clone(),
            function: function.clone(),
            scope: scope.clone(),
            language: language.clone(),
            module: module.clone(),
            index: &self.index,
            scopes: &self.scopes,
        };

        if let Some(resolver) = self.resolvers.get(language) {
            return resolver.resolve_call(call, &context);
        }

        ResolutionResult::unresolved()
    }
}

impl Default for ResolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}
