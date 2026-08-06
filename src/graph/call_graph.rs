use crate::graph::traits::GraphMetrics;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet; // ✅ Should already be there // ✅ Use this path

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionNode {
    pub name: String,
    pub full_path: String,
    pub file: String,
    pub line: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub params: Vec<String>,
    pub returns: Vec<String>,
    pub complexity: f64,
    pub importance_score: f64,
    pub doc_comment: Option<String>,
    pub writes_to: Vec<String>,
    pub reads_from: Vec<String>,
    pub errors: Vec<String>,
    // Call graph metrics
    pub fan_in: usize,  // Number of callers
    pub fan_out: usize, // Number of callees
    pub is_cycle: bool, // Part of a cycle in the call graph
    pub depth: usize,   // Call depth from entry points
    pub layer: String,  // Architecture layer (handler, service, repository, etc.)
    pub trait_impl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub call_type: String,
    pub line: usize,
}

#[derive(Clone)]
pub struct CallGraph {
    pub graph: DiGraph<FunctionNode, CallEdge>,
    pub name_index: HashMap<String, NodeIndex>,
    // Inverted indexes for fast lookups
    pub name_to_functions: HashMap<String, Vec<NodeIndex>>,
    pub file_to_functions: HashMap<String, Vec<NodeIndex>>,
    pub public_functions: Vec<NodeIndex>,
    pub async_functions: Vec<NodeIndex>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            name_index: HashMap::new(),
            name_to_functions: HashMap::new(),
            file_to_functions: HashMap::new(),
            public_functions: Vec::new(),
            async_functions: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: FunctionNode) -> NodeIndex {
        let name = func.full_path.clone();
        let idx = self.graph.add_node(func);
        self.name_index.insert(name.clone(), idx);

        // Update inverted indexes
        let func_ref = &self.graph[idx];

        // Name index (simple name, not full path)
        self.name_to_functions
            .entry(func_ref.name.clone())
            .or_default()
            .push(idx);

        // File index
        self.file_to_functions
            .entry(func_ref.file.clone())
            .or_default()
            .push(idx);

        // Public functions
        if func_ref.is_public {
            self.public_functions.push(idx);
        }

        // Async functions
        if func_ref.is_async {
            self.async_functions.push(idx);
        }

        idx
    }

    pub fn add_call(&mut self, caller: NodeIndex, callee: NodeIndex, edge: CallEdge) {
        self.graph.add_edge(caller, callee, edge);
    }
    pub fn get_functions_by_name(&self, name: &str) -> Vec<&FunctionNode> {
        self.name_to_functions
            .get(name)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.graph.node_weight(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_functions_by_file(&self, file: &str) -> Vec<&FunctionNode> {
        self.file_to_functions
            .get(file)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.graph.node_weight(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_public_functions(&self) -> Vec<&FunctionNode> {
        self.public_functions
            .iter()
            .filter_map(|&idx| self.graph.node_weight(idx))
            .collect()
    }

    pub fn get_async_functions(&self) -> Vec<&FunctionNode> {
        self.async_functions
            .iter()
            .filter_map(|&idx| self.graph.node_weight(idx))
            .collect()
    }

    pub fn get_by_name_simple(&self, name: &str) -> Option<NodeIndex> {
        // Exact match on simple name (not full path)
        self.name_to_functions
            .get(name)
            .and_then(|indices| indices.first().copied())
    }

    pub fn node_indices(&self) -> impl Iterator<Item = NodeIndex> {
        self.graph.node_indices()
    }

    pub fn get_callees(&self, func: NodeIndex) -> Vec<&FunctionNode> {
        self.graph
            .edges_directed(func, petgraph::Direction::Outgoing)
            .map(|e| &self.graph[e.target()])
            .collect()
    }

    pub fn get_callers(&self, func: NodeIndex) -> Vec<&FunctionNode> {
        self.graph
            .edges_directed(func, petgraph::Direction::Incoming)
            .map(|e| &self.graph[e.source()])
            .collect()
    }

    pub fn summarize_function(&self, func: NodeIndex) -> String {
        let node = &self.graph[func];
        let callees = self.get_callees(func);

        let mut summary = format!("ƒ {}\n", node.name);

        if let Some(doc) = &node.doc_comment {
            summary.push_str(&format!("  📝 {}\n", doc));
        }

        if !node.params.is_empty() {
            summary.push_str("  📥 Inputs:\n");
            for param in &node.params {
                summary.push_str(&format!("    - {}\n", param));
            }
        }

        if !callees.is_empty() {
            summary.push_str("  📞 Calls:\n");
            for callee in &callees {
                summary.push_str(&format!("    - {}\n", callee.name));
            }
        }

        summary
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph CallGraph {\n  rankdir=LR;\n");

        for node in self.graph.node_indices() {
            let func = &self.graph[node];
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\"];\n",
                func.full_path, func.name
            ));
        }

        for edge in self.graph.edge_references() {
            let source = &self.graph[edge.source()].full_path;
            let target = &self.graph[edge.target()].full_path;
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", source, target));
        }

        dot.push_str("}\n");
        dot
    }
    /// Detect cycles in the call graph using DFS
    /// Detect cycles in the call graph using iterative DFS
    pub fn detect_cycles(&self) -> Vec<Vec<NodeIndex>> {
        use std::collections::HashSet;

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();

        // Use iterative DFS with explicit stack to avoid recursion overflow
        for start_node in self.graph.node_indices() {
            if visited.contains(&start_node) {
                continue;
            }

            // Stack: (node, parent, path_index, state)
            // state: 0 = enter, 1 = exit
            let mut stack: Vec<(NodeIndex, Option<NodeIndex>, Vec<NodeIndex>, u8)> = Vec::new();
            stack.push((start_node, None, vec![start_node], 0));

            while let Some((node, _parent, path, state)) = stack.pop() {
                if state == 1 {
                    // Exit state - nothing to do
                    continue;
                }

                visited.insert(node);

                // Find unvisited neighbors
                let neighbors: Vec<NodeIndex> = self
                    .graph
                    .edges_directed(node, petgraph::Direction::Outgoing)
                    .map(|e| e.target())
                    .collect();

                // If no neighbors or all visited, backtrack
                let mut has_unvisited = false;
                for &neighbor in &neighbors {
                    if !visited.contains(&neighbor) {
                        has_unvisited = true;
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        stack.push((node, Some(node), path.clone(), 1)); // exit
                        stack.push((neighbor, Some(node), new_path, 0)); // enter
                        break;
                    } else if path.contains(&neighbor) {
                        // Found a cycle: neighbor is in current path
                        if let Some(pos) = path.iter().position(|&n| n == neighbor) {
                            let cycle: Vec<NodeIndex> = path[pos..].to_vec();
                            if !cycles.contains(&cycle) && cycle.len() > 1 {
                                cycles.push(cycle);
                            }
                        }
                    }
                }

                // If no unvisited neighbors, mark as fully processed
                if !has_unvisited {
                    // Already visited, continue
                }
            }
        }

        // Deduplicate cycles
        let mut unique_cycles = Vec::new();
        for cycle in cycles {
            if !unique_cycles.contains(&cycle) {
                unique_cycles.push(cycle);
            }
        }

        unique_cycles
    }

    /// Mark functions that are part of cycles
    pub fn mark_cycle_members(&mut self) {
        let cycles = self.detect_cycles();
        let mut cycle_members = HashSet::new();

        for cycle in cycles {
            for node in cycle {
                cycle_members.insert(node);
            }
        }

        for node in cycle_members {
            self.graph[node].is_cycle = true;
        }
    }

    /// Calculate fan-in and fan-out for all functions
    pub fn calculate_fan_metrics(&mut self) {
        let mut updates = Vec::new();
        for idx in self.graph.node_indices() {
            let callers = self.get_callers(idx);
            let callees = self.get_callees(idx);
            updates.push((idx, callers.len(), callees.len()));
        }
        for (idx, fan_in, fan_out) in updates {
            self.graph[idx].fan_in = fan_in;
            self.graph[idx].fan_out = fan_out;
        }
    }

    /// Detect architecture layers from file paths
    pub fn detect_layers(&mut self) {
        for idx in self.graph.node_indices() {
            let func = &self.graph[idx];
            let path = &func.file;
            let parts: Vec<&str> = path.split('/').collect();

            let layer = if parts.len() >= 2 {
                match parts[parts.len() - 2] {
                    "handlers" | "controllers" | "routes" => "handler",
                    "services" | "domain" | "business" => "service",
                    "db" | "repository" | "repositories" | "models" | "dao" => "repository",
                    "middleware" => "middleware",
                    "config" | "configuration" => "config",
                    "workers" | "jobs" | "tasks" => "worker",
                    "solana" | "blockchain" | "chain" => "blockchain",
                    "telemetry" | "metrics" | "tracing" | "observability" => "observability",
                    "auth" | "authentication" | "authorization" => "auth",
                    "utils" | "util" | "helpers" | "common" => "utility",
                    "api" | "rest" | "graphql" => "api",
                    "cli" | "cmd" => "cli",
                    "tests" | "test" | "integration" => "test",
                    _ => "core",
                }
            } else {
                "root"
            };

            self.graph[idx].layer = layer.to_string();
        }
    }

    /// Calculate call depth from entry points using iterative BFS
    pub fn calculate_call_depth(&mut self) {
        use std::collections::{HashMap, VecDeque};

        let mut depths = HashMap::new();

        // Entry points: public functions with no callers
        let entry_points: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].is_public && self.get_callers(idx).is_empty())
            .collect();

        // BFS from entry points (iterative, no recursion)
        let mut queue = VecDeque::new();
        for entry in entry_points {
            queue.push_back((entry, 0));
            depths.insert(entry, 0);
        }

        while let Some((node, depth)) = queue.pop_front() {
            for edge in self
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
            {
                let target = edge.target();
                let new_depth = depth + 1;

                // Visit each node exactly once — first (shortest) path wins.
                // A "revisit if deeper" rule never terminates on a cyclic graph,
                // since walking a cycle repeatedly always increases new_depth.
                if !depths.contains_key(&target) {
                    depths.insert(target, new_depth);
                    queue.push_back((target, new_depth));
                }
            }
        }

        // Apply depths, default to 0 for unvisited
        for idx in self.graph.node_indices() {
            self.graph[idx].depth = *depths.get(&idx).unwrap_or(&0);
        }
    }

    pub fn top_important_nodes(&self, max_nodes: usize, min_importance: f64) -> Vec<NodeIndex> {
        let mut candidates: Vec<NodeIndex> = self
            .node_indices()
            .filter(|idx| self.graph[*idx].importance_score > min_importance)
            .collect();

        candidates.sort_by(|a, b| {
            self.graph[*b]
                .importance_score
                .partial_cmp(&self.graph[*a].importance_score)
                .unwrap()
        });
        candidates.truncate(max_nodes);
        candidates
    }
}

impl std::ops::Index<NodeIndex> for CallGraph {
    type Output = FunctionNode;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.graph[index]
    }
}

impl std::ops::IndexMut<NodeIndex> for CallGraph {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.graph[index]
    }
}

impl GraphMetrics for CallGraph {
    fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
