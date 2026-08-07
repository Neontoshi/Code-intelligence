// src/analysis/importance.rs

use crate::graph::call_graph::CallGraph;

pub struct ImportanceScorer {
    complexity_weight: f64,
    centrality_weight: f64,
    connectivity_weight: f64,
    fan_in_weight: f64,
    layer_weight: f64,
}

impl ImportanceScorer {
    pub fn new() -> Self {
        Self {
            complexity_weight: 0.15,
            centrality_weight: 0.25,
            connectivity_weight: 0.20,
            fan_in_weight: 0.30,
            layer_weight: 0.10,
        }
    }

    pub fn score_all(&self, graph: &mut CallGraph) {
        // Get max fan-in for normalization
        let max_fan_in = graph
            .node_indices()
            .map(|idx| graph[idx].fan_in)
            .max()
            .unwrap_or(1) as f64;

        // Get max degree for centrality
        let max_degree = graph
            .node_indices()
            .map(|idx| graph.get_callees(idx).len() + graph.get_callers(idx).len())
            .max()
            .unwrap_or(1) as f64;

        let mut scores = Vec::new();
        for idx in graph.node_indices() {
            let func = &graph[idx];
            let mut score = 0.0;

            // Complexity (higher = more important)
            score += self.complexity_weight * (func.complexity / 50.0).min(1.0);

            // Centrality (higher = more connected)
            let degree = graph.get_callees(idx).len() + graph.get_callers(idx).len();
            let centrality = degree as f64 / max_degree.max(1.0);
            score += self.centrality_weight * centrality;

            // Connectivity (total connections)
            let connections = graph.get_callees(idx).len() + graph.get_callers(idx).len();
            score += self.connectivity_weight * (connections as f64 / 50.0).min(1.0);

            // Fan-in (more callers = more important)
            let normalized_fan_in = if max_fan_in > 0.0 {
                func.fan_in as f64 / max_fan_in
            } else {
                0.0
            };
            score += self.fan_in_weight * normalized_fan_in;

            // Layer importance (handlers and services get slight boost)
            let layer_bonus = match func.layer.as_str() {
                "handler" => 0.5,
                "service" => 0.4,
                "api" => 0.3,
                "repository" => 0.2,
                "blockchain" => 0.2,
                _ => 0.0,
            };
            score += self.layer_weight * layer_bonus;

            scores.push((idx, score.min(1.0)));
        }

        for (idx, score) in scores {
            graph[idx].importance_score = score;
        }
    }
}
