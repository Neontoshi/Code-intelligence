use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SemanticAnalysis {
    pub function_roles: HashMap<String, FunctionRole>,
    pub data_flows: Vec<DataFlow>,
    pub error_propagation: Vec<ErrorPath>,
    pub concurrency_patterns: Vec<ConcurrencyPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionRole {
    EntryPoint,
    Handler,
    Service,
    Repository,
    Utility,
    Validator,
    Factory,
    Converter,
    Middleware,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DataFlow {
    pub source: String,
    pub target: String,
    pub path: Vec<String>,
    pub data_type: String,
}

#[derive(Debug, Clone)]
pub struct ErrorPath {
    pub source: String,
    pub target: String,
    pub error_type: String,
    pub handling: ErrorHandling,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorHandling {
    Propagate,
    Handle,
    Ignore,
    Panic,
}

#[derive(Debug, Clone)]
pub struct ConcurrencyPattern {
    pub pattern_type: ConcurrencyType,
    pub functions: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConcurrencyType {
    Parallel,
    Async,
    Threaded,
    Mutex,
    Channel,
    Tokio,
}

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn analyze(call_graph: &CallGraph, files: &[ParsedFile]) -> SemanticAnalysis {
        let function_roles = Self::detect_function_roles(call_graph, files);
        let data_flows = Self::analyze_data_flows(call_graph, files);
        let error_propagation = Self::analyze_error_propagation(call_graph, files);
        let concurrency_patterns = Self::detect_concurrency_patterns(call_graph, files);

        SemanticAnalysis {
            function_roles,
            data_flows,
            error_propagation,
            concurrency_patterns,
        }
    }

    fn detect_function_roles(
        call_graph: &CallGraph,
        _files: &[ParsedFile],
    ) -> HashMap<String, FunctionRole> {
        let mut roles = HashMap::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let name = func.name.to_lowercase();
            let role = if name.contains("main") || name.contains("entry") {
                FunctionRole::EntryPoint
            } else if name.contains("handler") || name.contains("controller") {
                FunctionRole::Handler
            } else if name.contains("service") || name.contains("domain") {
                FunctionRole::Service
            } else if name.contains("repo") || name.contains("repository") || name.contains("dao") {
                FunctionRole::Repository
            } else if name.contains("util") || name.contains("helper") {
                FunctionRole::Utility
            } else if name.contains("validate") || name.contains("check") {
                FunctionRole::Validator
            } else if name.contains("factory") || name.contains("create") || name.contains("build")
            {
                FunctionRole::Factory
            } else if name.contains("convert") || name.contains("transform") || name.contains("map")
            {
                FunctionRole::Converter
            } else if name.contains("middleware") {
                FunctionRole::Middleware
            } else {
                FunctionRole::Unknown
            };

            roles.insert(func.full_path.clone(), role);
        }

        roles
    }

    fn analyze_data_flows(call_graph: &CallGraph, _files: &[ParsedFile]) -> Vec<DataFlow> {
        let mut flows = Vec::new();

        // Detect data flow through function calls
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let callees = call_graph.get_callees(idx);

            for callee in callees {
                // Check if data flows from func to callee
                // In a real implementation, we'd analyze parameters and return types
                let flow = DataFlow {
                    source: func.full_path.clone(),
                    target: callee.full_path.clone(),
                    path: vec![func.name.clone(), callee.name.clone()],
                    data_type: "unknown".to_string(),
                };
                flows.push(flow);
            }
        }

        flows
    }

    fn analyze_error_propagation(call_graph: &CallGraph, _files: &[ParsedFile]) -> Vec<ErrorPath> {
        let mut errors = Vec::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Check if function handles errors
            let has_try = func
                .doc_comment
                .as_ref()
                .map(|d| d.contains("try") || d.contains("?"))
                .unwrap_or(false);

            let handling = if has_try || func.name.contains("try") {
                ErrorHandling::Propagate
            } else if func.name.contains("handle") || func.name.contains("catch") {
                ErrorHandling::Handle
            } else if func.name.contains("panic") || func.name.contains("unwrap") {
                ErrorHandling::Panic
            } else {
                ErrorHandling::Ignore
            };

            if !matches!(handling, ErrorHandling::Ignore) {
                let error_path = ErrorPath {
                    source: func.full_path.clone(),
                    target: func.full_path.clone(),
                    error_type: "error".to_string(),
                    handling,
                };
                errors.push(error_path);
            }
        }

        errors
    }

    fn detect_concurrency_patterns(
        call_graph: &CallGraph,
        _files: &[ParsedFile],
    ) -> Vec<ConcurrencyPattern> {
        let mut patterns = Vec::new();

        let mut async_functions = Vec::new();
        let mut parallel_functions = Vec::new();
        let mut thread_functions = Vec::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            if func.is_async {
                async_functions.push(func.name.clone());
            }

            if func.name.contains("spawn") || func.name.contains("thread") {
                thread_functions.push(func.name.clone());
            }

            if func.name.contains("parallel") || func.name.contains("par_") {
                parallel_functions.push(func.name.clone());
            }
        }

        if !async_functions.is_empty() {
            patterns.push(ConcurrencyPattern {
                pattern_type: ConcurrencyType::Async,
                functions: async_functions,
                description: "Async functions detected".to_string(),
            });
        }

        if !parallel_functions.is_empty() {
            patterns.push(ConcurrencyPattern {
                pattern_type: ConcurrencyType::Parallel,
                functions: parallel_functions,
                description: "Parallel processing functions detected".to_string(),
            });
        }

        if !thread_functions.is_empty() {
            patterns.push(ConcurrencyPattern {
                pattern_type: ConcurrencyType::Threaded,
                functions: thread_functions,
                description: "Thread management functions detected".to_string(),
            });
        }

        patterns
    }

    /// Generate a human-readable summary of semantic analysis
    pub fn summarize(analysis: &SemanticAnalysis) -> String {
        let mut output = String::new();

        output.push_str("## Semantic Analysis\n\n");

        // Function roles
        output.push_str("### Function Roles\n\n");
        let mut role_counts: HashMap<FunctionRole, usize> = HashMap::new();
        for role in analysis.function_roles.values() {
            *role_counts.entry(role.clone()).or_insert(0) += 1;
        }

        for (role, count) in role_counts {
            output.push_str(&format!("- {:?}: {}\n", role, count));
        }
        output.push('\n');

        // Concurrency patterns
        if !analysis.concurrency_patterns.is_empty() {
            output.push_str("### Concurrency Patterns\n\n");
            for pattern in &analysis.concurrency_patterns {
                output.push_str(&format!(
                    "- **{:?}**: {}\n",
                    pattern.pattern_type, pattern.description
                ));
                if !pattern.functions.is_empty() {
                    output.push_str(&format!(
                        "  - Functions: {}\n",
                        pattern.functions.join(", ")
                    ));
                }
            }
            output.push('\n');
        }

        // Error handling
        if !analysis.error_propagation.is_empty() {
            output.push_str("### Error Handling\n\n");
            let mut error_counts: HashMap<ErrorHandling, usize> = HashMap::new();
            for e in &analysis.error_propagation {
                *error_counts.entry(e.handling.clone()).or_insert(0) += 1;
            }

            for (handling, count) in error_counts {
                output.push_str(&format!("- {:?}: {} functions\n", handling, count));
            }
        }

        output
    }
}
