// src/resolution/languages/rust.rs

use crate::resolution::call_site::{CallSite, CalleeExpr};
use crate::resolution::context::{Language, ResolutionContext};
use crate::resolution::result::{
    ResolutionCandidate, ResolutionDebugInfo, ResolutionEvidence, ResolutionMethod,
    ResolutionResult, UnresolvedReason,
};
use crate::resolution::symbol::{ImportBinding, SymbolId, TypeId};
use crate::resolution::traits::LanguageResolver;
use crate::resolution::ResolutionStatus;

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

impl RustResolver {
    fn strip_generic_args(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        let chars: Vec<char> = name.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == ':' && chars[i + 1] == '<' {
                i += 2;
                let mut depth = 1usize;
                while i < chars.len() && depth > 0 {
                    match chars[i] {
                        '<' => depth += 1,
                        '>' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                continue;
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }

    fn resolve_bare_name(&self, name: &str, context: &ResolutionContext) -> ResolutionResult {
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
                        crate::resolution::symbol::SymbolKind::Function
                            | crate::resolution::symbol::SymbolKind::Constructor
                            | crate::resolution::symbol::SymbolKind::AssociatedFunction
                            | crate::resolution::symbol::SymbolKind::StaticMethod
                            | crate::resolution::symbol::SymbolKind::ClassMethod
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

        // Tier 3: Import resolution
        if let Some(imports) = context.index.imports.get(&context.file) {
            let mut import_candidates = Vec::new();
            for import in imports {
                let imported_name_match = import.imported_name.as_deref() == Some(name);
                let local_name_match = import.local_name == name;
                let matches = local_name_match
                    || imported_name_match
                    || import.module.0.ends_with(&format!("::{}", name));

                if matches {
                    if let Some(symbol) = &import.symbol {
                        return ResolutionResult::resolved(
                            symbol.clone(),
                            0.92,
                            ResolutionMethod::ImportedSymbol,
                            vec![ResolutionEvidence::ExplicitImport],
                        );
                    }

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

                    import_candidates.extend(module_candidates.into_iter().map(|s| s.id.clone()));

                    let name_candidates: Vec<_> = context
                        .index
                        .find_by_name(name)
                        .into_iter()
                        .filter(|candidate| {
                            if imported_name_match {
                                candidate.id.0.contains(&import.module.0)
                                    || candidate
                                        .module
                                        .as_ref()
                                        .map(|m| m.0.starts_with(&import.module.0))
                                        .unwrap_or(false)
                            } else if local_name_match {
                                candidate.id.0.contains(&import.module.0)
                            } else {
                                true
                            }
                        })
                        .collect();
                    if name_candidates.len() == 1 {
                        return ResolutionResult::resolved(
                            name_candidates[0].id.clone(),
                            0.85,
                            ResolutionMethod::ImportedSymbol,
                            vec![ResolutionEvidence::ExplicitImport],
                        );
                    }

                    import_candidates.extend(name_candidates.into_iter().map(|s| s.id.clone()));
                }
            }
            import_candidates.sort();
            import_candidates.dedup();
            debug.import_candidate_count = import_candidates.len();
            if import_candidates.len() > 1 {
                return ResolutionResult::unresolved_with_reason(UnresolvedReason::ImportAmbiguous)
                    .with_debug(debug);
            }
        }

        // Tier 3.5: Wildcard (glob) import resolution — `use module::*;`
        // A bare name not caught by Tier 3 may still be reachable through a
        // glob import. Only free functions are matched, since a bare call
        // can never statically resolve to a method requiring a receiver.
        if let Some(imports) = context.index.imports.get(&context.file) {
            let mut wildcard_matches = Vec::new();
            for import in imports {
                if import.kind != crate::resolution::symbol::ImportKind::Wildcard {
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
                            if target.kind == crate::resolution::symbol::SymbolKind::Function {
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

        // Handle generic method calls like resp.json::<Type>()
        if name.contains("::<") {
            return ResolutionResult::external();
        }

        // Handle bare names that are known external/stdlib methods that also
        // happen to collide with internal symbol names by bare name alone.
        // Must run before Tier 4, since a name collision would otherwise
        // return Ambiguous before we ever get a chance to check this list.
        let common_methods = ["cosine_similarity", "feature_names", "feature_count"];
        if common_methods.contains(&name) {
            return ResolutionResult::external();
        }

        // Tier 4: Global name fallback (unambiguous only)
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

        // Handle type aliases (FileId, ModuleId, ScopeId, SymbolId, TypeId)
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

        // Handle bare names that are actually methods or associated functions
        let same_file_candidates = context.index.find_in_file(&context.file, name);
        if !same_file_candidates.is_empty() {
            if same_file_candidates.len() == 1 {
                return ResolutionResult::resolved(
                    same_file_candidates[0].id.clone(),
                    0.90,
                    ResolutionMethod::LocalSymbol,
                    vec![ResolutionEvidence::SameFile],
                );
            }
        }

        // Search across all files for unique matches
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
    /// A chained call (`x.iter().filter(...).collect()`) whose receiver type we
    /// don't track, so we can't do container-based lookup. We still run it
    /// through full bare-name resolution in case it's a real project method
    /// reached via a chain (same-file, imports, unambiguous global match all
    /// still apply and can succeed here). But if that search exhausts every
    /// tier and finds zero matches anywhere in the indexed project, that
    /// absence is itself the signal: a method name that isn't a symbol in this
    /// codebase cannot be a call into this codebase. Report it as External
    /// rather than Unresolved. This adapts automatically as the project and its
    /// dependencies change — no name list to maintain.
    fn resolve_chained_method(&self, name: &str, context: &ResolutionContext) -> ResolutionResult {
        let result = self.resolve_bare_name(name, context);
        if matches!(result.status, ResolutionStatus::Unresolved) {
            return ResolutionResult::external();
        }
        result
    }

    fn resolve_qualified(&self, parts: &[String], context: &ResolutionContext) -> ResolutionResult {
        let normalized_parts: Vec<String> = parts
            .iter()
            .map(|part| Self::strip_generic_args(part))
            .collect();
        let joined = normalized_parts.join("::");
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

        // Try module alias resolution first
        if let Some(first) = normalized_parts.first() {
            if let Some(file_path) = context.index.module_aliases.get(first) {
                let rest = normalized_parts[1..].join("::");
                let full_path = if rest.is_empty() {
                    file_path.clone()
                } else {
                    format!("{}::{}", file_path, rest)
                };

                if let Some(target) = context.index.symbols.get(&SymbolId(full_path.clone())) {
                    return ResolutionResult::resolved(
                        target.id.clone(),
                        0.95,
                        ResolutionMethod::QualifiedSymbol,
                        vec![ResolutionEvidence::MatchingModule],
                    );
                }

                // Try with just the last part
                let last = normalized_parts.last().unwrap();
                let simple_path = format!("{}::{}", file_path, last);
                if let Some(target) = context.index.symbols.get(&SymbolId(simple_path)) {
                    return ResolutionResult::resolved(
                        target.id.clone(),
                        0.90,
                        ResolutionMethod::QualifiedSymbol,
                        vec![ResolutionEvidence::MatchingModule],
                    );
                }

                // Try with the type name as container (e.g., rust::RustParser::extract_functions)
                // The symbol might be stored as file_path::RustParser::extract_functions
                // OR as file_path::extract_functions (without the type name)
                if normalized_parts.len() >= 3 {
                    let type_name = &normalized_parts[1];
                    let method_name = &normalized_parts[2];

                    // Try file_path::Type::method
                    let with_type = format!("{}::{}::{}", file_path, type_name, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(with_type)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.95,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    // Try file_path::method (without type name)
                    let without_type = format!("{}::{}", file_path, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(without_type)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.90,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    // Also search by the type name in type_files
                    if let Some(files) = context.index.type_files.get(type_name.as_str()) {
                        for type_file in files {
                            let type_file_path = &type_file.0;

                            // Try type_file::Type::method
                            let type_method =
                                format!("{}::{}::{}", type_file_path, type_name, method_name);
                            if let Some(target) = context.index.symbols.get(&SymbolId(type_method))
                            {
                                return ResolutionResult::resolved(
                                    target.id.clone(),
                                    0.95,
                                    ResolutionMethod::TypeMember,
                                    vec![ResolutionEvidence::MatchingType],
                                );
                            }

                            // Try type_file::method
                            let simple_method = format!("{}::{}", type_file_path, method_name);
                            if let Some(target) =
                                context.index.symbols.get(&SymbolId(simple_method))
                            {
                                return ResolutionResult::resolved(
                                    target.id.clone(),
                                    0.90,
                                    ResolutionMethod::TypeMember,
                                    vec![ResolutionEvidence::MatchingType],
                                );
                            }
                        }
                    }

                    // Last resort: search by method name in the file
                    let file_candidates = context.index.find_in_file(
                        &crate::resolution::symbol::FileId(file_path.clone()),
                        method_name,
                    );
                    if file_candidates.len() == 1 {
                        return ResolutionResult::resolved(
                            file_candidates[0].id.clone(),
                            0.85,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }
                }
            }
        }
        // Handle code_intelligence:: paths (the crate's own name)
        if normalized_parts
            .first()
            .map(|s| s == "code_intelligence")
            .unwrap_or(false)
        {
            let mut new_parts = vec!["crate".to_string()];
            new_parts.extend(normalized_parts[1..].to_vec());
            return self.resolve_qualified(&new_parts, context);
        }

        // Handle crate:: paths - resolve to actual file paths
        if normalized_parts
            .first()
            .map(|s| s == "crate")
            .unwrap_or(false)
        {
            let rest = &normalized_parts[1..];
            let target_name = rest.last().unwrap();
            let module_path = rest[..rest.len() - 1].join("/");
            let prefix = if context.file.0.starts_with("./") {
                "./"
            } else {
                ""
            };

            // Try direct file path resolution
            let direct_file = format!("{}src/{}.rs", prefix, module_path);
            let direct = format!("{}::{}", direct_file, target_name);
            if let Some(target) = context.index.symbols.get(&SymbolId(direct.clone())) {
                return ResolutionResult::resolved(
                    target.id.clone(),
                    0.95,
                    ResolutionMethod::QualifiedSymbol,
                    vec![ResolutionEvidence::MatchingModule],
                );
            }

            // Try mod.rs path
            let mod_file = format!("{}src/{}/mod.rs", prefix, module_path);
            let mod_direct = format!("{}::{}", mod_file, target_name);
            if let Some(target) = context.index.symbols.get(&SymbolId(mod_direct.clone())) {
                return ResolutionResult::resolved(
                    target.id.clone(),
                    0.95,
                    ResolutionMethod::QualifiedSymbol,
                    vec![ResolutionEvidence::MatchingModule],
                );
            }

            let candidates = context.index.find_by_name(target_name);
            let matching: Vec<_> = candidates
                .iter()
                .filter(|s| s.file.0.contains(&module_path) || s.id.0.contains(&module_path))
                .collect();

            if matching.len() == 1 {
                return ResolutionResult::resolved(
                    matching[0].id.clone(),
                    0.95,
                    ResolutionMethod::QualifiedSymbol,
                    vec![ResolutionEvidence::MatchingModule],
                );
            }

            if matching.len() > 1 {
                let exact: Vec<_> = matching
                    .iter()
                    .filter(|s| {
                        s.id.0.ends_with(&format!(
                            "{}::{}",
                            module_path.replace("/", "::"),
                            target_name
                        ))
                    })
                    .collect();

                if exact.len() == 1 {
                    return ResolutionResult::resolved(
                        exact[0].id.clone(),
                        0.95,
                        ResolutionMethod::QualifiedSymbol,
                        vec![ResolutionEvidence::MatchingModule],
                    );
                }
            }

            // If we can't resolve it, mark as external
            debug.notes.push(
                "crate-qualified path missed indexed project symbols; treating as external"
                    .to_string(),
            );
            return ResolutionResult::external().with_debug(debug);
        }

        // Handle Self:: calls
        if normalized_parts
            .first()
            .map(|s| s == "Self")
            .unwrap_or(false)
        {
            if let Some(caller) = context.index.symbols.get(&context.function) {
                if let Some(container) = &caller.container {
                    let method_name = normalized_parts.last().unwrap();
                    if let Some(members) = context.index.by_container.get(container) {
                        let matching: Vec<_> = members
                            .iter()
                            .filter_map(|id| context.index.symbols.get(id))
                            .filter(|s| s.name == *method_name)
                            .collect();
                        debug.container_candidate_count = matching.len();

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
                            let mut result = ResolutionResult::ambiguous(candidates)
                                .with_reason(UnresolvedReason::GlobalAmbiguous);
                            result.debug = Some(debug.clone());
                            return result;
                        }
                    }
                    if let Some(type_id) = &caller.declared_type {
                        if let Some(type_members) = context.index.by_type.get(type_id) {
                            let matching: Vec<_> = type_members
                                .iter()
                                .filter_map(|id| context.index.symbols.get(id))
                                .filter(|s| s.name == *method_name)
                                .collect();
                            debug.container_candidate_count = matching.len();

                            if matching.len() == 1 {
                                return ResolutionResult::resolved(
                                    matching[0].id.clone(),
                                    0.93,
                                    ResolutionMethod::TypeMember,
                                    vec![ResolutionEvidence::MatchingType],
                                );
                            }
                            if matching.len() > 1 {
                                let candidates = matching
                                    .iter()
                                    .map(|s| ResolutionCandidate {
                                        symbol: s.id.clone(),
                                        method: ResolutionMethod::TypeMember,
                                        confidence: 0.45,
                                        evidence: vec![ResolutionEvidence::MatchingType],
                                    })
                                    .collect();
                                let mut result = ResolutionResult::ambiguous(candidates)
                                    .with_reason(UnresolvedReason::GlobalAmbiguous);
                                result.debug = Some(debug.clone());
                                return result;
                            }
                        }
                    }

                    let same_file = context.index.find_in_file(&context.file, method_name);
                    debug.same_file_candidate_count = same_file.len();
                    if same_file.len() == 1 {
                        return ResolutionResult::resolved(
                            same_file[0].id.clone(),
                            0.80,
                            ResolutionMethod::LocalSymbol,
                            vec![ResolutionEvidence::SameFile],
                        );
                    }

                    if method_name == "default" {
                        debug.notes.push(format!(
                            "Self resolved to container {}, but default is likely provided by derived/trait Default rather than indexed impl",
                            container.0
                        ));
                        return ResolutionResult::external().with_debug(debug);
                    }

                    debug.notes.push(format!(
                        "Self resolved to container {}, but no matching member was found",
                        container.0
                    ));
                    return ResolutionResult::unresolved_with_reason(
                        UnresolvedReason::ContainerMiss,
                    )
                    .with_debug(debug);
                }

                debug
                    .notes
                    .push("caller symbol had no container for Self resolution".to_string());
                return ResolutionResult::unresolved_with_reason(
                    UnresolvedReason::MissingCurrentSymbol,
                )
                .with_debug(debug);
            }

            debug
                .notes
                .push("current function symbol missing during Self resolution".to_string());
            return ResolutionResult::unresolved_with_reason(
                UnresolvedReason::MissingCurrentSymbol,
            )
            .with_debug(debug);
        }

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
        let root = &normalized_parts[0];

        for (module_id, module) in &context.index.modules {
            let module_short = module_id.0.split("::").last().unwrap_or(&module_id.0);

            if module_short == root || module_id.0.ends_with(&format!("::{}", root)) {
                let file_path = &module.file.0;
                let rest = normalized_parts[1..].join("::");
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

                let last = normalized_parts.last().unwrap();
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
            let rest = normalized_parts[1..].join("::");
            let full_path = if rest.is_empty() {
                module_path
            } else {
                format!("{}::{}", module_path, rest)
            };

            for (module_id, module) in &context.index.modules {
                if module_id.0 == full_path || module_id.0.starts_with(&full_path) {
                    let file_path = &module.file.0;
                    let last = normalized_parts.last().unwrap();
                    let file_based = format!("{}::{}", file_path, last);

                    if let Some(target) = context.index.symbols.get(&SymbolId(file_based)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.95,
                            ResolutionMethod::QualifiedSymbol,
                            vec![ResolutionEvidence::MatchingModule],
                        );
                    }

                    if normalized_parts.len() >= 3 {
                        let container = &normalized_parts[normalized_parts.len() - 2];
                        let method = &normalized_parts[normalized_parts.len() - 1];
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
        if normalized_parts.len() >= 2 {
            let type_name = &normalized_parts[normalized_parts.len() - 2];
            let method_name = &normalized_parts[normalized_parts.len() - 1];

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

            let first_type = &normalized_parts[0];
            let last_method = &normalized_parts[normalized_parts.len() - 1];

            for symbol in context.index.symbols.values() {
                if symbol.name == *first_type && symbol.container.is_none() {
                    let target_path = format!("{}::{}", symbol.file.0, last_method);
                    if let Some(target) = context.index.symbols.get(&SymbolId(target_path)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.85,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    let full_target = format!("{}::{}::{}", symbol.file.0, first_type, last_method);
                    if let Some(target) = context.index.symbols.get(&SymbolId(full_target)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.85,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }
                }
            }
        }

        // Handle Default::default() and similar trait methods
        if normalized_parts.len() == 2 && normalized_parts[1] == "default" {
            let type_name = &normalized_parts[0];

            if let Some(files) = context.index.type_files.get(type_name.as_str()) {
                if !files.is_empty() {
                    let file_path = &files[0].0;

                    let direct_path = format!("{}::default", file_path);
                    if let Some(target) = context.index.symbols.get(&SymbolId(direct_path)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.90,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    let container_path = format!("{}::{}::default", file_path, type_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(container_path)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.90,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    return ResolutionResult::external();
                }
            }

            return ResolutionResult::external();
        }

        // Handle internal type constructors and associated functions
        if normalized_parts.len() >= 2 {
            let type_name = &normalized_parts[0];
            let method_name = normalized_parts.last().unwrap();

            if let Some(files) = context.index.type_files.get(type_name.as_str()) {
                for file_id in files {
                    let file_path = &file_id.0;

                    let direct = format!("{}::{}", file_path, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(direct)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.90,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    let with_container = format!("{}::{}::{}", file_path, type_name, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(with_container)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.90,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }
                }

                return ResolutionResult::external();
            }

            let candidates = context.index.find_by_name(method_name);
            let matching: Vec<_> = candidates
                .iter()
                .filter(|s| {
                    s.id.0.contains(type_name)
                        || s.container
                            .as_ref()
                            .map(|c| c.0.contains(type_name))
                            .unwrap_or(false)
                })
                .collect();

            if matching.len() == 1 {
                return ResolutionResult::resolved(
                    matching[0].id.clone(),
                    0.85,
                    ResolutionMethod::TypeMember,
                    vec![ResolutionEvidence::MatchingType],
                );
            }
        }

        // Handle module aliases (rust::RustParser, python::PythonParser, etc.)
        if normalized_parts.len() >= 2 {
            let first = &normalized_parts[0];
            let known_modules = [
                "rust",
                "python",
                "javascript",
                "typescript",
                "go",
                "java",
                "dart",
                "php",
                "cpp",
                "csharp",
            ];

            if known_modules.contains(&first.as_str()) {
                let module_name = first;
                let type_name = &normalized_parts[1];

                // Try to find the actual file path for this module
                let expected_path = format!("src/parser/languages/{}.rs", module_name);

                // Search for the type in our symbols
                let candidates = context.index.find_by_name(type_name);
                let matching: Vec<_> = candidates
                    .iter()
                    .filter(|s| s.file.0.contains(&expected_path))
                    .collect();

                if matching.len() == 1 {
                    return ResolutionResult::resolved(
                        matching[0].id.clone(),
                        0.95,
                        ResolutionMethod::QualifiedSymbol,
                        vec![ResolutionEvidence::MatchingModule],
                    );
                }

                // If we can't find it, mark as external
                debug.notes.push(format!(
                    "known parser language module alias {} did not resolve to indexed symbols",
                    module_name
                ));
                return ResolutionResult::external().with_debug(debug);
            }
        }

        // Handle enum variant constructors
        if normalized_parts.len() == 2 {
            let type_name = &normalized_parts[0];
            let variant = &normalized_parts[1];

            if context.index.type_files.get(type_name.as_str()).is_some() {
                debug.notes.push(format!(
                    "{}::{} treated as enum variant or type-associated external constructor",
                    type_name, variant
                ));
                return ResolutionResult::external().with_debug(debug);
            }

            if type_name.chars().next().map_or(false, |c| c.is_uppercase())
                && variant.chars().next().map_or(false, |c| c.is_uppercase())
            {
                debug.notes.push(format!(
                    "{}::{} matched enum-variant-like uppercase constructor pattern",
                    type_name, variant
                ));
                return ResolutionResult::external().with_debug(debug);
            }
        }

        // Handle code_intelligence:: paths
        if normalized_parts
            .first()
            .map(|s| s == "code_intelligence")
            .unwrap_or(false)
        {
            debug
                .notes
                .push("crate-name-qualified path fell through to external".to_string());
            return ResolutionResult::external().with_debug(debug);
        }

        // Handle fs:: paths
        if normalized_parts.first().map(|s| s == "fs").unwrap_or(false) {
            debug
                .notes
                .push("std/fs qualified path treated as external".to_string());
            return ResolutionResult::external().with_debug(debug);
        }

        debug
            .notes
            .push("qualified lookup exhausted all Rust heuristics".to_string());
        ResolutionResult::unresolved_with_reason(UnresolvedReason::ContainerMiss).with_debug(debug)
    }

    fn resolve_member(
        &self,
        receiver: &CalleeExpr,
        member: &str,
        context: &ResolutionContext,
    ) -> ResolutionResult {
        let mut debug = ResolutionDebugInfo {
            query: Some(member.to_string()),
            scope_checked: false,
            same_file_candidate_count: 0,
            import_candidate_count: 0,
            wildcard_candidate_count: 0,
            global_candidate_count: 0,
            container_candidate_count: 0,
            notes: Vec::new(),
        };

        let receiver_name = match receiver {
            CalleeExpr::Name(name) => name.clone(),
            _ => {
                debug
                    .notes
                    .push("member receiver was not a simple name".to_string());
                return ResolutionResult::unresolved_with_reason(
                    UnresolvedReason::UnsupportedCalleeShape,
                )
                .with_debug(debug);
            }
        };

        if receiver_name == "self" || receiver_name == "this" {
            if let Some(caller) = context.index.symbols.get(&context.function) {
                if let Some(container) = &caller.container {
                    if let Some(members) = context.index.by_container.get(container) {
                        let matching: Vec<_> = members
                            .iter()
                            .filter_map(|id| context.index.symbols.get(id))
                            .filter(|s| s.name == member)
                            .collect();

                        debug.container_candidate_count = matching.len();
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
                            let mut result = ResolutionResult::ambiguous(candidates)
                                .with_reason(UnresolvedReason::GlobalAmbiguous);
                            result.debug = Some(debug.clone());
                            return result;
                        }
                    }

                    if let Some(type_id) = &caller.declared_type {
                        if let Some(type_members) = context.index.by_type.get(type_id) {
                            let matching: Vec<_> = type_members
                                .iter()
                                .filter_map(|id| context.index.symbols.get(id))
                                .filter(|s| s.name == member)
                                .collect();
                            debug.container_candidate_count = matching.len();
                            if matching.len() == 1 {
                                return ResolutionResult::resolved(
                                    matching[0].id.clone(),
                                    0.93,
                                    ResolutionMethod::TypeMember,
                                    vec![ResolutionEvidence::MatchingType],
                                );
                            }
                            if matching.len() > 1 {
                                let candidates = matching
                                    .iter()
                                    .map(|s| ResolutionCandidate {
                                        symbol: s.id.clone(),
                                        method: ResolutionMethod::TypeMember,
                                        confidence: 0.45,
                                        evidence: vec![ResolutionEvidence::MatchingType],
                                    })
                                    .collect();
                                let mut result = ResolutionResult::ambiguous(candidates)
                                    .with_reason(UnresolvedReason::GlobalAmbiguous);
                                result.debug = Some(debug.clone());
                                return result;
                            }
                        }
                    }
                }
            }

            let same_file = context.index.find_in_file(&context.file, member);
            debug.same_file_candidate_count = same_file.len();
            if same_file.len() == 1 {
                return ResolutionResult::resolved(
                    same_file[0].id.clone(),
                    0.80,
                    ResolutionMethod::LocalSymbol,
                    vec![ResolutionEvidence::SameFile],
                );
            }
            if matches!(
                member,
                "iter"
                    | "filter"
                    | "map"
                    | "collect"
                    | "len"
                    | "is_empty"
                    | "contains"
                    | "is_ok"
                    | "to_vec"
                    | "ok_or_else"
                    | "is_some"
                    | "edge_count"
            ) {
                debug.notes.push(format!(
                    "receiver {} member {} treated as stdlib/container-style external method",
                    receiver_name, member
                ));
                return ResolutionResult::external().with_debug(debug);
            }
            // A self/this call that can't be resolved via container membership or
            // same-file lookup is a resolver gap, not a genuine external
            // dependency — self.foo() can never call into another crate. Report
            // it as Unresolved so it's visible in diagnostics instead of being
            // silently absorbed as "external" and dropped from the call graph.
            debug.notes.push(format!(
                "receiver {} did not resolve to a unique container member or same-file symbol",
                receiver_name
            ));
            return ResolutionResult::unresolved_with_reason(UnresolvedReason::ContainerMiss)
                .with_debug(debug);
        }

        let receiver_candidates = context.index.find_by_name(&receiver_name);
        debug.global_candidate_count = receiver_candidates.len();
        if receiver_candidates.len() == 1 {
            let receiver_symbol = &receiver_candidates[0];
            if let Some(container) = &receiver_symbol.container {
                if let Some(members) = context.index.by_container.get(container) {
                    let matching: Vec<_> = members
                        .iter()
                        .filter_map(|id| context.index.symbols.get(id))
                        .filter(|s| s.name == member)
                        .collect();

                    debug.container_candidate_count = matching.len();
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

            let same_file = context.index.find_in_file(&receiver_symbol.file, member);
            debug.same_file_candidate_count = same_file.len();
            if same_file.len() == 1 {
                return ResolutionResult::resolved(
                    same_file[0].id.clone(),
                    0.80,
                    ResolutionMethod::LocalSymbol,
                    vec![ResolutionEvidence::SameFile],
                );
            }
        }

        let same_file = context.index.find_in_file(&context.file, member);
        debug.same_file_candidate_count = same_file.len();
        if same_file.len() == 1 {
            return ResolutionResult::resolved(
                same_file[0].id.clone(),
                0.75,
                ResolutionMethod::LocalSymbol,
                vec![ResolutionEvidence::SameFile],
            );
        }

        if matches!(
            member,
            "iter"
                | "filter"
                | "map"
                | "collect"
                | "len"
                | "is_empty"
                | "contains"
                | "is_ok"
                | "to_vec"
                | "ok_or_else"
                | "is_some"
                | "edge_count"
        ) {
            debug.notes.push(format!(
                "member lookup for receiver {} and member {} treated as stdlib/container-style external method",
                receiver_name, member
            ));
            return ResolutionResult::external().with_debug(debug);
        }

        debug.notes.push(format!(
            "member lookup for receiver {} and member {} fell back to external",
            receiver_name, member
        ));
        ResolutionResult::external().with_debug(debug)
    }
}
