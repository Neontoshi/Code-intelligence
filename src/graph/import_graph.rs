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

        for file in files {
            for import in &file.imports {
                let resolved_target = if self.nodes.contains_key(&import.module) {
                    Some(import.module.clone())
                } else {
                    Self::resolve_relative_import(&file.path, &import.module, &self.nodes)
                };

                if let Some(target) = resolved_target {
                    self.edges.push(ImportEdge {
                        source_file: file.path.clone(),
                        target_file: target.clone(),
                        import_info: import.clone(),
                    });

                    self.adjacency
                        .entry(file.path.clone())
                        .or_insert_with(Vec::new)
                        .push(target);
                }
            }

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

    fn get_edges_by_file(&self, file: &str, as_importer: bool) -> Vec<&ImportEdge> {
        if as_importer {
            self.edges
                .iter()
                .filter(|e| e.target_file == file)
                .collect()
        } else {
            self.edges
                .iter()
                .filter(|e| e.source_file == file)
                .collect()
        }
    }

    pub fn get_imports(&self, file: &str) -> Vec<&ImportEdge> {
        self.get_edges_by_file(file, false)
    }

    fn resolve_relative_import(
        source_file: &str,
        import_module: &str,
        nodes: &HashMap<String, ImportNode>,
    ) -> Option<String> {
        let clean_module = import_module.trim_matches(|c| c == '\'' || c == '"');
        if !clean_module.starts_with('.') {
            return None;
        }

        let source_path = std::path::Path::new(source_file);
        let parent = source_path.parent()?;
        let joined = parent.join(clean_module);

        let candidates = [
            joined.to_string_lossy().to_string(),
            format!("{}.ts", joined.display()),
            format!("{}.tsx", joined.display()),
            format!("{}.js", joined.display()),
            format!("{}.jsx", joined.display()),
            joined.join("index.ts").to_string_lossy().to_string(),
            joined.join("index.tsx").to_string_lossy().to_string(),
            joined.join("index.js").to_string_lossy().to_string(),
        ];

        for candidate in &candidates {
            if nodes.contains_key(candidate) {
                return Some(candidate.clone());
            }
        }

        // Canonicalized fallback comparison
        if let Ok(canon_joined) = joined.canonicalize() {
            let canon_str = canon_joined.to_string_lossy().to_string();
            if nodes.contains_key(&canon_str) {
                return Some(canon_str);
            }
        }

        None
    }

    pub fn get_importers(&self, file: &str) -> Vec<&ImportEdge> {
        self.get_edges_by_file(file, true)
    }

    pub fn is_exported(&self, file: &str, func_name: &str) -> bool {
        self.exports
            .get(file)
            .map(|funcs| funcs.contains(&func_name.to_string()))
            .unwrap_or(false)
    }

    pub fn get_exports(&self, file: &str) -> Vec<String> {
        self.exports.get(file).cloned().unwrap_or_default()
    }

    // ⭐ NEW: Get all functions imported from a specific module
    pub fn get_imported_functions(&self, module: &str) -> Vec<String> {
        let mut functions = Vec::new();
        if let Some(exports) = self.exports.get(module) {
            functions.extend(exports.clone());
        }
        functions
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
                continue;
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

    pub fn iter_nodes(&self) -> impl Iterator<Item = &ImportNode> {
        self.nodes.values()
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = &ImportEdge> {
        self.edges.iter()
    }

    pub fn get_importing_files(&self, module: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.target_file == module)
            .map(|e| e.source_file.as_str())
            .collect()
    }

    pub fn get_imported_modules(&self, file: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.source_file == file)
            .map(|e| e.target_file.as_str())
            .collect()
    }

    pub fn is_imported(&self, module: &str) -> bool {
        self.edges.iter().any(|e| e.target_file == module)
    }

    pub fn find_unimported_files(&self) -> Vec<String> {
        self.nodes
            .keys()
            .filter(|&file| !self.is_imported(file))
            .cloned()
            .collect()
    }

    pub fn import_count(&self, file: &str) -> usize {
        self.edges.iter().filter(|e| e.target_file == file).count()
    }

    pub fn find_unused_imports(&self) -> Vec<ImportEdge> {
        self.edges
            .iter()
            .filter(|e| {
                let source_file = &e.source_file;
                let imported_funcs = self.get_imported_functions(&e.target_file);

                let mut any_used = false;
                for func in &imported_funcs {
                    if self.is_function_used_in_file(func, source_file) {
                        any_used = true;
                        break;
                    }
                }

                !any_used
            })
            .cloned()
            .collect()
    }

    pub fn is_function_used(&self, func_name: &str) -> bool {
        for edge in &self.edges {
            if edge.import_info.items.contains(&func_name.to_string()) {
                return true;
            }
        }
        false
    }

    // ⭐ NEW: Check if a function is used in a specific file
    pub fn is_function_used_in_file(&self, func_name: &str, file: &str) -> bool {
        for edge in &self.edges {
            if edge.source_file == file {
                if edge.import_info.items.contains(&func_name.to_string()) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_module_depth(&self, module: &str) -> usize {
        use std::collections::HashSet;

        let mut depth = 0;
        let mut current = module;
        let mut visited = HashSet::new();

        while let Some(importers) = self.adjacency.get(current) {
            if visited.contains(current) {
                break;
            }
            visited.insert(current);

            if let Some(importer) = importers.iter().find(|&i| i != current) {
                current = importer;
                depth += 1;
            } else {
                break;
            }
        }

        depth
    }
}
