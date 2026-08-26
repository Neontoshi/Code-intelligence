// src/engine/call_graph_builder.rs

use crate::graph::call_graph::{CallEdge, CallGraph, FunctionNode};
use crate::graph::resolution::ResolutionConfidence;
use crate::parser::tree_sitter::{CallSite, ParsedFile};
use std::collections::HashMap;

/// Builds call graphs from parsed files
pub struct CallGraphBuilder;

impl CallGraphBuilder {
    /// Build a call graph from parsed files
    pub fn build(files: &[ParsedFile]) -> CallGraph {
        let mut call_graph = CallGraph::new();
        let mut func_index: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
        let mut func_by_name: HashMap<String, Vec<String>> = HashMap::new();
        let mut func_by_file: HashMap<String, Vec<String>> = HashMap::new();
        let mut import_map: HashMap<String, Vec<String>> = HashMap::new();

        // First pass: Build import map
        for file in files {
            for import in &file.imports {
                let module = &import.module;
                for item in &import.items {
                    let full_path = format!("{}::{}", module, item);
                    import_map.entry(item.clone()).or_default().push(full_path);
                }
            }
        }

        // Second pass: Add all functions and index them
        for file in files {
            let file_path = file.path.clone();
            for func in &file.functions {
                let full_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };

                // Calculate complexity from the function body
                let node = FunctionNode {
                    name: func.name.clone(),
                    full_path: full_path.clone(),
                    file: file_path.clone(),
                    line: func.line,
                    body_start_line: func.body_start_line,
                    body_end_line: func.body_end_line,
                    is_public: func.is_public,
                    is_async: func.is_async,
                    params: func.params.iter().map(|p| p.name.clone()).collect(),
                    returns: func.return_type.clone().into_iter().collect(),
                    complexity: func.complexity as f64,
                    importance_score: 0.0,
                    doc_comment: func.doc_comment.clone(),
                    writes_to: Vec::new(),
                    reads_from: Vec::new(),
                    errors: Vec::new(),
                    fan_in: 0,
                    fan_out: 0,
                    is_cycle: false,
                    depth: 0,
                    layer: String::new(),
                    trait_impl: func.trait_impl.clone(),
                    decorators: func.decorators.clone(),
                    is_test: func.is_test,
                    is_trait_method: func.is_trait_method,
                    is_trait_default: func.is_trait_default,
                };
                let idx = call_graph.add_function(node);
                func_index.insert(full_path.clone(), idx);
                func_by_name
                    .entry(func.name.clone())
                    .or_default()
                    .push(full_path.clone());
                func_by_file
                    .entry(file_path.clone())
                    .or_default()
                    .push(full_path);
            }
        }

        // Build type-to-implementations index for method resolution
        let mut type_to_impls: HashMap<String, Vec<String>> = HashMap::new();
        let mut method_to_impls: HashMap<String, Vec<String>> = HashMap::new();

        for (path, _) in &func_index {
            let parts: Vec<&str> = path.split("::").collect();
            if parts.len() >= 3 {
                // Format: file::Type::method
                let type_name = parts[1].to_string();
                let method_name = parts[2].to_string();

                // Index by type
                type_to_impls
                    .entry(type_name.clone())
                    .or_default()
                    .push(path.clone());

                // Index by method name (for resolving calls like `uf.union()`)
                method_to_impls
                    .entry(method_name)
                    .or_default()
                    .push(path.clone());
            }
        }

        // Build local variable type tracking from parser info
        let mut var_to_type: HashMap<String, String> = HashMap::new();

        // Build a map of variable names to their types by scanning source
        for file in files {
            let source = &file.source;

            // Pattern: `let variable_name: TypeName` or `let variable_name = TypeName::new()`
            let lines: Vec<&str> = source.lines().collect();
            for (_i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Pattern 1: `let var: Type`
                if trimmed.starts_with("let ") && trimmed.contains(':') {
                    let parts: Vec<&str> = trimmed.split(':').collect();
                    if parts.len() >= 2 {
                        let var_part = parts[0].trim().trim_start_matches("let ").trim();
                        let type_part = parts[1]
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim();

                        // Clean up type name (remove generics, lifetime annotations)
                        let clean_type = type_part
                            .split('<')
                            .next()
                            .unwrap_or(type_part)
                            .split('&')
                            .last()
                            .unwrap_or(type_part)
                            .trim();

                        if !clean_type.is_empty() && !clean_type.starts_with('_') {
                            var_to_type.insert(var_part.to_string(), clean_type.to_string());
                        }
                    }
                }

                // Pattern 2: `let var = Type::new()` or `let var = Type { ... }`
                if trimmed.starts_with("let ") && trimmed.contains("= ") {
                    let parts: Vec<&str> = trimmed.split("= ").collect();
                    if parts.len() >= 2 {
                        let var_part = parts[0].trim().trim_start_matches("let ").trim();
                        let init_part = parts[1].trim();

                        // Check if it's a constructor call: Type::new() or Type { ... }
                        if init_part.contains("::") {
                            let type_part = init_part.split("::").next().unwrap_or("").trim();
                            let clean_type =
                                type_part.split('<').next().unwrap_or(type_part).trim();

                            if !clean_type.is_empty() && !clean_type.starts_with('_') {
                                var_to_type.insert(var_part.to_string(), clean_type.to_string());
                            }
                        } else if init_part.contains('{') && !init_part.contains('.') {
                            let type_part = init_part.split('{').next().unwrap_or("").trim();
                            if !type_part.is_empty() && !type_part.starts_with('_') {
                                var_to_type.insert(var_part.to_string(), type_part.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Trait-method index for operator-overload resolution
        let mut trait_method_index: HashMap<(String, String), Vec<petgraph::graph::NodeIndex>> =
            HashMap::new();
        for idx in call_graph.node_indices() {
            let node = &call_graph[idx];
            if let Some(trait_name) = &node.trait_impl {
                let base = Self::base_trait_name(trait_name);
                trait_method_index
                    .entry((base, node.name.clone()))
                    .or_default()
                    .push(idx);
            }
        }

        // Build function name to full path mapping for internal calls
        let mut func_name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        for (path, _) in &func_index {
            if let Some(name) = path.split("::").last() {
                func_name_to_paths
                    .entry(name.to_string())
                    .or_default()
                    .push(path.clone());
            }
        }

        // Build container to functions mapping for impl blocks
        let mut container_to_functions: HashMap<String, Vec<String>> = HashMap::new();
        for (path, _) in &func_index {
            let parts: Vec<&str> = path.split("::").collect();
            if parts.len() >= 3 {
                let container = format!("{}::{}", parts[0], parts[1]);
                container_to_functions
                    .entry(container)
                    .or_default()
                    .push(path.clone());
            }
        }

        // Build type name -> type definition index
        let mut type_definition_index: HashMap<String, Vec<String>> = HashMap::new();
        for file in files {
            for type_info in &file.types {
                type_definition_index
                    .entry(type_info.name.clone())
                    .or_default()
                    .push(file.path.clone());
            }
        }

        // Third pass: Build edges with import resolution and internal call detection
        // Third pass: Build edges with import resolution and internal call detection
        for file in files {
            let file_path = file.path.clone();
            for func in &file.functions {
                let caller_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };
                if let Some(&caller_idx) = func_index.get(&caller_path) {
                    for call_site in &func.calls {
                        let mut found = false;

                        match call_site {
                            CallSite::SelfMethod(method_name) => {
                                // Direct method call on self/this
                                if let Some(container) = &func.container {
                                    let full_path =
                                        format!("{}::{}::{}", file_path, container, method_name);
                                    if let Some(&callee_idx) = func_index.get(&full_path) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "self_method".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        call_graph.set_resolution_confidence(
                                            &caller_path,
                                            &full_path,
                                            ResolutionConfidence::Exact,
                                        );
                                        found = true;
                                    }
                                }
                            }

                            CallSite::Qualified(qualifier, method_name) => {
                                // Type::method() or namespace::function()
                                let qualified_path =
                                    format!("{}::{}::{}", file_path, qualifier, method_name);
                                if let Some(&callee_idx) = func_index.get(&qualified_path) {
                                    call_graph.add_call(
                                        caller_idx,
                                        callee_idx,
                                        CallEdge {
                                            call_type: "exact".to_string(),
                                            line: func.line,
                                        },
                                    );
                                    call_graph.set_resolution_confidence(
                                        &caller_path,
                                        &qualified_path,
                                        ResolutionConfidence::Exact,
                                    );
                                    found = true;
                                } else {
                                    // Try as constructor
                                    let constructor_path =
                                        format!("{}::{}::{}", file_path, qualifier, method_name);
                                    if method_name == "new"
                                        || method_name == "default"
                                        || method_name == "from"
                                        || method_name == "with_capacity"
                                    {
                                        if let Some(&callee_idx) = func_index.get(&constructor_path)
                                        {
                                            call_graph.add_call(
                                                caller_idx,
                                                callee_idx,
                                                CallEdge {
                                                    call_type: "constructor".to_string(),
                                                    line: func.line,
                                                },
                                            );
                                            call_graph.set_resolution_confidence(
                                                &caller_path,
                                                &constructor_path,
                                                ResolutionConfidence::Exact,
                                            );
                                            found = true;
                                        }
                                    }
                                }
                            }

                            CallSite::OnReceiver(receiver, method_name) => {
                                // variable.method() - resolve receiver type
                                if let Some(var_type) = var_to_type.get(receiver) {
                                    let type_method = format!("{}::{}", var_type, method_name);
                                    if let Some(&callee_idx) = func_index.get(&type_method) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "method_call_resolved".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        call_graph.set_resolution_confidence(
                                            &caller_path,
                                            &type_method,
                                            ResolutionConfidence::Inferred,
                                        );
                                        found = true;
                                    }
                                }

                                // Try to find method in the same container
                                if !found {
                                    if let Some(container) = &func.container {
                                        let full_path = format!(
                                            "{}::{}::{}",
                                            file_path, container, method_name
                                        );
                                        if let Some(&callee_idx) = func_index.get(&full_path) {
                                            call_graph.add_call(
                                                caller_idx,
                                                callee_idx,
                                                CallEdge {
                                                    call_type: "method_call_container".to_string(),
                                                    line: func.line,
                                                },
                                            );
                                            call_graph.set_resolution_confidence(
                                                &caller_path,
                                                &full_path,
                                                ResolutionConfidence::Inferred,
                                            );
                                            found = true;
                                        }
                                    }
                                }

                                // Try to find method by name (unambiguous)
                                if !found {
                                    if let Some(paths) = method_to_impls.get(method_name) {
                                        if paths.len() == 1 {
                                            if let Some(&callee_idx) = func_index.get(&paths[0]) {
                                                call_graph.add_call(
                                                    caller_idx,
                                                    callee_idx,
                                                    CallEdge {
                                                        call_type: "method_call_by_name"
                                                            .to_string(),
                                                        line: func.line,
                                                    },
                                                );
                                                call_graph.set_resolution_confidence(
                                                    &caller_path,
                                                    &paths[0],
                                                    ResolutionConfidence::Heuristic,
                                                );
                                                found = true;
                                            }
                                        }
                                    }
                                }

                                // Try to infer from context (if receiver is capitalized)
                                if !found
                                    && receiver
                                        .chars()
                                        .next()
                                        .map(|c| c.is_uppercase())
                                        .unwrap_or(false)
                                {
                                    let type_method = format!("{}::{}", receiver, method_name);
                                    if let Some(&callee_idx) = func_index.get(&type_method) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "method_call_type".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        call_graph.set_resolution_confidence(
                                            &caller_path,
                                            &type_method,
                                            ResolutionConfidence::Inferred,
                                        );
                                        found = true;
                                    }
                                }
                            }

                            CallSite::Chained(method_name) => {
                                // Chained call - the inner call is resolved separately
                                // Try to resolve just this method
                                if let Some(paths) = method_to_impls.get(method_name) {
                                    if paths.len() == 1 {
                                        if let Some(&callee_idx) = func_index.get(&paths[0]) {
                                            call_graph.add_call(
                                                caller_idx,
                                                callee_idx,
                                                CallEdge {
                                                    call_type: "chained_method".to_string(),
                                                    line: func.line,
                                                },
                                            );
                                            call_graph.set_resolution_confidence(
                                                &caller_path,
                                                &paths[0],
                                                ResolutionConfidence::Heuristic,
                                            );
                                            found = true;
                                        }
                                    }
                                }

                                // Try same container
                                if !found {
                                    if let Some(container) = &func.container {
                                        let full_path = format!(
                                            "{}::{}::{}",
                                            file_path, container, method_name
                                        );
                                        if let Some(&callee_idx) = func_index.get(&full_path) {
                                            call_graph.add_call(
                                                caller_idx,
                                                callee_idx,
                                                CallEdge {
                                                    call_type: "chained_container".to_string(),
                                                    line: func.line,
                                                },
                                            );
                                            call_graph.set_resolution_confidence(
                                                &caller_path,
                                                &full_path,
                                                ResolutionConfidence::Inferred,
                                            );
                                            found = true;
                                        }
                                    }
                                }
                            }

                            CallSite::Bare(name) => {
                                // Simple function call - use existing resolution
                                let simple_name = name.as_str();

                                // TIER 2: Internal calls within the same file
                                let candidates: Vec<String> = func_by_file
                                    .get(&file_path)
                                    .unwrap_or(&vec![])
                                    .iter()
                                    .filter(|path| {
                                        path.ends_with(&format!("::{}", simple_name))
                                            && *path != &caller_path
                                    })
                                    .cloned()
                                    .collect();

                                if candidates.len() == 1 {
                                    if let Some(&callee_idx) = func_index.get(&candidates[0]) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "internal_call".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        call_graph.set_resolution_confidence(
                                            &caller_path,
                                            &candidates[0],
                                            ResolutionConfidence::Exact,
                                        );
                                        found = true;
                                    }
                                } else if candidates.len() > 1 {
                                    if let Some(container) = &func.container {
                                        let container_key = format!("{}::{}", file_path, container);
                                        if let Some(container_funcs) =
                                            container_to_functions.get(&container_key)
                                        {
                                            let container_candidates: Vec<_> = candidates
                                                .iter()
                                                .filter(|path| container_funcs.contains(*path))
                                                .collect();
                                            if container_candidates.len() == 1 {
                                                if let Some(&callee_idx) =
                                                    func_index.get(container_candidates[0])
                                                {
                                                    call_graph.add_call(
                                                        caller_idx,
                                                        callee_idx,
                                                        CallEdge {
                                                            call_type: "internal_call".to_string(),
                                                            line: func.line,
                                                        },
                                                    );
                                                    call_graph.set_resolution_confidence(
                                                        &caller_path,
                                                        container_candidates[0],
                                                        ResolutionConfidence::Exact,
                                                    );
                                                    found = true;
                                                }
                                            }
                                        }
                                    }
                                }

                                // TIER 3: Import resolution
                                if !found {
                                    if let Some(imported_paths) = import_map.get(simple_name) {
                                        for imported_path in imported_paths {
                                            if let Some(&callee_idx) = func_index.get(imported_path)
                                            {
                                                call_graph.add_call(
                                                    caller_idx,
                                                    callee_idx,
                                                    CallEdge {
                                                        call_type: "imported".to_string(),
                                                        line: func.line,
                                                    },
                                                );
                                                call_graph.set_resolution_confidence(
                                                    &caller_path,
                                                    imported_path,
                                                    ResolutionConfidence::Inferred,
                                                );
                                                found = true;
                                                break;
                                            }
                                        }
                                    }
                                }

                                // TIER 4: Name match across files (only if unambiguous)
                                if !found {
                                    if let Some(paths) = func_by_name.get(simple_name) {
                                        let candidates: Vec<_> =
                                            paths.iter().filter(|p| *p != &caller_path).collect();
                                        if candidates.len() == 1 {
                                            if let Some(&callee_idx) = func_index.get(candidates[0])
                                            {
                                                call_graph.add_call(
                                                    caller_idx,
                                                    callee_idx,
                                                    CallEdge {
                                                        call_type: "by_name".to_string(),
                                                        line: func.line,
                                                    },
                                                );
                                                call_graph.set_resolution_confidence(
                                                    &caller_path,
                                                    candidates[0],
                                                    ResolutionConfidence::Heuristic,
                                                );
                                                found = true;
                                            }
                                        }
                                    }
                                }

                                // TIER 5: Self reference (functions calling themselves)
                                if !found && simple_name == func.name {
                                    if let Some(&callee_idx) = func_index.get(&caller_path) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "recursive".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        call_graph.set_resolution_confidence(
                                            &caller_path,
                                            &caller_path,
                                            ResolutionConfidence::Exact,
                                        );
                                        found = true;
                                    }
                                }

                                // TIER 6: Function calls within the same container
                                if !found {
                                    if let Some(container) = &func.container {
                                        let full_path = format!(
                                            "{}::{}::{}",
                                            file_path, container, simple_name
                                        );
                                        if let Some(&callee_idx) = func_index.get(&full_path) {
                                            call_graph.add_call(
                                                caller_idx,
                                                callee_idx,
                                                CallEdge {
                                                    call_type: "container_method".to_string(),
                                                    line: func.line,
                                                },
                                            );
                                            call_graph.set_resolution_confidence(
                                                &caller_path,
                                                &full_path,
                                                ResolutionConfidence::Inferred,
                                            );
                                            found = true;
                                        }
                                    }
                                }
                            }
                        }

                        if !found {
                            let display_name = match call_site {
                                CallSite::SelfMethod(m) => format!("self.{}", m),
                                CallSite::Qualified(q, m) => format!("{}::{}", q, m),
                                CallSite::OnReceiver(r, m) => format!("{}.{}", r, m),
                                CallSite::Chained(m) => format!("().{}", m),
                                CallSite::Bare(n) => n.clone(),
                            };
                            call_graph.mark_unresolved(&caller_path, &display_name);
                        }
                    }
                }
            }
        }
        call_graph
    }

    fn base_trait_name(raw: &str) -> String {
        let no_generics = raw.split('<').next().unwrap_or(raw).trim();
        no_generics
            .rsplit("::")
            .next()
            .unwrap_or(no_generics)
            .to_string()
    }
}
