use crate::resolution::call_site::CalleeExpr;
use crate::resolution::context::ResolutionContext;
use crate::resolution::result::{
    ResolutionCandidate, ResolutionDebugInfo, ResolutionEvidence, ResolutionMethod,
    ResolutionResult, UnresolvedReason,
};

use super::RustResolver;

impl RustResolver {
    pub(super) fn resolve_member(
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
            if Self::std_member_is_external(member) {
                debug.notes.push(format!(
                    "receiver {} member {} treated as stdlib/container-style external method",
                    receiver_name, member
                ));
                return ResolutionResult::external().with_debug(debug);
            }
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

        if Self::std_member_is_external(member) {
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
