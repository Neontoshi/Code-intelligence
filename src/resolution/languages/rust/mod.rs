use crate::resolution::call_site::{CallSite, CalleeExpr};
use crate::resolution::context::{Language, ResolutionContext};
use crate::resolution::result::{ResolutionResult, UnresolvedReason};
use crate::resolution::symbol::{ImportBinding, SymbolId, TypeId};
use crate::resolution::traits::LanguageResolver;

mod bare;
mod helpers;
mod member;
mod qualified;

pub struct RustResolver;

impl LanguageResolver for RustResolver {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn resolve_import(&self, import: &ImportBinding, context: &ResolutionContext) -> Vec<SymbolId> {
        let mut results = Vec::new();

        if let Some(symbol) = &import.symbol {
            results.push(symbol.clone());
            return results;
        }

        if let Some(name) = &import.imported_name {
            let candidates = context.index.find_by_name(name);
            for candidate in candidates {
                results.push(candidate.id.clone());
            }
        } else {
            let candidates = context.index.find_by_name(&import.local_name);
            for candidate in candidates {
                results.push(candidate.id.clone());
            }
        }

        results
    }

    fn resolve_call(&self, call: &CallSite, context: &ResolutionContext) -> ResolutionResult {
        match &call.callee {
            CalleeExpr::Name(name) => self.resolve_bare_name(name, context),
            CalleeExpr::Qualified(parts) => self.resolve_qualified(parts, context),
            CalleeExpr::Member { receiver, member } => {
                self.resolve_member(receiver, member, context)
            }
            CalleeExpr::Unknown(name) => self.resolve_chained_method(name, context),
            _ => ResolutionResult::unresolved_with_reason(UnresolvedReason::UnsupportedCalleeShape),
        }
    }

    fn resolve_type(&self, name: &str, context: &ResolutionContext) -> Vec<TypeId> {
        context
            .index
            .type_name_to_id
            .get(name)
            .cloned()
            .into_iter()
            .collect()
    }

    fn resolve_module(&self, module: &str, context: &ResolutionContext) -> Vec<String> {
        let mut results = Vec::new();
        for (module_id, _) in &context.index.modules {
            if module_id.0 == module || module_id.0.ends_with(&format!("::{}", module)) {
                results.push(module_id.0.clone());
            }
        }
        results
    }
}
