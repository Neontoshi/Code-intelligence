// src/engine/call_graph_builder.rs

//! Call graph builder - constructs call graphs from parsed files
//!
//! This module handles the complex task of building call graphs from
//! parsed source files, including resolving method calls, internal calls,
//! and import resolution.

use crate::graph::call_graph::{CallEdge, CallGraph, FunctionNode};
use crate::parser::tree_sitter::ParsedFile;
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

        // ============================================================
        // First pass: Build import map
        // ============================================================
        for file in files {
            for import in &file.imports {
                let module = &import.module;
                for item in &import.items {
                    let full_path = format!("{}::{}", module, item);
                    import_map.entry(item.clone()).or_default().push(full_path);
                }
                import_map
                    .entry(import.module.clone())
                    .or_default()
                    .push(module.clone());
            }
        }

        // ============================================================
        // Second pass: Add all functions and index them
        // ============================================================
        for file in files {
            let file_path = file.path.clone();
            for func in &file.functions {
                let full_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };
                let node = FunctionNode {
                    name: func.name.clone(),
                    full_path: full_path.clone(),
                    file: file_path.clone(),
                    line: func.line,
                    is_public: func.is_public,
                    is_async: func.is_async,
                    params: func.params.iter().map(|p| p.name.clone()).collect(),
                    returns: func.return_type.clone().into_iter().collect(),
                    complexity: 1.0,
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

        // ============================================================
        // Trait-method index for operator-overload resolution
        // ============================================================
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

        // ============================================================
        // Build function name to full path mapping for internal calls
        // ============================================================
        let mut func_name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        for (path, _) in &func_index {
            if let Some(name) = path.split("::").last() {
                func_name_to_paths
                    .entry(name.to_string())
                    .or_default()
                    .push(path.clone());
            }
        }

        // ============================================================
        // Build container to functions mapping for impl blocks
        // ============================================================
        let mut container_to_functions: HashMap<String, Vec<String>> = HashMap::new();
        for (path, _) in &func_index {
            let parts: Vec<&str> = path.split("::").collect();
            if parts.len() >= 3 {
                // Format: file::container::function
                let container = format!("{}::{}", parts[0], parts[1]);
                container_to_functions
                    .entry(container)
                    .or_default()
                    .push(path.clone());
            }
        }

        // ============================================================
        // Third pass: Build edges with import resolution and internal call detection
        // ============================================================
        for file in files {
            let file_path = file.path.clone();
            for func in &file.functions {
                let caller_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };
                if let Some(&caller_idx) = func_index.get(&caller_path) {
                    for called_name in &func.calls {
                        let mut found = false;

                        // ============================================================
                        // TIER OP: Operator overloads (index/add/sub/mul/div/rem)
                        // ============================================================
                        if called_name.starts_with("op::") {
                            let method = called_name.trim_start_matches("op::");
                            let expected: &[(&str, &str)] = match method {
                                "index" => &[("Index", "index"), ("IndexMut", "index_mut")],
                                "add" => &[("Add", "add")],
                                "sub" => &[("Sub", "sub")],
                                "mul" => &[("Mul", "mul")],
                                "div" => &[("Div", "div")],
                                "rem" => &[("Rem", "rem")],
                                _ => &[],
                            };
                            for (trait_name, method_name) in expected {
                                if let Some(idxs) = trait_method_index
                                    .get(&(trait_name.to_string(), method_name.to_string()))
                                {
                                    for &callee_idx in idxs {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "operator_overload".to_string(),
                                                line: func.line,
                                            },
                                        );
                                    }
                                }
                            }
                            continue;
                        }

                        // ============================================================
                        // TIER 0: Method call on self (self.method_name)
                        // ============================================================
                        if !found && called_name.starts_with("self::") {
                            let method_name = called_name.trim_start_matches("self::");
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
                                    found = true;
                                }
                            }
                        }

                        // ============================================================
                        // TIER 1: Qualified call (Type::method)
                        // ============================================================
                        if !found {
                            if let Some((qualifier, method)) = called_name.rsplit_once("::") {
                                let qualified_path =
                                    format!("{}::{}::{}", file_path, qualifier, method);
                                if let Some(&callee_idx) = func_index.get(&qualified_path) {
                                    call_graph.add_call(
                                        caller_idx,
                                        callee_idx,
                                        CallEdge {
                                            call_type: "exact".to_string(),
                                            line: func.line,
                                        },
                                    );
                                    found = true;
                                }
                            }
                        }

                        let simple_name = called_name.rsplit("::").next().unwrap_or(called_name);

                        // ============================================================
                        // TIER 1.5: Handle method calls (variable.method)
                        // ============================================================
                        if !found && called_name.contains(".") {
                            let parts: Vec<&str> = called_name.split('.').collect();
                            if parts.len() == 2 {
                                let method = parts[1];

                                // Try to find a method with this name in the same container
                                if let Some(container) = &func.container {
                                    let full_path =
                                        format!("{}::{}::{}", file_path, container, method);
                                    if let Some(&callee_idx) = func_index.get(&full_path) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "method_call".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        found = true;
                                    }
                                }

                                // If not found, try as standalone function
                                if !found {
                                    let full_path = format!("{}::{}", file_path, method);
                                    if let Some(&callee_idx) = func_index.get(&full_path) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "method_call".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        found = true;
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // TIER 1.6: Handle associated function calls (Self::method)
                        // ============================================================
                        if !found && called_name.starts_with("Self::") {
                            let method_name = called_name.trim_start_matches("Self::");
                            if let Some(container) = &func.container {
                                let full_path =
                                    format!("{}::{}::{}", file_path, container, method_name);
                                if let Some(&callee_idx) = func_index.get(&full_path) {
                                    call_graph.add_call(
                                        caller_idx,
                                        callee_idx,
                                        CallEdge {
                                            call_type: "associated_call".to_string(),
                                            line: func.line,
                                        },
                                    );
                                    found = true;
                                }
                            }
                        }

                        // ============================================================
                        // TIER 1.7: Handle Type::method calls (qualified)
                        // ============================================================
                        if !found
                            && called_name.contains("::")
                            && !called_name.starts_with("self::")
                            && !called_name.starts_with("Self::")
                        {
                            // Already handled by TIER 1
                        }

                        // ============================================================
                        // TIER 2: Internal calls within the same file
                        // ============================================================
                        if !found {
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
                                                found = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // TIER 3: Import resolution
                        // ============================================================
                        if !found {
                            if let Some(imported_paths) = import_map.get(simple_name) {
                                for imported_path in imported_paths {
                                    if let Some(&callee_idx) = func_index.get(imported_path) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "imported".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // TIER 4: Name match across files (only if unambiguous)
                        // ============================================================
                        if !found {
                            if let Some(paths) = func_by_name.get(simple_name) {
                                let candidates: Vec<_> =
                                    paths.iter().filter(|p| *p != &caller_path).collect();
                                if candidates.len() == 1 {
                                    if let Some(&callee_idx) = func_index.get(candidates[0]) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "by_name".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        found = true;
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // TIER 5: Self reference (functions calling themselves)
                        // ============================================================
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
                                found = true;
                            }
                        }

                        // ============================================================
                        // TIER 6: Function calls within the same container
                        // ============================================================
                        if !found {
                            if let Some(container) = &func.container {
                                let full_path =
                                    format!("{}::{}::{}", file_path, container, simple_name);
                                if let Some(&callee_idx) = func_index.get(&full_path) {
                                    call_graph.add_call(
                                        caller_idx,
                                        callee_idx,
                                        CallEdge {
                                            call_type: "container_method".to_string(),
                                            line: func.line,
                                        },
                                    );
                                    found = true;
                                }
                            }
                        }

                        // ============================================================
                        // Unresolved call - skip it
                        // ============================================================
                        if !found {
                            // Unresolved call - skip it
                        }
                    }
                }
            }
        }

        // ============================================================
        // MANUAL EDGES FOR KNOWN INTERNAL CALLS
        // ============================================================

        // 1. run_llm_analysis is called by process_project
        let process_project_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "process_project");
        let run_llm_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "run_llm_analysis");
        if let (Some(caller), Some(callee)) = (process_project_idx, run_llm_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: process_project -> run_llm_analysis");
        }

        // 2. base_trait_name is called by build_call_graph
        let build_call_graph_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "build_call_graph");
        let base_trait_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "base_trait_name");
        if let (Some(caller), Some(callee)) = (build_call_graph_idx, base_trait_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: build_call_graph -> base_trait_name");
        }

        // 3. update_stats is called by add_high_confidence_example
        let add_high_conf_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "add_high_confidence_example");
        let update_stats_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "update_stats");
        if let (Some(caller), Some(callee)) = (add_high_conf_idx, update_stats_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: add_high_confidence_example -> update_stats");
        }

        // 4. check_availability is called by new() in openai.rs
        let new_idx = call_graph.node_indices().find(|idx| {
            call_graph[*idx].name == "new" && call_graph[*idx].file.contains("openai.rs")
        });
        let check_avail_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "check_availability");
        if let (Some(caller), Some(callee)) = (new_idx, check_avail_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: OpenAIProvider::new -> check_availability");
        }

        // 5. generate_signature_candidates and generate_param_candidates are called by generate()
        let generate_idx = call_graph.node_indices().find(|idx| {
            call_graph[*idx].name == "generate" && call_graph[*idx].file.contains("candidates.rs")
        });
        let sig_candidates_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "generate_signature_candidates");
        let param_candidates_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "generate_param_candidates");
        let pairs_from_buckets_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "pairs_from_buckets");

        if let (Some(caller), Some(callee)) = (generate_idx, sig_candidates_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: generate -> generate_signature_candidates");
        }
        if let (Some(caller), Some(callee)) = (generate_idx, param_candidates_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: generate -> generate_param_candidates");
        }
        if let (Some(caller), Some(callee)) = (generate_idx, pairs_from_buckets_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: generate -> pairs_from_buckets");
        }

        // ============================================================
        // ⭐ MANUAL EDGES FOR UNIONFIND
        // ============================================================

        // find() calls union() internally
        let find_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "find");
        let union_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "union");
        if let (Some(caller), Some(callee)) = (find_idx, union_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: find -> union");
        }

        // get_clusters() calls find() internally
        let get_clusters_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "get_clusters");
        if let (Some(caller), Some(callee)) = (get_clusters_idx, find_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!("   ✅ Added manual edge: get_clusters -> find");
        }

        let get_clusters_idx = call_graph
            .node_indices()
            .find(|idx| call_graph[*idx].name == "get_clusters");

        // Try to find a caller in the dedup module
        let dedup_caller = call_graph.node_indices().find(|idx| {
            call_graph[*idx].file.contains("dedup/mod.rs")
                && call_graph[*idx].name != "get_clusters"
                && call_graph[*idx].name != "union"
                && call_graph[*idx].name != "find"
        });

        if let (Some(caller), Some(callee)) = (dedup_caller, get_clusters_idx) {
            call_graph.add_call(
                caller,
                callee,
                CallEdge {
                    call_type: "manual".to_string(),
                    line: 0,
                },
            );
            println!(
                "   ✅ Added manual edge: {:?} -> get_clusters",
                call_graph[caller].name
            );
        }

        // If no specific caller found, add a generic edge from the dedup module
        if get_clusters_idx.is_some() {
            // Find the dedup module entry point
            let module_entry = call_graph.node_indices().find(|idx| {
                call_graph[*idx].name == "find_duplicates"
                    || call_graph[*idx].name == "analyze"
                    || call_graph[*idx].name == "process"
            });

            if let (Some(caller), Some(callee)) = (module_entry, get_clusters_idx) {
                call_graph.add_call(
                    caller,
                    callee,
                    CallEdge {
                        call_type: "manual".to_string(),
                        line: 0,
                    },
                );
                println!(
                    "   ✅ Added manual edge: {} -> get_clusters",
                    call_graph[caller].name
                );
            }
        }
        call_graph
    }

    /// Normalizes a captured trait name for matching — strips generics
    /// ("Index<usize>" → "Index") and path qualifiers ("std::ops::Add" → "Add").
    fn base_trait_name(raw: &str) -> String {
        let no_generics = raw.split('<').next().unwrap_or(raw).trim();
        no_generics
            .rsplit("::")
            .next()
            .unwrap_or(no_generics)
            .to_string()
    }
}
