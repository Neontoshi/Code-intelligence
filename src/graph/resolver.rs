// src/graph/resolver.rs

//! Call resolver with multi-stage resolution

use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::import_graph::ImportGraph;
use crate::graph::resolution::{ResolutionConfidence, ResolutionMethod, ResolvedCall};
use crate::graph::type_graph::TypeGraph;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct CallResolver {
    call_graph: CallGraph,
    import_graph: ImportGraph,
    type_graph: TypeGraph,
    method_cache: HashMap<String, Vec<String>>,
}

impl CallResolver {
    pub fn new(call_graph: CallGraph, import_graph: ImportGraph, type_graph: TypeGraph) -> Self {
        Self {
            call_graph,
            import_graph,
            type_graph,
            method_cache: HashMap::new(),
        }
    }

    pub fn resolve_all(&mut self) -> CallGraph {
        let mut resolved_graph = self.call_graph.clone();

        for idx in resolved_graph.node_indices() {
            let func = resolved_graph[idx].clone();
            let calls = self.resolve_calls_for_function(&func, &resolved_graph);

            for resolved in calls {
                if resolved.target_full_path.is_empty() {
                    // Mark as unresolved
                    resolved_graph
                        .unresolved_calls
                        .entry(func.full_path.clone())
                        .or_default()
                        .push(resolved.target_name.clone());
                    continue;
                }

                if let Some(_callee_idx) = resolved_graph.name_index.get(&resolved.target_full_path)
                {
                    resolved_graph.set_resolution_confidence(
                        &func.full_path,
                        &resolved.target_full_path,
                        resolved.confidence,
                    );
                }
            }
        }

        resolved_graph
    }

    pub fn resolve_calls_for_function(
        &self,
        func: &FunctionNode,
        call_graph: &CallGraph,
    ) -> Vec<ResolvedCall> {
        let mut resolved = Vec::new();

        // Get the call targets from the function info
        // In practice, this comes from the parser
        let call_targets = self.extract_call_targets(func);

        for target in call_targets {
            let resolution = self.resolve_single_call(func, &target, call_graph);
            resolved.push(resolution);
        }

        resolved
    }

    fn extract_call_targets(&self, func: &FunctionNode) -> Vec<String> {
        // Read the callees directly from the graph edges or indexed name lookups
        if let Some(&idx) = self.call_graph.name_index.get(&func.full_path) {
            self.call_graph
                .get_callees(idx)
                .into_iter()
                .map(|callee| callee.name.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    fn resolve_single_call(
        &self,
        caller: &FunctionNode,
        target: &str,
        call_graph: &CallGraph,
    ) -> ResolvedCall {
        let (target_path, confidence, method) = self.resolve_target(caller, target, call_graph);

        ResolvedCall {
            target_full_path: target_path.unwrap_or_default(),
            target_name: target.to_string(),
            confidence,
            resolution_method: method,
            source_file: caller.file.clone(),
            line: caller.line,
        }
    }

    fn resolve_target(
        &self,
        caller: &FunctionNode,
        target: &str,
        call_graph: &CallGraph,
    ) -> (Option<String>, ResolutionConfidence, ResolutionMethod) {
        // Try direct match first
        if let Some(&idx) = call_graph.name_index.get(target) {
            return (
                Some(call_graph[idx].full_path.clone()),
                ResolutionConfidence::Exact,
                ResolutionMethod::Direct,
            );
        }

        // Extract the enclosing container from the function full_path.
        // Expected full_path format: "file::container::method"
        let container = Self::extract_container(&caller.full_path);

        // Try method call on self
        if target.starts_with("self::") {
            let method_name = target.trim_start_matches("self::");
            // Look for method in the same container
            if let Some(container) = &container {
                let candidate = format!("{}::{}::{}", caller.file, container, method_name);
                if let Some(&idx) = call_graph.name_index.get(&candidate) {
                    return (
                        Some(call_graph[idx].full_path.clone()),
                        ResolutionConfidence::Inferred,
                        ResolutionMethod::SelfMethod,
                    );
                }
            }
        }

        // Try associated function (Type::method)
        if target.contains("::") {
            if let Some(&idx) = call_graph.name_index.get(target) {
                return (
                    Some(call_graph[idx].full_path.clone()),
                    ResolutionConfidence::Inferred,
                    ResolutionMethod::Associated,
                );
            }
        }

        // Try constructor calls
        if target.ends_with("::new") || target.ends_with("::default") {
            if let Some(&idx) = call_graph.name_index.get(target) {
                return (
                    Some(call_graph[idx].full_path.clone()),
                    ResolutionConfidence::Inferred,
                    ResolutionMethod::Constructor,
                );
            }
        }

        // Try import resolution
        let simple_name = target.rsplit("::").next().unwrap_or(target);
        // Resolve imported candidate paths for the caller's file.
        let imported_paths = self.import_graph.get_imported_functions(&caller.file);
        for path in imported_paths {
            if path.ends_with(simple_name) {
                if let Some(&idx) = call_graph.name_index.get(&path) {
                    return (
                        Some(call_graph[idx].full_path.clone()),
                        ResolutionConfidence::Inferred,
                        ResolutionMethod::Import,
                    );
                }
            }
        }

        // Try name match (single candidate)
        if let Some(candidates) = call_graph.name_to_functions.get(simple_name) {
            if candidates.len() == 1 {
                return (
                    Some(call_graph[candidates[0]].full_path.clone()),
                    ResolutionConfidence::Heuristic,
                    ResolutionMethod::NameMatch,
                );
            } else if candidates.len() > 1 {
                // Multiple candidates - try to disambiguate by file
                let same_file: Vec<_> = candidates
                    .iter()
                    .filter(|&&idx| call_graph[idx].file == caller.file)
                    .collect();

                if same_file.len() == 1 {
                    return (
                        Some(call_graph[*same_file[0]].full_path.clone()),
                        ResolutionConfidence::Heuristic,
                        ResolutionMethod::NameMatch,
                    );
                }

                // Try to disambiguate by container
                if let Some(container) = &container {
                    let same_container: Vec<_> = candidates
                        .iter()
                        .filter(|&&idx| {
                            call_graph[idx]
                                .full_path
                                .contains(&format!("::{}::", container))
                        })
                        .collect();

                    if same_container.len() == 1 {
                        return (
                            Some(call_graph[*same_container[0]].full_path.clone()),
                            ResolutionConfidence::Heuristic,
                            ResolutionMethod::ContainerMethod,
                        );
                    }
                }
            }
        }

        // Try container method using extracted container
        if let Some(container) = &container {
            let candidate = format!("{}::{}::{}", caller.file, container, simple_name);
            if let Some(&idx) = call_graph.name_index.get(&candidate) {
                return (
                    Some(call_graph[idx].full_path.clone()),
                    ResolutionConfidence::Heuristic,
                    ResolutionMethod::ContainerMethod,
                );
            }
        }

        // Try trait method resolution
        if let Some(trait_name) = &caller.trait_impl {
            // Look for trait method implementations
            for idx in call_graph.node_indices() {
                let f = &call_graph[idx];
                if f.name == simple_name && f.trait_impl.as_ref() == Some(trait_name) {
                    return (
                        Some(f.full_path.clone()),
                        ResolutionConfidence::Ambiguous,
                        ResolutionMethod::TraitMethod,
                    );
                }
            }
        }

        // Unresolved
        (
            None,
            ResolutionConfidence::Unresolved,
            ResolutionMethod::Unresolved,
        )
    }

    /// ⭐ NEW: Extract container from full_path
    /// Format: "file::container::method" or "file::method"
    fn extract_container(full_path: &str) -> Option<String> {
        let parts: Vec<&str> = full_path.split("::").collect();
        if parts.len() >= 3 {
            // file::container::method -> container
            Some(parts[parts.len() - 2].to_string())
        } else {
            None
        }
    }

    /// Generate resolution report
    pub fn resolution_report(&self) -> String {
        let stats = self.call_graph.resolution_stats();

        let mut report = String::new();
        report.push_str("## 📞 Call Resolution Report\n\n");
        report.push_str(&format!("- **Total calls**: {}\n", stats.total_calls));
        report.push_str(&format!(
            "- **Resolved**: {} ({:.1}%)\n",
            stats.resolved_calls,
            stats.resolution_rate * 100.0
        ));
        report.push_str(&format!("- **Unresolved**: {}\n", stats.unresolved_calls));
        report.push_str(&format!(
            "- **Average confidence**: {:.1}%\n",
            stats.average_confidence * 100.0
        ));

        report.push_str("\n### Resolution Breakdown\n\n");
        report.push_str(&format!("- Exact: {}\n", stats.exact_count));
        report.push_str(&format!("- Inferred: {}\n", stats.inferred_count));
        report.push_str(&format!("- Heuristic: {}\n", stats.heuristic_count));
        report.push_str(&format!("- Ambiguous: {}\n", stats.ambiguous_count));

        report
    }
}
