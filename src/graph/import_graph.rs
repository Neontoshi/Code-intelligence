// src/graph/import_graph.rs

use crate::parser::tree_sitter::{ImportInfo, ParsedFile};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ImportNode {
    pub name: String,
    pub path: String,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct ImportEdge {
    pub source_file: String,
    pub target_file: String,
    pub import_info: ImportInfo,
}

#[derive(Debug)]
pub struct ImportGraph {
    nodes: HashMap<String, ImportNode>,
    edges: Vec<ImportEdge>,
    adjacency: HashMap<String, Vec<String>>,
    // ⭐ NEW: Track exports
    exports: HashMap<String, Vec<String>>,
}

impl ImportGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            exports: HashMap::new(),
        }
    }

    pub fn build_from_files(&mut self, files: &[ParsedFile]) {
        for file in files {
            let node_name = file.path.clone();
            let node = ImportNode {
                name: node_name.clone(),
                path: file.path.clone(),
                is_external: false,
            };
            self.nodes.insert(node_name, node);
        }

        // Build import relationships
        for file in files {
            for import in &file.imports {
                let target = import.module.clone();
                if self.nodes.contains_key(&target) {
                    self.edges.push(ImportEdge {
                        source_file: file.path.clone(),
                        target_file: target.clone(),
                        import_info: import.clone(),
                    });

                    // Update adjacency
                    self.adjacency
                        .entry(file.path.clone())
                        .or_insert_with(Vec::new)
                        .push(target);
                }
            }

            // ⭐ NEW: Build exports index
            for func in &file.functions {
                if func.is_public {
                    self.exports
                        .entry(file.path.clone())
                        .or_default()
                        .push(func.name.clone());
                }
            }
        }
    }

    pub fn get_imports(&self, file: &str) -> Vec<&ImportEdge> {
        self.edges
            .iter()
            .filter(|e| e.source_file == file)
            .collect()
    }

    pub fn get_importers(&self, file: &str) -> Vec<&ImportEdge> {
        self.edges
            .iter()
            .filter(|e| e.target_file == file)
            .collect()
    }

    // ⭐ NEW: Check if a function is exported from its file
    pub fn is_exported(&self, file: &str, func_name: &str) -> bool {
        self.exports
            .get(file)
            .map(|funcs| funcs.contains(&func_name.to_string()))
            .unwrap_or(false)
    }

    // ⭐ NEW: Get all exports from a file
    pub fn get_exports(&self, file: &str) -> Vec<String> {
        self.exports.get(file).cloned().unwrap_or_default()
    }

    pub fn find_circular_imports(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.dfs_imports(node, &mut visited, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_imports(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if let Some(pos) = path.iter().position(|n| n == current) {
            let cycle = path[pos..].to_vec();
            if !cycles.contains(&cycle) {
                cycles.push(cycle);
            }
            return;
        }

        if visited.contains(current) {
            return;
        }

        visited.insert(current.to_string());
        path.push(current.to_string());

        if let Some(neighbors) = self.adjacency.get(current) {
            for neighbor in neighbors {
                self.dfs_imports(neighbor, visited, path, cycles);
            }
        }

        path.pop();
    }

    pub fn get_import_depth(&self, start: &str) -> HashMap<String, usize> {
        let mut depth = HashMap::new();
        let mut queue = vec![(start.to_string(), 0)];

        while let Some((current, d)) = queue.pop() {
            if d > 10 {
                continue; // Avoid infinite recursion
            }

            if let Some(neighbors) = self.adjacency.get(&current) {
                for neighbor in neighbors {
                    let current_depth = *depth.get(neighbor).unwrap_or(&usize::MAX);
                    if d + 1 < current_depth {
                        depth.insert(neighbor.clone(), d + 1);
                        queue.push((neighbor.clone(), d + 1));
                    }
                }
            }
        }

        depth
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
