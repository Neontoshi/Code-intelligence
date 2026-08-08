// src/analysis/dead_code/reachability.rs

#![allow(deprecated)]
use crate::graph::call_graph::CallGraph;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct ReachabilityReport {
    pub reachable: Vec<String>,
    pub unreachable: Vec<String>,
    pub entry_points: Vec<String>,
    pub reachability_graph: Vec<(String, String)>,
}

#[deprecated(
    since = "0.2.0",
    note = "Use crate::analysis::roots::ReachabilityAnalyzer instead"
)]
#[allow(dead_code)]
pub struct ReachabilityAnalyzer;

#[allow(dead_code)]
impl ReachabilityAnalyzer {
    /// Find all functions reachable from entry points
    #[deprecated]
    #[allow(dead_code)]
    pub fn find_reachable_functions(call_graph: &CallGraph) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from entry points: main + public functions
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if func.name == "main" || func.name == "async_main" || func.is_public {
                if !reachable.contains(&func.full_path) {
                    reachable.insert(func.full_path.clone());
                    queue.push_back(idx);
                }
            }
        }

        // BFS traversal
        while let Some(current_idx) = queue.pop_front() {
            for callee in call_graph.get_callees(current_idx) {
                if !reachable.contains(&callee.full_path) {
                    reachable.insert(callee.full_path.clone());
                    // Find the callee's node index
                    for idx in call_graph.node_indices() {
                        if call_graph[idx].full_path == callee.full_path {
                            queue.push_back(idx);
                            break;
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Generate reachability report with entry points
    #[deprecated]
    #[allow(dead_code)]
    pub fn analyze_reachability(call_graph: &CallGraph) -> ReachabilityReport {
        let reachable = Self::find_reachable_functions(call_graph);
        let mut entry_points = Vec::new();
        let mut unreachable = Vec::new();

        // Identify entry points
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if func.name == "main" || func.name == "async_main" {
                entry_points.push(func.full_path.clone());
            }
        }

        // Find unreachable functions
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if !reachable.contains(&func.full_path) {
                unreachable.push(func.full_path.clone());
            }
        }

        // Build reachability graph (simplified)
        let mut reachability_graph = Vec::new();
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if reachable.contains(&func.full_path) {
                for callee in call_graph.get_callees(idx) {
                    if reachable.contains(&callee.full_path) {
                        reachability_graph.push((func.full_path.clone(), callee.full_path.clone()));
                    }
                }
            }
        }

        ReachabilityReport {
            reachable: reachable.into_iter().collect(),
            unreachable,
            entry_points,
            reachability_graph,
        }
    }
}
