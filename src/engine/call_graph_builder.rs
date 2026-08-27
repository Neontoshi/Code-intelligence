// src/engine/call_graph_builder.rs

use crate::graph::call_graph::{CallEdge, CallGraph, FunctionNode};
use crate::graph::resolution::ResolutionConfidence;
use crate::parser::tree_sitter::{CallSite as ParserCallSite, FunctionInfo, ParsedFile};
use crate::resolution::call_site::{
    CallKind, CallSite as SemanticCallSite, CalleeExpr, SourceLocation,
};
use crate::resolution::context::Language;
use crate::resolution::index_builder::IndexBuilder;
use crate::resolution::result::ResolutionStatus;
use crate::resolution::symbol::{FileId, ModuleId, ScopeId, SymbolId};
use crate::resolution::ResolutionEngine;
use std::collections::HashMap;

pub struct CallGraphBuilder;

impl CallGraphBuilder {
    pub fn build(files: &[ParsedFile]) -> CallGraph {
        let mut call_graph = CallGraph::new();
        let mut external_calls: HashMap<String, Vec<String>> = HashMap::new();

        // IndexBuilder is the SOLE owner of symbols and scopes
        let (index, scopes, type_context) = IndexBuilder::build(files);
        let mut engine = ResolutionEngine::new();
        engine.index = index;
        engine.scopes = scopes;
        engine.type_context = type_context;

        // Pass 1: Add function nodes to the call graph
        for file in files {
            for func in &file.functions {
                let full_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file.path, c, func.name),
                    None => format!("{}::{}", file.path, func.name),
                };

                let node = FunctionNode {
                    name: func.name.clone(),
                    full_path: full_path.clone(),
                    file: file.path.clone(),
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

                let _ = call_graph.add_function(node);
            }
        }

        // Pass 2: Resolve calls and build edges
        for file in files {
            let file_path = file.path.clone();
            let file_id = FileId(file_path.clone());
            let language = Language::from_file_extension(file.path.split('.').last().unwrap_or(""))
                .unwrap_or(Language::Rust);
            let module_id = ModuleId(IndexBuilder::file_to_module_path(&file_path));

            for func in &file.functions {
                let caller_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };

                if let Some(&caller_idx) = call_graph.name_index.get(&caller_path) {
                    let function_id = SymbolId(caller_path.clone());
                    let scope_id = ScopeId(format!("scope_{}", caller_path));

                    for parser_call in &func.calls {
                        let semantic_call = convert_call_site(parser_call, &file_path, func);

                        // Skip external dependency calls (now handles all cases)
                        if engine.is_external_call(&semantic_call.callee, &language, &file_id) {
                            let display = format!("{:?}", semantic_call.callee);
                            external_calls
                                .entry(caller_path.clone())
                                .or_default()
                                .push(display);
                            continue;
                        }

                        let result = engine.infer_and_resolve_call(
                            &semantic_call,
                            &file_id,
                            &function_id,
                            &scope_id,
                            &language,
                            &module_id,
                        );

                        match result.status {
                            ResolutionStatus::Resolved => {
                                if let Some(target) = result.target {
                                    if let Some(&callee_idx) = call_graph.name_index.get(&target.0)
                                    {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: format!("{:?}", result.method),
                                                line: func.line,
                                            },
                                        );
                                        call_graph.set_resolution_confidence(
                                            &caller_path,
                                            &target.0,
                                            match result.confidence {
                                                c if c >= 0.90 => ResolutionConfidence::Exact,
                                                c if c >= 0.70 => ResolutionConfidence::Inferred,
                                                _ => ResolutionConfidence::Heuristic,
                                            },
                                        );
                                    }
                                }
                            }
                            ResolutionStatus::External => {
                                // Track external dependencies for analytics
                                let display = format!("{:?}", semantic_call.callee);
                                external_calls
                                    .entry(caller_path.clone())
                                    .or_default()
                                    .push(display);
                            }
                            ResolutionStatus::Ambiguous
                            | ResolutionStatus::Dynamic
                            | ResolutionStatus::Unresolved => {
                                let display = format!("{:?}", semantic_call.callee);

                                static DEBUG_COUNT: std::sync::atomic::AtomicUsize =
                                    std::sync::atomic::AtomicUsize::new(0);
                                let debug_count =
                                    DEBUG_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                if debug_count < 50 {
                                    eprintln!(
                                        "UNRESOLVED: file={} caller={} callee={:?} kind={:?} status={:?}",
                                        file_path,
                                        caller_path,
                                        semantic_call.callee,
                                        semantic_call.kind,
                                        result.status,
                                    );
                                }

                                call_graph.mark_unresolved(&caller_path, &display);
                            }
                        }
                    }
                }
            }
        }

        // Add external calls to the call graph for analytics
        for (caller, callees) in external_calls {
            for callee in callees {
                call_graph.mark_external(&caller, &callee);
            }
        }

        call_graph
    }
}

fn convert_call_site(
    parser_call: &ParserCallSite,
    file_path: &str,
    func: &FunctionInfo,
) -> SemanticCallSite {
    let (kind, callee) = match parser_call {
        ParserCallSite::Bare(name) => {
            // Check if it's a constructor call (capitalized name)
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                (CallKind::Constructor, CalleeExpr::Name(name.clone()))
            } else {
                (CallKind::Function, CalleeExpr::Name(name.clone()))
            }
        }
        ParserCallSite::Qualified(module, name) => {
            let mut parts: Vec<String> = module.split("::").map(|s| s.to_string()).collect();
            parts.push(name.clone());

            // Check if it's a type constructor (e.g., Vec::new(), PathBuf::from())
            if module.chars().next().map_or(false, |c| c.is_uppercase()) {
                (CallKind::Constructor, CalleeExpr::Qualified(parts))
            } else {
                (CallKind::Function, CalleeExpr::Qualified(parts))
            }
        }
        ParserCallSite::SelfMethod(method) => (
            CallKind::Method,
            CalleeExpr::Member {
                receiver: Box::new(CalleeExpr::Name("self".to_string())),
                member: method.clone(),
            },
        ),
        ParserCallSite::OnReceiver(receiver, method) => (
            CallKind::Method,
            CalleeExpr::Member {
                receiver: Box::new(CalleeExpr::Name(receiver.clone())),
                member: method.clone(),
            },
        ),
        ParserCallSite::Chained(method) => (CallKind::Method, CalleeExpr::Unknown(method.clone())),
    };

    SemanticCallSite {
        kind,
        callee,
        location: SourceLocation {
            file: file_path.to_string(),
            line: func.line,
            column: 0,
        },
        receiver_type: None,
        scope: Vec::new(),
    }
}
