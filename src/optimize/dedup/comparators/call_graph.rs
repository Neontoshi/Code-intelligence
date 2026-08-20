// src/optimize/dedup/comparators/call_graph.rs

use crate::graph::call_graph::{CallGraph, FunctionNode};
use std::collections::HashSet;

pub struct CallGraphComparator;

impl CallGraphComparator {
    pub fn compare(a: &FunctionNode, b: &FunctionNode, call_graph: &CallGraph) -> f64 {
        let idx_a = call_graph.name_index.get(&a.full_path).copied();
        let idx_b = call_graph.name_index.get(&b.full_path).copied();
        if let (Some(idx_a), Some(idx_b)) = (idx_a, idx_b) {
            let callees_a: HashSet<String> = call_graph
                .get_callees(idx_a)
                .iter()
                .map(|f| f.name.clone())
                .collect();
            let callees_b: HashSet<String> = call_graph
                .get_callees(idx_b)
                .iter()
                .map(|f| f.name.clone())
                .collect();

            // Leaf functions that don't call anything provide NO call graph evidence
            if callees_a.is_empty() && callees_b.is_empty() {
                return 0.0;
            }

            let intersection = callees_a.intersection(&callees_b).count();
            let union = callees_a.union(&callees_b).count();

            if union == 0 {
                return 0.0;
            }
            return intersection as f64 / union as f64;
        }

        0.0
    }

    pub fn call_pattern_similarity(a: &FunctionNode, b: &FunctionNode) -> f64 {
        let call_count_a = a.params.len();
        let call_count_b = b.params.len();

        if call_count_a == 0 && call_count_b == 0 {
            return 0.0;
        }

        let count_diff = (call_count_a as i32 - call_count_b as i32).abs();
        (1.0 - (count_diff as f64 / 10.0)).max(0.0)
    }
}
