// src/output/json.rs

use crate::analysis::dead_code::DeadCodeDetector;
use crate::analysis::layers::LayerOrchestrator;
use crate::graph::call_graph::CallGraph;
use crate::graph::traits::GraphMetrics;
use crate::parser::tree_sitter::ParsedFile;
use serde_json::{json, Value};

pub struct JsonOutput;

impl JsonOutput {
    /// Generate a complete JSON representation for AI training
    pub fn generate(
        call_graph: &CallGraph,
        files: &[ParsedFile],
        root: &std::path::Path,
    ) -> String {
        let project_name = root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut report = json!({
            "project": project_name,
            "stats": {
                "total_functions": call_graph.node_count(),
                "total_relationships": call_graph.edge_count(),
                "total_files": files.len(),
                "languages": Vec::<String>::new(),
            },
            "architecture": {
                "entry_points": Vec::<Value>::new(),
                "layers": Vec::<Value>::new(),
            },
            "functions": Vec::<Value>::new(),
            "call_graph": {
                "nodes": Vec::<Value>::new(),
                "edges": Vec::<Value>::new(),
            },
            "dead_code": {
                "unused_functions": Vec::<String>::new(),
                "dead_modules": Vec::<String>::new(),
                "ratio": 0.0,
                "unused_count": 0,
            },
            "training_metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "compression_ratio": 0.0,
            }
        });

        // Detect languages
        let mut languages: Vec<String> = files
            .iter()
            .map(|f| f.language.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        languages.sort();
        report["stats"]["languages"] = json!(languages);

        // Entry points
        let entry_points: Vec<Value> = call_graph
            .node_indices()
            .filter(|idx| {
                let func = &call_graph[*idx];
                func.is_public && call_graph.get_callers(*idx).is_empty()
            })
            .map(|idx| {
                let func = &call_graph[idx];
                json!({
                    "name": func.name,
                    "file": func.file,
                    "line": func.line,
                    "is_async": func.is_async,
                    "params": func.params,
                    "returns": func.returns,
                    "importance": func.importance_score,
                })
            })
            .collect();
        report["architecture"]["entry_points"] = json!(entry_points);

        // Detect layers using the new orchestrator
        let layers = Self::detect_layers(files);
        report["architecture"]["layers"] = json!(layers);

        // All functions with metadata
        let functions: Vec<Value> = call_graph
            .node_indices()
            .map(|idx| {
                let func = &call_graph[idx];
                let callees = call_graph.get_callees(idx);
                let callers = call_graph.get_callers(idx);

                json!({
                    "name": func.name,
                    "full_path": func.full_path,
                    "file": func.file,
                    "line": func.line,
                    "is_public": func.is_public,
                    "is_async": func.is_async,
                    "params": func.params,
                    "returns": func.returns,
                    "complexity": func.complexity,
                    "importance": func.importance_score,
                    "doc_comment": func.doc_comment,
                    "calls": callees.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                    "callers": callers.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                    "callee_count": callees.len(),
                    "caller_count": callers.len(),
                })
            })
            .collect();
        report["functions"] = json!(functions);

        // Call graph edges (for graph analysis)
        let edges: Vec<Value> = call_graph
            .node_indices()
            .flat_map(|idx| {
                let func = &call_graph[idx];
                call_graph
                    .get_callees(idx)
                    .iter()
                    .map(|callee| {
                        json!({
                            "source": func.name,
                            "target": callee.name,
                            "source_file": func.file,
                            "target_file": callee.file,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        report["call_graph"]["edges"] = json!(edges);

        // Nodes for graph
        let nodes: Vec<Value> = call_graph
            .node_indices()
            .map(|idx| {
                let func = &call_graph[idx];
                json!({
                    "id": func.name,
                    "file": func.file,
                    "importance": func.importance_score,
                    "is_public": func.is_public,
                    "caller_count": call_graph.get_callers(idx).len(),
                    "callee_count": call_graph.get_callees(idx).len(),
                })
            })
            .collect();
        report["call_graph"]["nodes"] = json!(nodes);

        // Dead code analysis - using verdict-based approach
        let stats = DeadCodeDetector::get_dead_stats(call_graph, files);
        let dead_modules = DeadCodeDetector::find_dead_modules(files);
        let dead_ratio = if stats.total > 0 {
            stats.dead as f64 / stats.total as f64
        } else {
            0.0
        };

        report["dead_code"] = json!({
            "unused_functions": Vec::<String>::new(),
            "dead_modules": dead_modules,
            "ratio": dead_ratio,
            "unused_count": stats.dead,
        });

        // Training metadata
        report["training_metadata"]["compression_ratio"] = json!(dead_ratio);

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    /// Generate a minimal JSON for fine-tuning (input-output pairs)
    pub fn generate_training_pairs(_call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut pairs = Vec::new();

        for file in files {
            if file.functions.is_empty() {
                continue;
            }

            // Create input: function signatures
            let input = file
                .functions
                .iter()
                .map(|f| {
                    format!(
                        "ƒ {} ({}) → {}",
                        f.name,
                        f.params
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                        f.return_type.as_deref().unwrap_or("()")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            // Create output: what this file does
            let all_calls: Vec<String> = file
                .functions
                .iter()
                .flat_map(|f| f.calls.iter().map(|c| c.to_string()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let types_defined: Vec<String> = file
                .types
                .iter()
                .map(|t| format!("{} ({:?})", t.name, t.kind))
                .collect();

            let output = json!({
                "file": file.path,
                "language": file.language,
                "functions_defined": file.functions.len(),
                "types_defined": types_defined,
                "external_calls": all_calls,
                "description": format!(
                    "Defines {} functions and {} types. Calls: {}",
                    file.functions.len(),
                    file.types.len(),
                    all_calls.iter().take(10).cloned().collect::<Vec<_>>().join(", ")
                ),
            });

            pairs.push(json!({
                "input": input,
                "output": output,
            }));
        }

        serde_json::to_string_pretty(&pairs).unwrap_or_default()
    }

    /// Detect architectural layers from file paths using the LayerOrchestrator
    pub fn detect_layers(files: &[ParsedFile]) -> Vec<Value> {
        use std::collections::HashMap;

        let orchestrator = LayerOrchestrator::new();
        let mut layers: HashMap<String, Vec<String>> = HashMap::new();

        for file in files {
            let layer = orchestrator.detect_layer(file);
            let filename = file.path.split('/').last().unwrap_or(&file.path);
            layers.entry(layer).or_default().push(filename.to_string());
        }

        layers
            .into_iter()
            .map(|(layer, files)| {
                json!({
                    "name": layer,
                    "description": orchestrator.get_layer_description(&layer),
                    "color": orchestrator.get_layer_color(&layer),
                    "files": files,
                    "file_count": files.len(),
                })
            })
            .collect()
    }
}
