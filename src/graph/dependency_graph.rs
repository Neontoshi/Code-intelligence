// src/graph/dependency_graph.rs

use crate::define_graph;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub edge_type: DependencyType,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Import,
    Crate,
    Package,
    Module,
    Local,
}

// ⭐ The macro creates the struct and new() function
define_graph!(DependencyGraph, DependencyNode, DependencyEdge);

// ⭐ All other methods go in a separate impl block
impl DependencyGraph {
    pub fn add_node(&mut self, node: DependencyNode) -> NodeIndex {
        let key = node.name.clone();
        if let Some(&idx) = self.node_index.get(&key) {
            return idx;
        }

        let idx = self.graph.add_node(node);
        self.node_index.insert(key, idx);
        idx
    }

    pub fn add_dependency(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        edge_type: DependencyType,
        line: usize,
    ) {
        let edge = DependencyEdge { edge_type, line };
        self.graph.add_edge(from, to, edge);
    }

    pub fn add_dependency_by_name(
        &mut self,
        from: &str,
        to: &str,
        edge_type: DependencyType,
        line: usize,
    ) {
        if let (Some(&from_idx), Some(&to_idx)) =
            (self.node_index.get(from), self.node_index.get(to))
        {
            self.add_dependency(from_idx, to_idx, edge_type, line);
        }
    }

    pub fn get_dependencies(&self, node: NodeIndex) -> Vec<&DependencyNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .map(|e| &self.graph[e.target()])
            .collect()
    }

    pub fn get_dependents(&self, node: NodeIndex) -> Vec<&DependencyNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .map(|e| &self.graph[e.source()])
            .collect()
    }

    pub fn find_cycles(&self) -> Vec<Vec<NodeIndex>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        for start in self.graph.node_indices() {
            if !visited.contains(&start) {
                self.dfs_cycles(start, &mut visited, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycles(
        &self,
        current: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
        cycles: &mut Vec<Vec<NodeIndex>>,
    ) {
        if let Some(pos) = path.iter().position(|&n| n == current) {
            // Found a cycle
            let cycle = path[pos..].to_vec();
            if !cycles.contains(&cycle) {
                cycles.push(cycle);
            }
            return;
        }

        if visited.contains(&current) {
            return;
        }

        visited.insert(current);
        path.push(current);

        for edge in self
            .graph
            .edges_directed(current, petgraph::Direction::Outgoing)
        {
            self.dfs_cycles(edge.target(), visited, path, cycles);
        }

        path.pop();
    }

    pub fn topological_sort(&self) -> Vec<&DependencyNode> {
        let mut sorted = Vec::new();
        let mut indegree = HashMap::new();

        // Calculate indegree
        for node in self.graph.node_indices() {
            let degree = self
                .graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .count();
            indegree.insert(node, degree);
        }

        let mut queue: Vec<_> = indegree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&node, _)| node)
            .collect();

        while let Some(node) = queue.pop() {
            sorted.push(&self.graph[node]);

            for edge in self
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
            {
                let target = edge.target();
                if let Some(degree) = indegree.get_mut(&target) {
                    *degree -= 1;

                    if *degree == 0 {
                        queue.push(target);
                    }
                }
            }
        }

        sorted
    }
}

crate::impl_graph_metrics!(DependencyGraph);
crate::impl_graph_index!(DependencyGraph, DependencyNode);
