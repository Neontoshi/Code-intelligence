// src/analysis/importance.rs

use crate::graph::call_graph::CallGraph;
use crate::graph::GraphMetrics;
use petgraph::Direction;

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

    pub fn score_all(&self, call_graph: &mut CallGraph) {
        if call_graph.node_count() == 0 {
            return;
        }

        // 1. Determine maximum values across the graph for proper normalization
        let max_fan_in = call_graph
            .node_indices()
            .map(|idx| call_graph[idx].fan_in)
            .max()
            .unwrap_or(1) as f64;

        let max_degree = call_graph
            .node_indices()
            .map(|idx| {
                let in_deg = call_graph
                    .graph
                    .neighbors_directed(idx, Direction::Incoming)
                    .count();
                let out_deg = call_graph
                    .graph
                    .neighbors_directed(idx, Direction::Outgoing)
                    .count();
                in_deg + out_deg
            })
            .max()
            .unwrap_or(1) as f64;

        // 2. Compute importance scores immutably
        let computed_scores: Vec<(petgraph::graph::NodeIndex, f64)> = call_graph
            .node_indices()
            .map(|idx| {
                let func = &call_graph[idx];
                let mut score = 0.0;

                // Complexity
                score += self.complexity_weight * (func.complexity / 50.0).clamp(0.0, 1.0);

                // Centrality
                let in_deg = call_graph
                    .graph
                    .neighbors_directed(idx, Direction::Incoming)
                    .count();
                let out_deg = call_graph
                    .graph
                    .neighbors_directed(idx, Direction::Outgoing)
                    .count();
                let total_degree = (in_deg + out_deg) as f64;
                score += self.centrality_weight * (total_degree / max_degree.max(1.0));

                // Connectivity
                score += self.connectivity_weight * (total_degree / 50.0).clamp(0.0, 1.0);

                // Fan-in
                let normalized_fan_in = if max_fan_in > 0.0 {
                    func.fan_in as f64 / max_fan_in
                } else {
                    0.0
                };
                score += self.fan_in_weight * normalized_fan_in;

                // Architectural layer weight
                let layer_bonus = match func.layer.as_str() {
                    "handler" => 0.5,
                    "service" => 0.4,
                    "api" => 0.3,
                    "repository" => 0.2,
                    "blockchain" => 0.2,
                    _ => 0.0,
                };
                score += self.layer_weight * layer_bonus;

                (idx, score.clamp(0.0, 1.0))
            })
            .collect();

        // 3. Mutate node weights
        for (idx, score) in computed_scores {
            call_graph[idx].importance_score = score;
        }
    }
}

impl Default for ImportanceScorer {
    fn default() -> Self {
        Self::new()
    }
}
