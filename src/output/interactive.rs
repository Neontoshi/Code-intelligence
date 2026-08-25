// src/output/interactive.rs

use crate::analysis::layers::LayerOrchestrator;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::traits::GraphMetrics;
use crate::parser::tree_sitter::ParsedFile;
use serde_json::json;

/// HTML template for interactive graph
const INTERACTIVE_HTML: &str = include_str!("../designs/interactive.html");

pub struct InteractiveGraph;

impl InteractiveGraph {
    pub fn generate(call_graph: &CallGraph, files: &[ParsedFile], project_name: &str) -> String {
        let nodes_data = Self::collect_nodes(call_graph, files);
        let edges_data = Self::collect_edges(call_graph);
        let stats = Self::collect_stats(call_graph, files);

        let nodes_json = serde_json::to_string(&nodes_data).unwrap_or_else(|_| "[]".to_string());
        let edges_json = serde_json::to_string(&edges_data).unwrap_or_else(|_| "[]".to_string());
        let stats_json = serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string());

        INTERACTIVE_HTML
            .replace("{{PROJECT_NAME}}", project_name)
            .replace("{{NODES_DATA}}", &nodes_json)
            .replace("{{EDGES_DATA}}", &edges_json)
            .replace("{{STATS_DATA}}", &stats_json)
    }

    fn collect_nodes(call_graph: &CallGraph, files: &[ParsedFile]) -> Vec<serde_json::Value> {
        use crate::analysis::{
            dead_code::is_never_dead,
            roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
            verdict_source::{VerdictConfig, VerdictEngine},
        };

        let layer_orchestrator = LayerOrchestrator::new();

        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, files, &root_config);
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);
        let verdict_engine = VerdictEngine::new(VerdictConfig::default());
        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);

        let mut dead_map = std::collections::HashMap::new();
        for verdict in verdicts {
            if verdict.is_dead() {
                dead_map.insert(verdict.full_path.clone(), true);
            }
        }

        let mut nodes = Vec::new();
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let is_dead = dead_map.contains_key(&func.full_path) && !is_never_dead(func);
            let layer = if func.layer.is_empty() {
                "core"
            } else {
                &func.layer
            };
            let layer_color = layer_orchestrator.get_layer_color(layer);

            nodes.push(json!({
                "id": idx.index(),
                "label": func.name,
                "full_path": func.full_path,
                "file": func.file,
                "line": func.line,
                "is_public": func.is_public,
                "is_async": func.is_async,
                "complexity": func.complexity,
                "importance": func.importance_score,
                "fan_in": func.fan_in,
                "fan_out": func.fan_out,
                "layer": layer,
                "layer_color": layer_color,
                "is_dead": is_dead,
                "is_test": func.is_test,
                "is_trait_method": func.is_trait_method,
                "size": Self::calculate_node_size(func),
                "color": Self::calculate_node_color(func, is_dead, &layer_color),
            }));
        }

        nodes
    }

    fn collect_edges(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        let mut edges = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for idx in call_graph.node_indices() {
            let source = idx.index();
            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = call_graph.name_index.get(&callee.full_path) {
                    let target = callee_idx.index();
                    let key = (source, target);
                    if !seen.contains(&key) {
                        seen.insert(key);
                        edges.push(json!({
                            "source": source,
                            "target": target,
                        }));
                    }
                }
            }
        }

        edges
    }

    fn collect_stats(call_graph: &CallGraph, files: &[ParsedFile]) -> serde_json::Value {
        use crate::analysis::{
            dead_code::is_never_dead,
            roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
            verdict_source::{VerdictConfig, VerdictEngine},
        };

        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, files, &root_config);
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);
        let verdict_engine = VerdictEngine::new(VerdictConfig::default());
        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);

        let mut dead_map = std::collections::HashMap::new();
        for verdict in verdicts {
            if verdict.is_dead() {
                dead_map.insert(verdict.full_path.clone(), true);
            }
        }

        let total_functions = call_graph.node_count();
        let dead_functions = call_graph
            .node_indices()
            .filter(|idx| {
                let func = &call_graph[*idx];
                dead_map.contains_key(&func.full_path) && !is_never_dead(func)
            })
            .count();
        let test_functions = call_graph
            .node_indices()
            .filter(|idx| call_graph[*idx].is_test)
            .count();
        let public_functions = call_graph
            .node_indices()
            .filter(|idx| call_graph[*idx].is_public)
            .count();
        let async_functions = call_graph
            .node_indices()
            .filter(|idx| call_graph[*idx].is_async)
            .count();

        let mut languages = std::collections::HashSet::new();
        for file in files {
            languages.insert(file.language.clone());
        }

        json!({
            "total_functions": total_functions,
            "dead_functions": dead_functions,
            "test_functions": test_functions,
            "public_functions": public_functions,
            "async_functions": async_functions,
            "total_edges": call_graph.edge_count(),
            "total_files": files.len(),
            "languages": languages.into_iter().collect::<Vec<String>>(),
        })
    }

    fn calculate_node_size(func: &FunctionNode) -> usize {
        let base = 10;
        let importance_factor = (func.importance_score * 20.0) as usize;
        let complexity_factor = (func.complexity / 5.0) as usize;
        base + importance_factor + complexity_factor
    }

    fn calculate_node_color(func: &FunctionNode, is_dead: bool, layer_color: &str) -> String {
        if func.is_test {
            "#8e44ad".to_string()
        } else if func.is_trait_method {
            "#2ecc71".to_string()
        } else if is_dead {
            "#e74c3c".to_string()
        } else if func.is_public {
            "#3498db".to_string()
        } else if func.importance_score > 0.7 {
            "#f39c12".to_string()
        } else if !layer_color.is_empty() && layer_color != "#475569" {
            layer_color.to_string()
        } else {
            "#95a5a6".to_string()
        }
    }
}
