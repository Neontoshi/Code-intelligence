// src/resolution/scope.rs

use crate::resolution::symbol::{ScopeId, SymbolId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub symbols: HashMap<String, SymbolId>,
}

impl Scope {
    pub fn new(id: ScopeId, parent: Option<ScopeId>) -> Self {
        Self {
            id,
            parent,
            symbols: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol: SymbolId) {
        self.symbols.insert(name, symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolId> {
        self.symbols.get(name)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScopeChain {
    pub scopes: HashMap<ScopeId, Scope>,
}

impl ScopeChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_scope(&mut self, scope: Scope) {
        self.scopes.insert(scope.id.clone(), scope);
    }

    pub fn resolve(&self, start: &ScopeId, name: &str) -> Option<&SymbolId> {
        let mut current = Some(start);

        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get(scope_id) {
                if let Some(symbol) = scope.lookup(name) {
                    return Some(symbol);
                }
                current = scope.parent.as_ref();
            } else {
                break;
            }
        }

        None
    }
}
