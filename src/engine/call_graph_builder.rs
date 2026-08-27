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

        // module path index, built from the files we were given.
        let mut file_module_index: HashMap<String, String> = HashMap::new();
        let mut module_to_file: HashMap<String, String> = HashMap::new();
        for file in files {
            let module = Self::file_to_module_path(&file.path);
            module_to_file.insert(module.clone(), file.path.clone());
            file_module_index.insert(file.path.clone(), module);
        }

        // Build module alias map: maps short alias -> full module path
        // e.g., "rust" -> "crate::parser::languages::rust"
        //       "fs" -> "std::fs"
        let mut module_alias_map: HashMap<String, String> = HashMap::new();

        // First pass: Build import map and module alias map
        for file in files {
            let current_module = &file_module_index[&file.path];
            for import in &file.imports {
                let resolved_module = Self::resolve_module_path(current_module, &import.module);

                // Store alias for the last segment of the module path
                let alias = resolved_module
                    .split("::")
                    .last()
                    .unwrap_or(&resolved_module)
                    .to_string();

                // Map both the alias and the full import string
                module_alias_map.insert(alias, resolved_module.clone());
                module_alias_map.insert(import.module.clone(), resolved_module.clone());

                if let Some(target_file) = module_to_file.get(&resolved_module) {
                    for item in &import.items {
                        let full_path = format!("{}::{}", target_file, item);
                        import_map.entry(item.clone()).or_default().push(full_path);
                    }
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
                                let current_module = &file_module_index[&file_path];

                                let resolved_module =
                                    Self::resolve_module_path(current_module, qualifier);
                                if let Some(target_file) = module_to_file.get(&resolved_module) {
                                    let candidate = format!("{}::{}", target_file, method_name);
                                    if let Some(&callee_idx) = func_index.get(&candidate) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "qualified_cross_file".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        call_graph.set_resolution_confidence(
                                            &caller_path,
                                            &candidate,
                                            ResolutionConfidence::Exact,
                                        );
                                        found = true;
                                    }
                                }

                                // Type::method() in the same file
                                if !found {
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
                                    }
                                }

                                // Look for TypeName across all files and find method in that file
                                if !found {
                                    if let Some(type_files) = type_definition_index.get(qualifier) {
                                        if type_files.len() == 1 {
                                            let target_file = &type_files[0];
                                            let candidate = format!(
                                                "{}::{}::{}",
                                                target_file, qualifier, method_name
                                            );
                                            if let Some(&callee_idx) = func_index.get(&candidate) {
                                                call_graph.add_call(
                                                    caller_idx,
                                                    callee_idx,
                                                    CallEdge {
                                                        call_type: "cross_file_type_method"
                                                            .to_string(),
                                                        line: func.line,
                                                    },
                                                );
                                                call_graph.set_resolution_confidence(
                                                    &caller_path,
                                                    &candidate,
                                                    ResolutionConfidence::Inferred,
                                                );
                                                found = true;
                                            } else {
                                                // Try without container (associated free function)
                                                let candidate2 =
                                                    format!("{}::{}", target_file, method_name);
                                                if let Some(&callee_idx) =
                                                    func_index.get(&candidate2)
                                                {
                                                    call_graph.add_call(
                                                        caller_idx,
                                                        callee_idx,
                                                        CallEdge {
                                                            call_type: "cross_file_type_associated"
                                                                .to_string(),
                                                            line: func.line,
                                                        },
                                                    );
                                                    call_graph.set_resolution_confidence(
                                                        &caller_path,
                                                        &candidate2,
                                                        ResolutionConfidence::Inferred,
                                                    );
                                                    found = true;
                                                }
                                            }
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

                                // TIER 3.5: Name match using name_to_functions index
                                if !found {
                                    if let Some(candidates) =
                                        call_graph.name_to_functions.get(simple_name)
                                    {
                                        let mut unique_paths: Vec<String> = Vec::new();
                                        for &idx in candidates {
                                            if call_graph[idx].full_path != caller_path {
                                                unique_paths
                                                    .push(call_graph[idx].full_path.clone());
                                            }
                                        }

                                        if unique_paths.len() == 1 {
                                            if let Some(&callee_idx) =
                                                func_index.get(&unique_paths[0])
                                            {
                                                call_graph.add_call(
                                                    caller_idx,
                                                    callee_idx,
                                                    CallEdge {
                                                        call_type: "name_index".to_string(),
                                                        line: func.line,
                                                    },
                                                );
                                                call_graph.set_resolution_confidence(
                                                    &caller_path,
                                                    &unique_paths[0],
                                                    ResolutionConfidence::Heuristic,
                                                );
                                                found = true;
                                            }
                                        } else if unique_paths.len() > 1 {
                                            // Try same file
                                            let same_file: Vec<String> = unique_paths
                                                .iter()
                                                .filter(|p| p.starts_with(&file_path))
                                                .cloned()
                                                .collect();

                                            if same_file.len() == 1 {
                                                if let Some(&callee_idx) =
                                                    func_index.get(&same_file[0])
                                                {
                                                    call_graph.add_call(
                                                        caller_idx,
                                                        callee_idx,
                                                        CallEdge {
                                                            call_type: "name_index_same_file"
                                                                .to_string(),
                                                            line: func.line,
                                                        },
                                                    );
                                                    call_graph.set_resolution_confidence(
                                                        &caller_path,
                                                        &same_file[0],
                                                        ResolutionConfidence::Heuristic,
                                                    );
                                                    found = true;
                                                }
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

                            // Try module alias resolution
                            let root = display_name
                                .split("::")
                                .next()
                                .unwrap_or(&display_name)
                                .to_string();

                            if let Some(resolved_module) = module_alias_map.get(&root) {
                                let rest = display_name.strip_prefix(&root).unwrap_or("");
                                let full_call = format!("{}{}", resolved_module, rest);

                                if let Some(&callee_idx) = func_index.get(&full_call) {
                                    call_graph.add_call(
                                        caller_idx,
                                        callee_idx,
                                        CallEdge {
                                            call_type: "module_alias".to_string(),
                                            line: func.line,
                                        },
                                    );
                                    call_graph.set_resolution_confidence(
                                        &caller_path,
                                        &full_call,
                                        ResolutionConfidence::Exact,
                                    );
                                    continue;
                                }
                            }

                            // UNIVERSAL: External if root module is not in the project
                            let is_external = {
                                !module_to_file.contains_key(&root)
                                    && !module_to_file.values().any(|f| f.contains(&root))
                            };

                            if is_external {
                                continue;
                            }

                            static DEBUG_COUNT: std::sync::atomic::AtomicUsize =
                                std::sync::atomic::AtomicUsize::new(0);
                            let debug_count =
                                DEBUG_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if debug_count < 50 {
                                eprintln!(
                                    "UNRESOLVED: file={} caller={} callee={} is_external={}",
                                    file_path, caller_path, display_name, is_external
                                );
                            }

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

    /// Convert a file path like "./src/graph/resolver.rs" into its Rust
    /// module path, e.g. "crate::graph::resolver". Handles mod.rs/main.rs/lib.rs.
    fn file_to_module_path(file_path: &str) -> String {
        let p = file_path.trim_start_matches("./");
        let rel = p.strip_prefix("src/").unwrap_or(p);
        let rel = rel.strip_suffix(".rs").unwrap_or(rel);
        let mut segments: Vec<&str> = rel.split('/').collect();
        if matches!(segments.last(), Some(&"mod") | Some(&"main") | Some(&"lib")) {
            segments.pop();
        }
        if segments.is_empty() {
            "crate".to_string()
        } else {
            format!("crate::{}", segments.join("::"))
        }
    }

    /// Resolve a raw `use`/qualifier path (which may start with `crate::`,
    /// `super::`, `self::`, or be a bare relative segment) into an absolute
    /// module path, relative to the module doing the referencing.
    fn resolve_module_path(current_module: &str, raw: &str) -> String {
        if raw == "crate" || raw.starts_with("crate::") {
            return raw.to_string();
        }
        if raw == "self" {
            return current_module.to_string();
        }
        if let Some(rest) = raw.strip_prefix("self::") {
            return format!("{}::{}", current_module, rest);
        }
        if raw == "super" || raw.starts_with("super") {
            let mut base: Vec<&str> = current_module.split("::").collect();
            let mut rest = raw;
            while rest == "super" || rest.starts_with("super::") || rest.starts_with("super") {
                base.pop();
                rest = rest.strip_prefix("super").unwrap_or(rest);
                rest = rest.strip_prefix("::").unwrap_or(rest);
                if !rest.starts_with("super") {
                    break;
                }
            }
            return if rest.is_empty() {
                base.join("::")
            } else {
                format!("{}::{}", base.join("::"), rest)
            };
        }
        // Bare path (sibling module referenced without `crate::`, or an
        // external crate). Left as-is — external crates won't be found in
        // module_to_file, which is the correct outcome.
        raw.to_string()
    }
}
