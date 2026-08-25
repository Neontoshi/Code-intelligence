// src/output/overview.rs

use crate::analysis::layers::LayerOrchestrator;
use crate::graph::call_graph::CallGraph;
use std::collections::HashMap;

/// HTML template for overview graph
const OVERVIEW_HTML: &str = include_str!("../designs/overview.html");

pub struct LayerSummary {
    pub name: String,
    pub count: usize,
    pub description: String,
    pub color: String,
    pub top_functions: Vec<String>,
}

pub struct LayerEdge {
    pub from: String,
    pub to: String,
    pub weight: usize,
}

pub struct OverviewGraph;

impl OverviewGraph {
    pub fn generate(call_graph: &CallGraph, project_name: &str) -> String {
        let (layers, edges) = Self::collect_layer_data(call_graph);
        let function_nodes = Self::collect_function_nodes(call_graph);
        let function_edges = Self::collect_function_edges(call_graph);

        let layers_json = serde_json::to_string(
            &layers
                .iter()
                .map(|l| {
                    serde_json::json!({
                        "name": l.name,
                        "count": l.count,
                        "description": l.description,
                        "color": l.color,
                        "top_functions": l.top_functions,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .expect("Failed to serialize layers for overview");

        let edges_json = serde_json::to_string(
            &edges
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "from": e.from,
                        "to": e.to,
                        "weight": e.weight,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .expect("Failed to serialize layer edges for overview");

        let nodes_json =
            serde_json::to_string(&function_nodes).unwrap_or_else(|_| "[]".to_string());

        let fn_edges_json =
            serde_json::to_string(&function_edges).unwrap_or_else(|_| "[]".to_string());

        OVERVIEW_HTML
            .replace("{{PROJECT_NAME}}", project_name)
            .replace("{{LAYERS_DATA}}", &layers_json)
            .replace("{{LAYER_EDGES_DATA}}", &edges_json)
            .replace("{{NODES_DATA}}", &nodes_json)
            .replace("{{FN_EDGES_DATA}}", &fn_edges_json)
    }

    fn collect_layer_data(call_graph: &CallGraph) -> (Vec<LayerSummary>, Vec<LayerEdge>) {
        let orchestrator = LayerOrchestrator::new();
        let mut layer_counts: HashMap<String, usize> = HashMap::new();
        let mut layer_top: HashMap<String, Vec<(String, f64)>> = HashMap::new();
        let mut layer_edges: HashMap<(String, String), usize> = HashMap::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let layer = if func.layer.is_empty() {
                "core".to_string()
            } else {
                func.layer.clone()
            };

            *layer_counts.entry(layer.clone()).or_insert(0) += 1;
            layer_top
                .entry(layer.clone())
                .or_default()
                .push((func.name.clone(), func.importance_score));

            for callee in call_graph.get_callees(idx) {
                if let Some(&callee_idx) = call_graph.name_index.get(&callee.full_path) {
                    let callee_layer = call_graph[callee_idx].layer.clone();
                    let callee_layer = if callee_layer.is_empty() {
                        "core".to_string()
                    } else {
                        callee_layer
                    };
                    if callee_layer != layer {
                        *layer_edges
                            .entry((layer.clone(), callee_layer))
                            .or_insert(0) += 1;
                    }
                }
            }
        }

        let mut layers: Vec<LayerSummary> = layer_counts
            .into_iter()
            .map(|(name, count)| {
                let mut top = layer_top.remove(&name).unwrap_or_default();
                top.sort_by(|a, b| b.1.total_cmp(&a.1));
                let top_functions = top.into_iter().take(3).map(|(n, _)| n).collect();

                LayerSummary {
                    color: orchestrator.get_layer_color(&name),
                    description: orchestrator.get_layer_description(&name),
                    name,
                    count,
                    top_functions,
                }
            })
            .collect();
        layers.sort_by(|a, b| b.count.cmp(&a.count));

        let edges = layer_edges
            .into_iter()
            .map(|((from, to), weight)| LayerEdge { from, to, weight })
            .collect();

        (layers, edges)
    }

    fn collect_function_nodes(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        let orchestrator = LayerOrchestrator::new();

        call_graph
            .node_indices()
            .map(|idx| {
                let func = &call_graph[idx];
                let layer = if func.layer.is_empty() {
                    "core".to_string()
                } else {
                    func.layer.clone()
                };
                serde_json::json!({
                    "id": idx.index(),
                    "name": func.name,
                    "file": func.file,
                    "line": func.line,
                    "layer": layer,
                    "layer_color": orchestrator.get_layer_color(&layer),
                    "importance": func.importance_score,
                })
            })
            .collect()
    }

    fn collect_function_edges(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        let mut edges = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for idx in call_graph.node_indices() {
            let source = idx.index();
            for callee in call_graph.get_callees(idx) {
                if let Some(&callee_idx) = call_graph.name_index.get(&callee.full_path) {
                    let target = callee_idx.index();
                    if seen.insert((source, target)) {
                        edges.push(serde_json::json!({ "source": source, "target": target }));
                    }
                }
            }
        }
        edges
    }
}
