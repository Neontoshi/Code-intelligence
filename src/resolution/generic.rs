use crate::resolution::call_site::{CallSite, CalleeExpr};
use crate::resolution::context::ResolutionContext;
use crate::resolution::result::{
    ResolutionCandidate, ResolutionDebugInfo, ResolutionEvidence, ResolutionMethod,
    ResolutionResult, UnresolvedReason,
};
use crate::resolution::symbol::{ImportKind, SymbolId};

pub fn resolve_call(call: &CallSite, context: &ResolutionContext) -> ResolutionResult {
    match &call.callee {
        CalleeExpr::Name(name) => resolve_bare_name(name, context),
        CalleeExpr::Qualified(parts) => resolve_qualified(parts, context),
        CalleeExpr::Member { receiver, member } => resolve_member(receiver, member, context),
        CalleeExpr::Unknown(name) => resolve_unknown(name, context),
        _ => ResolutionResult::unresolved_with_reason(UnresolvedReason::UnsupportedCalleeShape),
    }
}

fn resolve_bare_name(name: &str, context: &ResolutionContext) -> ResolutionResult {
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
        return ResolutionResult::resolved(
            symbol_id.clone(),
            1.0,
            ResolutionMethod::LexicalScope,
            vec![ResolutionEvidence::MatchingScope],
        );
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
        return ResolutionResult::unresolved_with_reason(UnresolvedReason::SameFileAmbiguous)
            .with_debug(debug);
    }

    if let Some(imports) = context.index.imports.get(&context.file) {
        let mut explicit_matches = Vec::new();
        let mut wildcard_matches = Vec::new();

        for import in imports {
            let matches = import.local_name == name
                || import.imported_name.as_deref() == Some(name)
                || import.module.0.ends_with(&format!("::{}", name))
                || import.module.0.ends_with(&format!(".{}", name));

            if import.kind == ImportKind::Wildcard {
                wildcard_matches.extend(resolve_wildcard_import(import, name, context));
                continue;
            }

            if matches {
                if let Some(symbol) = &import.symbol {
                    explicit_matches.push(symbol.clone());
                }

                let qualified_candidates = collect_unique_symbol_ids(
                    context
                        .index
                        .find_by_qualified(&format!("{}::{}", import.module.0, name))
                        .into_iter()
                        .map(|s| s.id.clone()),
                );
                explicit_matches.extend(qualified_candidates);

                let global_name_matches = collect_unique_symbol_ids(
                    context
                        .index
                        .find_by_name(name)
                        .into_iter()
                        .map(|s| s.id.clone()),
                );
                if global_name_matches.len() == 1 {
                    explicit_matches.extend(global_name_matches);
                }
            }
        }

        explicit_matches = collect_unique_symbol_ids(explicit_matches.into_iter());
        wildcard_matches = collect_unique_symbol_ids(wildcard_matches.into_iter());
        debug.import_candidate_count = explicit_matches.len();
        debug.wildcard_candidate_count = wildcard_matches.len();

        if explicit_matches.len() == 1 {
            return ResolutionResult::resolved(
                explicit_matches[0].clone(),
                0.90,
                ResolutionMethod::ImportedSymbol,
                vec![ResolutionEvidence::ExplicitImport],
            );
        }
        if explicit_matches.len() > 1 {
            return ResolutionResult::unresolved_with_reason(UnresolvedReason::ImportAmbiguous)
                .with_debug(debug);
        }

        if wildcard_matches.len() == 1 {
            return ResolutionResult::resolved(
                wildcard_matches[0].clone(),
                0.85,
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
        result.debug = Some(debug);
        return result;
    }

    ResolutionResult::unresolved_with_reason(UnresolvedReason::NoCandidates).with_debug(debug)
}

fn resolve_qualified(parts: &[String], context: &ResolutionContext) -> ResolutionResult {
    let joined = parts.join("::");
    let mut debug = ResolutionDebugInfo {
        query: Some(joined.clone()),
        scope_checked: false,
        same_file_candidate_count: 0,
        import_candidate_count: 0,
        wildcard_candidate_count: 0,
        global_candidate_count: 0,
        container_candidate_count: 0,
        notes: Vec::new(),
    };

    let qualified = context.index.find_by_qualified(&joined);
    debug.global_candidate_count = qualified.len();
    if qualified.len() == 1 {
        return ResolutionResult::resolved(
            qualified[0].id.clone(),
            0.95,
            ResolutionMethod::QualifiedSymbol,
            vec![ResolutionEvidence::MatchingModule],
        );
    }

    if let Some(first) = parts.first() {
        if let Some(file_path) = context.index.module_aliases.get(first) {
            if let Some(last) = parts.last() {
                let candidate = SymbolId(format!("{}::{}", file_path, last));
                if context.index.symbols.contains_key(&candidate) {
                    return ResolutionResult::resolved(
                        candidate,
                        0.90,
                        ResolutionMethod::QualifiedSymbol,
                        vec![ResolutionEvidence::MatchingModule],
                    );
                }
                debug.notes.push(format!(
                    "module alias '{}' found but symbol '{}' was missing",
                    first, candidate.0
                ));
            }
        } else {
            debug.notes.push(format!("no module alias for '{}'", first));
        }
    }

    ResolutionResult::unresolved_with_reason(UnresolvedReason::QualifiedPathMiss).with_debug(debug)
}

fn resolve_member(
    receiver: &CalleeExpr,
    member: &str,
    context: &ResolutionContext,
) -> ResolutionResult {
    match receiver {
        CalleeExpr::Name(name) if name == "self" || name == "this" || name == "cls" => {
            resolve_container_member(member, context)
        }
        CalleeExpr::Name(name) => {
            if let Some(container_matches) = resolve_member_on_named_receiver(name, member, context)
            {
                return container_matches;
            }
            resolve_bare_name(member, context)
        }
        _ => resolve_bare_name(member, context),
    }
}

fn resolve_unknown(name: &str, context: &ResolutionContext) -> ResolutionResult {
    let mut result = resolve_bare_name(name, context);
    if result.reason.is_none() {
        result = result.with_reason(UnresolvedReason::UnsupportedCalleeShape);
    }
    result
}

fn resolve_container_member(member: &str, context: &ResolutionContext) -> ResolutionResult {
    let current_symbol = match context.index.symbols.get(&context.function) {
        Some(symbol) => symbol,
        None => {
            return ResolutionResult::unresolved_with_reason(UnresolvedReason::MissingCurrentSymbol)
                .with_debug(ResolutionDebugInfo {
                    query: Some(member.to_string()),
                    scope_checked: false,
                    same_file_candidate_count: 0,
                    import_candidate_count: 0,
                    wildcard_candidate_count: 0,
                    global_candidate_count: 0,
                    container_candidate_count: 0,
                    notes: vec!["current function symbol missing from index".to_string()],
                })
        }
    };

    if let Some(container_id) = &current_symbol.container {
        if let Some(members) = context.index.by_container.get(container_id) {
            let matches: Vec<_> = members
                .iter()
                .filter_map(|id| context.index.symbols.get(id))
                .filter(|symbol| symbol.name == member)
                .collect();

            if matches.len() == 1 {
                return ResolutionResult::resolved(
                    matches[0].id.clone(),
                    0.95,
                    ResolutionMethod::ContainerMember,
                    vec![ResolutionEvidence::MatchingContainer],
                );
            }

            if matches.len() > 1 {
                return ResolutionResult::unresolved_with_reason(UnresolvedReason::ContainerMiss)
                    .with_debug(ResolutionDebugInfo {
                        query: Some(member.to_string()),
                        scope_checked: false,
                        same_file_candidate_count: 0,
                        import_candidate_count: 0,
                        wildcard_candidate_count: 0,
                        global_candidate_count: 0,
                        container_candidate_count: matches.len(),
                        notes: vec!["multiple container members matched".to_string()],
                    });
            }
        }
    }

    ResolutionResult::unresolved_with_reason(UnresolvedReason::ContainerMiss).with_debug(
        ResolutionDebugInfo {
            query: Some(member.to_string()),
            scope_checked: false,
            same_file_candidate_count: 0,
            import_candidate_count: 0,
            wildcard_candidate_count: 0,
            global_candidate_count: 0,
            container_candidate_count: 0,
            notes: vec!["no matching member found in current container".to_string()],
        },
    )
}

fn resolve_member_on_named_receiver(
    receiver_name: &str,
    member: &str,
    context: &ResolutionContext,
) -> Option<ResolutionResult> {
    let scoped_receiver = match context.scopes.resolve(&context.scope, receiver_name) {
        Some(symbol) => symbol,
        None => {
            return Some(
                ResolutionResult::unresolved_with_reason(UnresolvedReason::ReceiverUnbound)
                    .with_debug(ResolutionDebugInfo {
                        query: Some(format!("{}.{}", receiver_name, member)),
                        scope_checked: true,
                        same_file_candidate_count: 0,
                        import_candidate_count: 0,
                        wildcard_candidate_count: 0,
                        global_candidate_count: 0,
                        container_candidate_count: 0,
                        notes: vec![
                            "receiver name was not found in the current scope chain".to_string()
                        ],
                    }),
            )
        }
    };

    let symbol = context.index.symbols.get(scoped_receiver);

    if let Some(symbol) = symbol {
        if let Some(container_id) = &symbol.container {
            if let Some(members) = context.index.by_container.get(container_id) {
                let matches: Vec<_> = members
                    .iter()
                    .filter_map(|id| context.index.symbols.get(id))
                    .filter(|candidate| candidate.name == member)
                    .collect();

                if matches.len() == 1 {
                    return Some(ResolutionResult::resolved(
                        matches[0].id.clone(),
                        0.85,
                        ResolutionMethod::ContainerMember,
                        vec![ResolutionEvidence::MatchingContainer],
                    ));
                }

                if matches.len() > 1 {
                    return Some(
                        ResolutionResult::unresolved_with_reason(UnresolvedReason::ContainerMiss)
                            .with_debug(ResolutionDebugInfo {
                                query: Some(format!("{}.{}", receiver_name, member)),
                                scope_checked: true,
                                same_file_candidate_count: 0,
                                import_candidate_count: 0,
                                wildcard_candidate_count: 0,
                                global_candidate_count: 0,
                                container_candidate_count: matches.len(),
                                notes: vec![
                                    "multiple members matched receiver container".to_string()
                                ],
                            }),
                    );
                }
            }
        }
    }

    None
}

fn resolve_wildcard_import(
    import: &crate::resolution::symbol::ImportBinding,
    name: &str,
    context: &ResolutionContext,
) -> Vec<SymbolId> {
    let module_path = import
        .module
        .0
        .trim_end_matches("::*")
        .trim_end_matches('*')
        .trim_end_matches("::");

    let mut results = collect_unique_symbol_ids(
        context
            .index
            .find_by_qualified(&format!("{}::{}", module_path, name))
            .into_iter()
            .map(|s| s.id.clone()),
    );

    if let Some(file_path) = context.index.module_aliases.get(module_path) {
        let candidate = SymbolId(format!("{}::{}", file_path, name));
        if context.index.symbols.contains_key(&candidate) {
            results.push(candidate);
        }
    }

    collect_unique_symbol_ids(results.into_iter())
}

fn collect_unique_symbol_ids(iter: impl Iterator<Item = SymbolId>) -> Vec<SymbolId> {
    let mut ids: Vec<_> = iter.collect();
    ids.sort();
    ids.dedup();
    ids
}
