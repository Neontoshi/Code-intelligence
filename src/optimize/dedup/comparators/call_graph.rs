use crate::graph::call_graph::{CallGraph, FunctionNode};
use std::collections::HashSet;

pub struct CallGraphComparator;

impl CallGraphComparator {
    pub fn compare(a: &FunctionNode, b: &FunctionNode, call_graph: &CallGraph) -> f64 {
        // name_index is already a HashMap<full_path, NodeIndex> — O(1) instead
        // of scanning every node in the graph for every candidate pair.
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

            // Jaccard similarity of callees
            let intersection = callees_a.intersection(&callees_b).count();
            let union = callees_a.union(&callees_b).count();

            if union == 0 {
                return 1.0;
            }
            return intersection as f64 / union as f64;
        }

        0.0
    }

    pub fn call_pattern_similarity(a: &FunctionNode, b: &FunctionNode) -> f64 {
        // Compare call counts and patterns
        // Use params as proxy for call count
        let call_count_a = a.params.len();
        let call_count_b = b.params.len();

        if call_count_a == 0 && call_count_b == 0 {
            return 1.0;
        }

        let count_diff = (call_count_a as i32 - call_count_b as i32).abs();
        let result = 1.0 - (count_diff as f64 / 10.0);
        if result < 0.0 {
            0.0
        } else {
            result
        }
    }
}
