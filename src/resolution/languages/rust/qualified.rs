use crate::resolution::context::ResolutionContext;
use crate::resolution::result::{
    ResolutionCandidate, ResolutionDebugInfo, ResolutionEvidence, ResolutionMethod,
    ResolutionResult, UnresolvedReason,
};
use crate::resolution::symbol::{FileId, SymbolId};

use super::RustResolver;

impl RustResolver {
    pub(super) fn resolve_qualified(
        &self,
        parts: &[String],
        context: &ResolutionContext,
    ) -> ResolutionResult {
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

                if normalized_parts.len() >= 3 {
                    let type_name = &normalized_parts[1];
                    let method_name = &normalized_parts[2];

                    let with_type = format!("{}::{}::{}", file_path, type_name, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(with_type)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.95,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    let without_type = format!("{}::{}", file_path, method_name);
                    if let Some(target) = context.index.symbols.get(&SymbolId(without_type)) {
                        return ResolutionResult::resolved(
                            target.id.clone(),
                            0.90,
                            ResolutionMethod::TypeMember,
                            vec![ResolutionEvidence::MatchingType],
                        );
                    }

                    if let Some(files) = context.index.type_files.get(type_name.as_str()) {
                        for type_file in files {
                            let type_file_path = &type_file.0;
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

                    let file_candidates = context
                        .index
                        .find_in_file(&FileId(file_path.clone()), method_name);
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

        if normalized_parts
            .first()
            .map(|s| s == "code_intelligence")
            .unwrap_or(false)
        {
            let mut new_parts = vec!["crate".to_string()];
            new_parts.extend(normalized_parts[1..].to_vec());
            return self.resolve_qualified(&new_parts, context);
        }

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

            debug.notes.push(
                "crate-qualified path missed indexed project symbols; treating as external"
                    .to_string(),
            );
            return ResolutionResult::external().with_debug(debug);
        }

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

                let method_name = normalized_parts.last().unwrap();
                let same_file = context.index.find_in_file(&context.file, method_name);
                debug.same_file_candidate_count = same_file.len();
                if same_file.len() == 1 {
                    debug.notes.push(
                        "caller had no container, but same-file Self target resolved uniquely"
                            .to_string(),
                    );
                    return ResolutionResult::resolved(
                        same_file[0].id.clone(),
                        0.82,
                        ResolutionMethod::LocalSymbol,
                        vec![ResolutionEvidence::SameFile],
                    );
                }

                let global_type_matches: Vec<_> = context
                    .index
                    .find_by_name(method_name)
                    .into_iter()
                    .filter(|symbol| symbol.declared_type.is_some())
                    .collect();
                debug.container_candidate_count = global_type_matches.len();
                if global_type_matches.len() == 1 {
                    debug.notes.push(
                        "caller had no container, but a unique typed method matched Self target"
                            .to_string(),
                    );
                    return ResolutionResult::resolved(
                        global_type_matches[0].id.clone(),
                        0.78,
                        ResolutionMethod::TypeMember,
                        vec![ResolutionEvidence::MatchingType],
                    );
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

        let qualified_candidates = context.index.find_by_qualified(&joined);
        if qualified_candidates.len() == 1 {
            return ResolutionResult::resolved(
                qualified_candidates[0].id.clone(),
                0.95,
                ResolutionMethod::QualifiedSymbol,
                vec![ResolutionEvidence::MatchingSymbol],
            );
        }

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
                let expected_path = format!("src/parser/languages/{}.rs", module_name);

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

                debug.notes.push(format!(
                    "known parser language module alias {} did not resolve to indexed symbols",
                    module_name
                ));
                return ResolutionResult::external().with_debug(debug);
            }
        }

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

        if normalized_parts.first().map(|s| s == "fs").unwrap_or(false) {
            debug
                .notes
                .push("std/fs qualified path treated as external".to_string());
            return ResolutionResult::external().with_debug(debug);
        }

        if Self::looks_like_external_qualified_call(&normalized_parts) {
            debug.notes.push(format!(
                "qualified path {} matched external/std/dependency heuristic",
                joined
            ));
            return ResolutionResult::external().with_debug(debug);
        }

        debug
            .notes
            .push("qualified lookup exhausted all Rust heuristics".to_string());
        ResolutionResult::unresolved_with_reason(UnresolvedReason::ContainerMiss).with_debug(debug)
    }
}
