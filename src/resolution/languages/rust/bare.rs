use crate::resolution::context::ResolutionContext;
use crate::resolution::result::{
    ResolutionCandidate, ResolutionDebugInfo, ResolutionEvidence, ResolutionMethod,
    ResolutionResult, UnresolvedReason,
};
use crate::resolution::symbol::{ImportKind, SymbolId, SymbolKind};
use crate::resolution::ResolutionStatus;

use super::RustResolver;

impl RustResolver {
    pub(super) fn resolve_bare_name(
        &self,
        name: &str,
        context: &ResolutionContext,
    ) -> ResolutionResult {
        let mut debug = ResolutionDebugInfo {
            query: Some(name.to_string()),
            scope_checked: true,
            same_file_candidate_count: 0,
            import_candidate_count: 0,
            wildcard_candidate_count: 0,
            global_candidate_count: 0,
            container_candidate_count: 0,
            notes: Vec::new(),
        };

        if let Some(symbol_id) = context.scopes.resolve(&context.scope, name) {
            if symbol_id.0.starts_with("import::") {
                debug.notes.push(format!(
                    "scope lookup hit imported alias {}; deferring to import resolution",
                    name
                ));
            } else if symbol_id.0.contains("::param::") || symbol_id.0.contains("::local::") {
                debug.notes.push(format!(
                    "scope lookup hit local callable binding {}",
                    symbol_id.0
                ));
                return ResolutionResult::callback(&format!(
                    "call through local or parameter binding: {}",
                    symbol_id.0
                ))
                .with_debug(debug);
            } else {
                return ResolutionResult::resolved(
                    symbol_id.clone(),
                    1.0,
                    ResolutionMethod::LexicalScope,
                    vec![ResolutionEvidence::MatchingScope],
                );
            }
        }
        debug.notes.push("scope lookup missed".to_string());

        let same_file = context.index.find_in_file(&context.file, name);
        debug.same_file_candidate_count = same_file.len();
        if same_file.len() == 1 {
            return ResolutionResult::resolved(
                same_file[0].id.clone(),
                0.95,
                ResolutionMethod::LocalSymbol,
                vec![ResolutionEvidence::SameFile],
            );
        }
        if same_file.len() > 1 {
            let callable_same_file: Vec<_> = same_file
                .iter()
                .filter(|s| {
                    matches!(
                        s.kind,
                        SymbolKind::Function
                            | SymbolKind::Constructor
                            | SymbolKind::AssociatedFunction
                            | SymbolKind::StaticMethod
                            | SymbolKind::ClassMethod
                    )
                })
                .collect();
            if callable_same_file.len() == 1 {
                debug.notes.push(
                    "same-file duplicate name disambiguated in favor of callable free/associated function"
                        .to_string(),
                );
                return ResolutionResult::resolved(
                    callable_same_file[0].id.clone(),
                    0.93,
                    ResolutionMethod::LocalSymbol,
                    vec![ResolutionEvidence::SameFile],
                );
            }
            return ResolutionResult::unresolved_with_reason(UnresolvedReason::SameFileAmbiguous)
                .with_debug(debug);
        }

        if let Some(imports) = context.index.imports.get(&context.file) {
            let mut import_candidates = Vec::new();
            for import in imports {
                let imported_name_match = import.imported_name.as_deref() == Some(name);
                let local_name_match = import.local_name == name;
                let matches = local_name_match
                    || imported_name_match
                    || import.module.0.ends_with(&format!("::{}", name));

                if !matches {
                    continue;
                }

                if let Some(symbol) = &import.symbol {
                    return ResolutionResult::resolved(
                        symbol.clone(),
                        0.92,
                        ResolutionMethod::ImportedSymbol,
                        vec![ResolutionEvidence::ExplicitImport],
                    );
                }

                let imported_symbol_name = import.imported_name.as_deref().unwrap_or(name);
                let qualified_candidates = [
                    format!("{}::{}", import.module.0, imported_symbol_name),
                    format!("{}::{}", import.module.0, name),
                ];

                for qualified in qualified_candidates {
                    let module_candidates = context.index.find_by_qualified(&qualified);
                    if module_candidates.len() == 1 {
                        return ResolutionResult::resolved(
                            module_candidates[0].id.clone(),
                            0.90,
                            ResolutionMethod::ImportedSymbol,
                            vec![ResolutionEvidence::ExplicitImport],
                        );
                    }
                    import_candidates.extend(module_candidates.into_iter().map(|s| s.id.clone()));
                }

                let name_candidates: Vec<_> = context
                    .index
                    .find_by_name(imported_symbol_name)
                    .into_iter()
                    .filter(|candidate| {
                        let module_matches = candidate
                            .module
                            .as_ref()
                            .map(|m| {
                                m.0 == import.module.0
                                    || m.0.starts_with(&format!("{}::", import.module.0))
                            })
                            .unwrap_or(false);

                        let id_matches = candidate.id.0.contains(&import.module.0);
                        module_matches || id_matches
                    })
                    .collect();

                if name_candidates.len() == 1 {
                    return ResolutionResult::resolved(
                        name_candidates[0].id.clone(),
                        0.87,
                        ResolutionMethod::ImportedSymbol,
                        vec![ResolutionEvidence::ExplicitImport],
                    );
                }

                import_candidates.extend(name_candidates.into_iter().map(|s| s.id.clone()));
            }
            import_candidates.sort();
            import_candidates.dedup();
            debug.import_candidate_count = import_candidates.len();
            if import_candidates.len() == 1 {
                return ResolutionResult::resolved(
                    import_candidates[0].clone(),
                    0.84,
                    ResolutionMethod::ImportedSymbol,
                    vec![ResolutionEvidence::ExplicitImport],
                );
            }
            if import_candidates.len() > 1 {
                return ResolutionResult::unresolved_with_reason(UnresolvedReason::ImportAmbiguous)
                    .with_debug(debug);
            }
        }

        if let Some(imports) = context.index.imports.get(&context.file) {
            let mut wildcard_matches = Vec::new();
            for import in imports {
                if import.kind != ImportKind::Wildcard {
                    continue;
                }

                let module_path = import
                    .module
                    .0
                    .trim_end_matches("::*")
                    .trim_end_matches('*')
                    .trim_end_matches("::");

                if let Some(rest) = module_path.strip_prefix("crate::") {
                    let path = rest.replace("::", "/");
                    let prefix = if context.file.0.starts_with("./") {
                        "./"
                    } else {
                        ""
                    };
                    for file_path in [
                        format!("{}src/{}.rs", prefix, path),
                        format!("{}src/{}/mod.rs", prefix, path),
                    ] {
                        let candidate = SymbolId(format!("{}::{}", file_path, name));
                        if let Some(target) = context.index.symbols.get(&candidate) {
                            if target.kind == SymbolKind::Function {
                                wildcard_matches.push(target.id.clone());
                            }
                        }
                    }
                }
            }
            wildcard_matches.sort();
            wildcard_matches.dedup();
            debug.wildcard_candidate_count = wildcard_matches.len();
            if wildcard_matches.len() == 1 {
                return ResolutionResult::resolved(
                    wildcard_matches[0].clone(),
                    0.92,
                    ResolutionMethod::ImportedSymbol,
                    vec![ResolutionEvidence::ExplicitImport],
                );
            }
            if wildcard_matches.len() > 1 {
                return ResolutionResult::unresolved_with_reason(
                    UnresolvedReason::WildcardImportAmbiguous,
                )
                .with_debug(debug);
            }
        }

        if name.contains("::<") {
            return ResolutionResult::external();
        }

        let common_methods = ["cosine_similarity", "feature_names", "feature_count"];
        if common_methods.contains(&name) {
            return ResolutionResult::external();
        }

        let global = context.index.find_by_name(name);
        debug.global_candidate_count = global.len();
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

            let mut result = ResolutionResult::ambiguous(candidates);
            result.debug = Some(debug.clone());
            return result;
        }

        let type_aliases = ["FileId", "ModuleId", "ScopeId", "SymbolId", "TypeId"];
        if type_aliases.contains(&name) {
            if let Some(files) = context.index.type_files.get(name) {
                if !files.is_empty() {
                    let file_path = &files[0].0;
                    let constructor_path = format!("{}::{}", file_path, name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(constructor_path)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.95,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }
                }
            }
            return ResolutionResult::external();
        }

        let same_file_candidates = context.index.find_in_file(&context.file, name);
        if !same_file_candidates.is_empty() && same_file_candidates.len() == 1 {
            return ResolutionResult::resolved(
                same_file_candidates[0].id.clone(),
                0.90,
                ResolutionMethod::LocalSymbol,
                vec![ResolutionEvidence::SameFile],
            );
        }

        let all_candidates = context.index.find_by_name(name);
        if all_candidates.len() == 1 {
            return ResolutionResult::resolved(
                all_candidates[0].id.clone(),
                0.80,
                ResolutionMethod::GlobalNameFallback,
                vec![ResolutionEvidence::MatchingSymbol],
            );
        }

        ResolutionResult::unresolved_with_reason(UnresolvedReason::NoCandidates).with_debug(debug)
    }

    pub(super) fn resolve_chained_method(
        &self,
        name: &str,
        context: &ResolutionContext,
    ) -> ResolutionResult {
        let result = self.resolve_bare_name(name, context);
        if matches!(result.status, ResolutionStatus::Unresolved) {
            return ResolutionResult::external();
        }
        result
    }
}
