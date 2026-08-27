// src/resolution/languages/rust.rs

use crate::resolution::call_site::{CallSite, CalleeExpr};
use crate::resolution::context::{Language, ResolutionContext};
use crate::resolution::result::{
    ResolutionCandidate, ResolutionEvidence, ResolutionMethod, ResolutionResult,
};
use crate::resolution::symbol::{ImportBinding, SymbolId, TypeId};
use crate::resolution::traits::LanguageResolver;

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
            _ => ResolutionResult::unresolved(),
        }
    }

    fn resolve_type(&self, name: &str, context: &ResolutionContext) -> Vec<TypeId> {
        let mut results = Vec::new();
        for symbol in context.index.find_by_name(name) {
            results.push(TypeId(format!("type_{}", symbol.id.0)));
        }
        results
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

impl RustResolver {
    fn resolve_bare_name(&self, name: &str, context: &ResolutionContext) -> ResolutionResult {
        // Tier 1: Local scope chain
        if let Some(symbol_id) = context.scopes.resolve(&context.scope, name) {
            return ResolutionResult::resolved(
                symbol_id.clone(),
                1.0,
                ResolutionMethod::LexicalScope,
                vec![ResolutionEvidence::MatchingScope],
            );
        }

        // Tier 2: Same file
        let same_file = context.index.find_in_file(&context.file, name);
        if same_file.len() == 1 {
            return ResolutionResult::resolved(
                same_file[0].id.clone(),
                0.95,
                ResolutionMethod::LocalSymbol,
                vec![ResolutionEvidence::SameFile],
            );
        }

        // Tier 3: Import resolution
        if let Some(imports) = context.index.imports.get(&context.file) {
            for import in imports {
                let matches = import.local_name == name
                    || import.imported_name.as_deref() == Some(name)
                    || import.module.0.ends_with(&format!("::{}", name));

                if matches {
                    // Try to find the symbol in the imported module
                    let module_candidates = context
                        .index
                        .find_by_qualified(&format!("{}::{}", import.module.0, name));

                    if module_candidates.len() == 1 {
                        return ResolutionResult::resolved(
                            module_candidates[0].id.clone(),
                            0.90,
                            ResolutionMethod::ImportedSymbol,
                            vec![ResolutionEvidence::ExplicitImport],
                        );
                    }

                    // Fallback: search by name
                    let name_candidates = context.index.find_by_name(name);
                    if name_candidates.len() == 1 {
                        return ResolutionResult::resolved(
                            name_candidates[0].id.clone(),
                            0.85,
                            ResolutionMethod::ImportedSymbol,
                            vec![ResolutionEvidence::ExplicitImport],
                        );
                    }
                }
            }
        }

        // Tier 4: Global name fallback (unambiguous only)
        let global = context.index.find_by_name(name);
        if global.len() == 1 {
            return ResolutionResult::resolved(
                global[0].id.clone(),
                0.70,
                ResolutionMethod::GlobalNameFallback,
                vec![ResolutionEvidence::MatchingSymbol],
            );
        }

        if global.len() > 1 {
            let candidates: Vec<ResolutionCandidate> = global
                .iter()
                .map(|s| ResolutionCandidate {
                    symbol: s.id.clone(),
                    method: ResolutionMethod::GlobalNameFallback,
                    confidence: 0.40,
                    evidence: vec![ResolutionEvidence::MatchingSymbol],
                })
                .collect();

            return ResolutionResult::ambiguous(candidates);
        }

        ResolutionResult::unresolved()
    }

    fn resolve_qualified(&self, parts: &[String], context: &ResolutionContext) -> ResolutionResult {
        let joined = parts.join("::");

        // Tier 1: Direct qualified match
        let qualified_candidates = context.index.find_by_qualified(&joined);
        if qualified_candidates.len() == 1 {
            return ResolutionResult::resolved(
                qualified_candidates[0].id.clone(),
                0.95,
                ResolutionMethod::QualifiedSymbol,
                vec![ResolutionEvidence::MatchingSymbol],
            );
        }

        // Tier 2: Try file-based path conversion
        // e.g., "rust::RustParser" -> "./src/parser/languages/rust.rs::RustParser"
        let root = &parts[0];

        // Try to find the module/file for this root
        for (module_id, module) in &context.index.modules {
            let module_short = module_id.0.split("::").last().unwrap_or(&module_id.0);

            if module_short == root || module_id.0.ends_with(&format!("::{}", root)) {
                let file_path = &module.file.0;
                let rest = parts[1..].join("::");
                let file_based = if rest.is_empty() {
                    file_path.clone()
                } else {
                    format!("{}::{}", file_path, rest)
                };

                if let Some(target) = context.index.symbols.get(&SymbolId(file_based.clone())) {
                    return ResolutionResult::resolved(
                        target.id.clone(),
                        0.90,
                        ResolutionMethod::QualifiedSymbol,
                        vec![ResolutionEvidence::MatchingModule],
                    );
                }

                // Also try without the container (free function)
                let last = parts.last().unwrap();
                let simple_path = format!("{}::{}", file_path, last);
                if let Some(target) = context.index.symbols.get(&SymbolId(simple_path)) {
                    return ResolutionResult::resolved(
                        target.id.clone(),
                        0.85,
                        ResolutionMethod::QualifiedSymbol,
                        vec![ResolutionEvidence::MatchingModule],
                    );
                }
            }
        }

        // Tier 3: crate::, self::, super:: resolution
        let resolved_module = match root.as_str() {
            "crate" => Some(context.module.0.clone()),
            "self" => Some(context.module.0.clone()),
            "super" => {
                let mut base: Vec<&str> = context.module.0.split("::").collect();
                base.pop();
                Some(base.join("::"))
            }
            _ => None,
        };

        if let Some(module_path) = resolved_module {
            let rest = parts[1..].join("::");
            let full_path = if rest.is_empty() {
                module_path
            } else {
                format!("{}::{}", module_path, rest)
            };

            // Try to find the file for this module
            for (module_id, module) in &context.index.modules {
                if module_id.0 == full_path || module_id.0.starts_with(&full_path) {
                    let file_path = &module.file.0;
                    let last = parts.last().unwrap();
                    let file_based = format!("{}::{}", file_path, last);

                    if let Some(target) = context.index.symbols.get(&SymbolId(file_based)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.95,
                            ResolutionMethod::QualifiedSymbol,
                            vec![ResolutionEvidence::MatchingModule],
                        );
                    }

                    // Try with container
                    if parts.len() >= 3 {
                        let container = &parts[parts.len() - 2];
                        let method = &parts[parts.len() - 1];
                        let full = format!("{}::{}::{}", file_path, container, method);
                        if let Some(target) = context.index.symbols.get(&SymbolId(full)) {
                            return ResolutionResult::resolved(
                                target.id.clone(),
                                0.95,
                                ResolutionMethod::QualifiedSymbol,
                                vec![ResolutionEvidence::MatchingModule],
                            );
                        }
                    }
                }
            }
        }

        // Tier 4: Type::method resolution
        if parts.len() >= 2 {
            let type_name = &parts[parts.len() - 2];
            let method_name = &parts[parts.len() - 1];

            // Types aren't functions - look them up in the type index
            // (declared name -> declaring file), not the function symbol table.
            if let Some(files) = context.index.type_files.get(type_name.as_str()) {
                if files.len() == 1 {
                    let type_file = &files[0].0;
                    let candidate = format!("{}::{}::{}", type_file, type_name, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(candidate)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.85,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    // Try without container (associated function)
                    let simple = format!("{}::{}", type_file, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(simple)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.80,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }
                }
            }
        }

        ResolutionResult::unresolved()
    }

    fn resolve_member(
        &self,
        receiver: &CalleeExpr,
        member: &str,
        context: &ResolutionContext,
    ) -> ResolutionResult {
        // Resolve receiver name
        let receiver_name = match receiver {
            CalleeExpr::Name(name) => name.clone(),
            _ => return ResolutionResult::unresolved(),
        };

        // `self`/`this` isn't a symbol to look up - it IS the caller's own
        // container. Resolve it directly instead of searching the symbol
        // index for something literally named "self".
        if receiver_name == "self" || receiver_name == "this" {
            if let Some(caller) = context.index.symbols.get(&context.function) {
                if let Some(container) = &caller.container {
                    if let Some(members) = context.index.by_container.get(container) {
                        let matching: Vec<_> = members
                            .iter()
                            .filter_map(|id| context.index.symbols.get(id))
                            .filter(|s| s.name == member)
                            .collect();

                        if matching.len() == 1 {
                            return ResolutionResult::resolved(
                                matching[0].id.clone(),
                                0.95,
                                ResolutionMethod::ContainerMember,
                                vec![ResolutionEvidence::MatchingContainer],
                            );
                        }
                        if matching.len() > 1 {
                            let candidates = matching
                                .iter()
                                .map(|s| ResolutionCandidate {
                                    symbol: s.id.clone(),
                                    method: ResolutionMethod::ContainerMember,
                                    confidence: 0.5,
                                    evidence: vec![ResolutionEvidence::MatchingContainer],
                                })
                                .collect();
                            return ResolutionResult::ambiguous(candidates);
                        }
                    }
                }
            }

            // No known container, or member wasn't among its siblings - fall back.
            let same_file = context.index.find_in_file(&context.file, member);
            if same_file.len() == 1 {
                return ResolutionResult::resolved(
                    same_file[0].id.clone(),
                    0.80,
                    ResolutionMethod::LocalSymbol,
                    vec![ResolutionEvidence::SameFile],
                );
            }
            return ResolutionResult::unresolved();
        }

        // Try to find the receiver symbol and look for member in same container
        let receiver_candidates = context.index.find_by_name(&receiver_name);
        if receiver_candidates.len() == 1 {
            let receiver_symbol = &receiver_candidates[0];
            if let Some(container) = &receiver_symbol.container {
                if let Some(members) = context.index.by_container.get(container) {
                    let matching: Vec<_> = members
                        .iter()
                        .filter_map(|id| context.index.symbols.get(id))
                        .filter(|s| s.name == member)
                        .collect();

                    if matching.len() == 1 {
                        return ResolutionResult::resolved(
                            matching[0].id.clone(),
                            0.90,
                            ResolutionMethod::ContainerMember,
                            vec![ResolutionEvidence::MatchingContainer],
                        );
                    }
                }
            }

            // Try same file
            let same_file = context.index.find_in_file(&receiver_symbol.file, member);
            if same_file.len() == 1 {
                return ResolutionResult::resolved(
                    same_file[0].id.clone(),
                    0.80,
                    ResolutionMethod::LocalSymbol,
                    vec![ResolutionEvidence::SameFile],
                );
            }
        }

        // Fallback: find member by name in same file
        let same_file = context.index.find_in_file(&context.file, member);
        if same_file.len() == 1 {
            return ResolutionResult::resolved(
                same_file[0].id.clone(),
                0.75,
                ResolutionMethod::LocalSymbol,
                vec![ResolutionEvidence::SameFile],
            );
        }

        ResolutionResult::unresolved()
    }
}
