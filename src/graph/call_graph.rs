use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

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
    pub fan_in: usize,
    pub fan_out: usize,
    pub is_cycle: bool,
    pub depth: usize,
    pub layer: String,
    pub trait_impl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub call_type: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CycleDetectionResult {
    pub cycles: Vec<Vec<NodeIndex>>,
    pub skipped: bool,
    pub node_count: usize,
    pub max_nodes_limit: usize,
}

#[derive(Clone)]
pub struct CallGraph {
    pub graph: DiGraph<FunctionNode, CallEdge>,
    pub name_index: HashMap<String, NodeIndex>,
    pub name_to_functions: HashMap<String, Vec<NodeIndex>>,
    pub file_to_functions: HashMap<String, Vec<NodeIndex>>,
    pub public_functions: Vec<NodeIndex>,
    pub async_functions: Vec<NodeIndex>,
    pub cycle_detection_skipped: bool,
    pub cycle_detection_node_count: usize,
    pub duplicate_functions: Vec<String>, // ⭐ new
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
            cycle_detection_skipped: false,
            cycle_detection_node_count: 0,
            duplicate_functions: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: FunctionNode) -> NodeIndex {
        let name = func.full_path.clone();

        if let Some(&existing) = self.name_index.get(&name) {
            self.duplicate_functions.push(name);
            return existing;
        }
        let idx = self.graph.add_node(func);
        self.name_index.insert(name, idx);

        let func_ref = &self.graph[idx];

        self.name_to_functions
            .entry(func_ref.name.clone())
            .or_default()
            .push(idx);

        self.file_to_functions
            .entry(func_ref.file.clone())
            .or_default()
            .push(idx);

        if func_ref.is_public {
            self.public_functions.push(idx);
        }

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

    /// Returns None if multiple functions share the same name
    pub fn get_by_name_simple(&self, name: &str) -> Option<NodeIndex> {
        if let Some(indices) = self.name_to_functions.get(name) {
            if indices.len() == 1 {
                Some(indices[0])
            } else {
                eprintln!(
                    "⚠️ Ambiguous function name '{}' ({} matches). Use full_path instead.",
                    name,
                    indices.len()
                );
                None
            }
        } else {
            None
        }
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

    pub fn detect_cycles(&self) -> CycleDetectionResult {
        const MAX_NODES: usize = 5000;
        let total_nodes = self.graph.node_count();

        if total_nodes > MAX_NODES {
            return CycleDetectionResult {
                cycles: Vec::new(),
                skipped: true,
                node_count: total_nodes,
                max_nodes_limit: MAX_NODES,
            };
        }

        let mut cycles = Vec::new();
        let mut index = 0;
        let mut stack = Vec::new();
        let mut indices = HashMap::new();
        let mut lowlink = HashMap::new();
        let mut on_stack = HashSet::new();

        for start_node in self.graph.node_indices() {
            if indices.contains_key(&start_node) {
                continue;
            }

            let mut dfs_stack = vec![(start_node, 0)];
            indices.insert(start_node, index);
            lowlink.insert(start_node, index);
            index += 1;
            stack.push(start_node);
            on_stack.insert(start_node);

            while let Some((node, next_idx)) = dfs_stack.last_mut() {
                let neighbors: Vec<NodeIndex> = self
                    .graph
                    .edges_directed(*node, petgraph::Direction::Outgoing)
                    .map(|e| e.target())
                    .collect();

                if *next_idx < neighbors.len() {
                    let neighbor = neighbors[*next_idx];
                    *next_idx += 1;

                    if !indices.contains_key(&neighbor) {
                        indices.insert(neighbor, index);
                        lowlink.insert(neighbor, index);
                        index += 1;
                        stack.push(neighbor);
                        on_stack.insert(neighbor);
                        dfs_stack.push((neighbor, 0));
                    } else if on_stack.contains(&neighbor) {
                        let neighbor_low = *lowlink.get(&neighbor).unwrap_or(&0);
                        let node_low = lowlink.get_mut(node).unwrap();
                        if neighbor_low < *node_low {
                            *node_low = neighbor_low;
                        }
                    }
                } else {
                    let node = *node;
                    let node_low = *lowlink.get(&node).unwrap_or(&0);

                    let len = dfs_stack.len();
                    if len >= 2 {
                        let parent = dfs_stack[len - 2].0;
                        let parent_low = lowlink.get_mut(&parent).unwrap();
                        if node_low < *parent_low {
                            *parent_low = node_low;
                        }
                    }

                    if node_low == *indices.get(&node).unwrap_or(&0) {
                        let mut scc = Vec::new();
                        while let Some(w) = stack.pop() {
                            on_stack.remove(&w);
                            scc.push(w);
                            if w == node {
                                break;
                            }
                        }
                        if scc.len() > 1 {
                            cycles.push(scc);
                        }
                    }

                    dfs_stack.pop();
                }
            }
        }

        CycleDetectionResult {
            cycles,
            skipped: false,
            node_count: total_nodes,
            max_nodes_limit: MAX_NODES,
        }
    }

    pub fn mark_cycle_members(&mut self) {
        let result = self.detect_cycles();
        self.cycle_detection_skipped = result.skipped;
        self.cycle_detection_node_count = result.node_count;

        if result.skipped {
            eprintln!(
                "⚠️ Cycle detection skipped: {} nodes exceeds limit of {}",
                result.node_count, result.max_nodes_limit
            );
            return;
        }

        let mut cycle_members = HashSet::new();
        for cycle in result.cycles {
            for node in cycle {
                cycle_members.insert(node);
            }
        }
        for node in cycle_members {
            self.graph[node].is_cycle = true;
        }
    }

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

    pub fn calculate_call_depth(&mut self) {
        use std::collections::{HashMap, VecDeque};

        let mut depths = HashMap::new();

        let entry_points: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].is_public && self.get_callers(idx).is_empty())
            .collect();

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

                if !depths.contains_key(&target) {
                    depths.insert(target, new_depth);
                    queue.push_back((target, new_depth));
                }
            }
        }

        for idx in self.graph.node_indices() {
            self.graph[idx].depth = *depths.get(&idx).unwrap_or(&0);
        }
    }

    pub fn top_important_nodes(&self, max_nodes: usize, min_importance: f64) -> Vec<NodeIndex> {
        let mut candidates: Vec<NodeIndex> = self
            .node_indices()
            .filter(|idx| self.graph[*idx].importance_score > min_importance)
            .collect();

        // Use total_cmp - never panics on NaN
        candidates.sort_by(|a, b| {
            self.graph[*b]
                .importance_score
                .total_cmp(&self.graph[*a].importance_score)
        });
        candidates.truncate(max_nodes);
        candidates
    }
}

crate::impl_graph_metrics!(CallGraph);
crate::impl_graph_index!(CallGraph, FunctionNode);

impl std::ops::Index<petgraph::graph::NodeIndex> for &CallGraph {
    type Output = FunctionNode;

    fn index(&self, index: petgraph::graph::NodeIndex) -> &Self::Output {
        &self.graph[index]
    }
}
