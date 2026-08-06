//! Unified project graph - single graph containing all relationship types

use crate::graph::call_graph::{CallEdge, FunctionNode};
use crate::graph::dependency_graph::{DependencyEdge, DependencyNode};
use crate::graph::import_graph::{ImportEdge, ImportNode};
use crate::graph::type_graph::{TypeEdge, TypeNode};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Unified Node Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum ProjectNode {
    Function(FunctionNode),
    Type(TypeNode),
    Module(String, PathBuf),
    File(String, PathBuf),
    Import(ImportNode),
    Dependency(DependencyNode),
}

impl ProjectNode {
    pub fn name(&self) -> &str {
        match self {
            ProjectNode::Function(f) => &f.name,
            ProjectNode::Type(t) => &t.name,
            ProjectNode::Module(name, _) => name,
            ProjectNode::File(name, _) => name,
            ProjectNode::Import(i) => &i.name,
            ProjectNode::Dependency(d) => &d.name,
        }
    }

    pub fn full_path(&self) -> Option<&str> {
        match self {
            ProjectNode::Function(f) => Some(&f.full_path),
            ProjectNode::Type(t) => Some(&t.name),
            ProjectNode::Module(_, path) => path.to_str(),
            ProjectNode::File(_, path) => path.to_str(),
            ProjectNode::Import(i) => Some(&i.path),
            ProjectNode::Dependency(d) => d.path.to_str(),
        }
    }

    pub fn node_type(&self) -> &'static str {
        match self {
            ProjectNode::Function(_) => "function",
            ProjectNode::Type(_) => "type",
            ProjectNode::Module(_, _) => "module",
            ProjectNode::File(_, _) => "file",
            ProjectNode::Import(_) => "import",
            ProjectNode::Dependency(_) => "dependency",
        }
    }
}

// ============================================================================
// Unified Edge Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum ProjectEdge {
    Call(CallEdge),
    Import(ImportEdge),
    Dependency(DependencyEdge),
    Type(TypeEdge),
    Contains,   // Module contains File, File contains Function
    Inherits,   // Type inherits from Type
    Implements, // Type implements Interface
    References, // Function references Type
}

impl ProjectEdge {
    pub fn edge_type(&self) -> &'static str {
        match self {
            ProjectEdge::Call(_) => "call",
            ProjectEdge::Import(_) => "import",
            ProjectEdge::Dependency(_) => "dependency",
            ProjectEdge::Type(_) => "type",
            ProjectEdge::Contains => "contains",
            ProjectEdge::Inherits => "inherits",
            ProjectEdge::Implements => "implements",
            ProjectEdge::References => "references",
        }
    }

    pub fn line(&self) -> Option<usize> {
        match self {
            ProjectEdge::Call(e) => Some(e.line),
            ProjectEdge::Import(e) => Some(e.import_info.line),
            ProjectEdge::Dependency(e) => Some(e.line),
            ProjectEdge::Type(e) => Some(e.line),
            _ => None,
        }
    }
}

// ============================================================================
// Unified Project Graph
// ============================================================================
#[derive(Debug)]
pub struct ProjectGraph {
    graph: DiGraph<ProjectNode, ProjectEdge>,
    // Indexes for fast lookups
    function_index: HashMap<String, NodeIndex>,
    type_index: HashMap<String, NodeIndex>,
    file_index: HashMap<String, NodeIndex>,
    module_index: HashMap<String, NodeIndex>,
    // Type-specific indexes
    function_by_name: HashMap<String, Vec<NodeIndex>>,
    type_by_name: HashMap<String, Vec<NodeIndex>>,
}

impl ProjectGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            function_index: HashMap::new(),
            type_index: HashMap::new(),
            file_index: HashMap::new(),
            module_index: HashMap::new(),
            function_by_name: HashMap::new(),
            type_by_name: HashMap::new(),
        }
    }

    // ========================================================================
    // Node Operations
    // ========================================================================

    pub fn add_function(&mut self, func: FunctionNode) -> NodeIndex {
        let full_path = func.full_path.clone();
        let name = func.name.clone();
        let node = ProjectNode::Function(func);
        let idx = self.graph.add_node(node);

        self.function_index.insert(full_path.clone(), idx);
        self.function_by_name.entry(name).or_default().push(idx);

        // Also add to file index
        if let Some(file) = self.graph[idx].full_path() {
            let file_path = file.split("::").next().unwrap_or(file);
            self.file_index.entry(file_path.to_string()).or_insert(idx);
        }

        idx
    }

    pub fn add_type(&mut self, type_node: TypeNode) -> NodeIndex {
        let name = type_node.name.clone();
        let node = ProjectNode::Type(type_node);
        let idx = self.graph.add_node(node);

        self.type_index.insert(name.clone(), idx);
        self.type_by_name.entry(name).or_default().push(idx);

        idx
    }

    pub fn add_file(&mut self, path: String, _contents: String) -> NodeIndex {
        let name = path.split('/').last().unwrap_or(&path).to_string();
        let node = ProjectNode::File(name.clone(), PathBuf::from(&path));
        let idx = self.graph.add_node(node);

        self.file_index.insert(path, idx);

        idx
    }

    pub fn add_module(&mut self, name: String, path: PathBuf) -> NodeIndex {
        let node = ProjectNode::Module(name.clone(), path);
        let idx = self.graph.add_node(node);

        self.module_index.insert(name, idx);

        idx
    }

    // ========================================================================
    // Edge Operations
    // ========================================================================

    pub fn add_call(&mut self, from: NodeIndex, to: NodeIndex, edge: CallEdge) {
        self.graph.add_edge(from, to, ProjectEdge::Call(edge));
    }

    pub fn add_import(&mut self, from: NodeIndex, to: NodeIndex, edge: ImportEdge) {
        self.graph.add_edge(from, to, ProjectEdge::Import(edge));
    }

    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex, edge: DependencyEdge) {
        self.graph.add_edge(from, to, ProjectEdge::Dependency(edge));
    }

    pub fn add_type_relationship(&mut self, from: NodeIndex, to: NodeIndex, edge: TypeEdge) {
        self.graph.add_edge(from, to, ProjectEdge::Type(edge));
    }

    pub fn add_contains(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ProjectEdge::Contains);
    }

    pub fn add_inherits(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ProjectEdge::Inherits);
    }

    pub fn add_implements(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ProjectEdge::Implements);
    }

    pub fn add_references(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ProjectEdge::References);
    }

    // ========================================================================
    // Query Operations
    // ========================================================================

    pub fn get_function(&self, full_path: &str) -> Option<&ProjectNode> {
        self.function_index
            .get(full_path)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    pub fn get_function_by_name(&self, name: &str) -> Vec<&ProjectNode> {
        self.function_by_name
            .get(name)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.graph.node_weight(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_type(&self, name: &str) -> Option<&ProjectNode> {
        self.type_index
            .get(name)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    pub fn get_file(&self, path: &str) -> Option<&ProjectNode> {
        self.file_index
            .get(path)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    pub fn get_callers(&self, node: NodeIndex) -> Vec<&ProjectNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .filter_map(|e| {
                if matches!(e.weight(), ProjectEdge::Call(_)) {
                    self.graph.node_weight(e.source())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_callees(&self, node: NodeIndex) -> Vec<&ProjectNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .filter_map(|e| {
                if matches!(e.weight(), ProjectEdge::Call(_)) {
                    self.graph.node_weight(e.target())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_imports(&self, node: NodeIndex) -> Vec<&ProjectNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .filter_map(|e| {
                if matches!(e.weight(), ProjectEdge::Import(_)) {
                    self.graph.node_weight(e.target())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_importers(&self, node: NodeIndex) -> Vec<&ProjectNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .filter_map(|e| {
                if matches!(e.weight(), ProjectEdge::Import(_)) {
                    self.graph.node_weight(e.source())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_dependencies(&self, node: NodeIndex) -> Vec<&ProjectNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .filter_map(|e| {
                if matches!(e.weight(), ProjectEdge::Dependency(_)) {
                    self.graph.node_weight(e.target())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_dependents(&self, node: NodeIndex) -> Vec<&ProjectNode> {
        self.graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .filter_map(|e| {
                if matches!(e.weight(), ProjectEdge::Dependency(_)) {
                    self.graph.node_weight(e.source())
                } else {
                    None
                }
            })
            .collect()
    }

    // ========================================================================
    // Cross-Query Operations
    // ========================================================================

    /// Get all types referenced by a function
    pub fn get_types_referenced_by_function(&self, func_idx: NodeIndex) -> Vec<&ProjectNode> {
        let mut types = Vec::new();

        // Check if this is a function node
        if let Some(ProjectNode::Function(func)) = self.graph.node_weight(func_idx) {
            // Look for references to types
            for edge in self
                .graph
                .edges_directed(func_idx, petgraph::Direction::Outgoing)
            {
                if matches!(edge.weight(), ProjectEdge::References) {
                    if let Some(node) = self.graph.node_weight(edge.target()) {
                        if matches!(node, ProjectNode::Type(_)) {
                            types.push(node);
                        }
                    }
                }
            }

            // Also check parameter and return types
            for param in &func.params {
                if let Some(type_node) = self.get_type(param) {
                    types.push(type_node);
                }
            }
            for ret in &func.returns {
                if let Some(type_node) = self.get_type(ret) {
                    types.push(type_node);
                }
            }
        }

        types
    }

    /// Get all functions that call a specific type
    pub fn get_functions_calling_type(&self, type_idx: NodeIndex) -> Vec<&ProjectNode> {
        let mut functions = Vec::new();

        for edge in self
            .graph
            .edges_directed(type_idx, petgraph::Direction::Incoming)
        {
            if matches!(edge.weight(), ProjectEdge::References)
                || matches!(edge.weight(), ProjectEdge::Call(_))
            {
                if let Some(node) = self.graph.node_weight(edge.source()) {
                    if matches!(node, ProjectNode::Function(_)) {
                        functions.push(node);
                    }
                }
            }
        }

        functions
    }

    /// Get the module containing a file
    pub fn get_module_for_file(&self, file_idx: NodeIndex) -> Option<&ProjectNode> {
        for edge in self
            .graph
            .edges_directed(file_idx, petgraph::Direction::Incoming)
        {
            if matches!(edge.weight(), ProjectEdge::Contains) {
                if let Some(node) = self.graph.node_weight(edge.source()) {
                    if matches!(node, ProjectNode::Module(_, _)) {
                        return Some(node);
                    }
                }
            }
        }
        None
    }

    /// Get all files in a module
    pub fn get_files_in_module(&self, module_idx: NodeIndex) -> Vec<&ProjectNode> {
        let mut files = Vec::new();

        for edge in self
            .graph
            .edges_directed(module_idx, petgraph::Direction::Outgoing)
        {
            if matches!(edge.weight(), ProjectEdge::Contains) {
                if let Some(node) = self.graph.node_weight(edge.target()) {
                    if matches!(node, ProjectNode::File(_, _)) {
                        files.push(node);
                    }
                }
            }
        }

        files
    }

    // ========================================================================
    // Analysis Operations
    // ========================================================================

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn node_indices(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.node_indices()
    }

    pub fn get_node(&self, idx: NodeIndex) -> Option<&ProjectNode> {
        self.graph.node_weight(idx)
    }

    pub fn get_edge(&self, idx: petgraph::graph::EdgeIndex) -> Option<&ProjectEdge> {
        self.graph.edge_weight(idx)
    }

    /// Get all functions
    pub fn get_all_functions(&self) -> Vec<&ProjectNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                if matches!(self.graph[idx], ProjectNode::Function(_)) {
                    Some(&self.graph[idx])
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all types
    pub fn get_all_types(&self) -> Vec<&ProjectNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                if matches!(self.graph[idx], ProjectNode::Type(_)) {
                    Some(&self.graph[idx])
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all files
    pub fn get_all_files(&self) -> Vec<&ProjectNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                if matches!(self.graph[idx], ProjectNode::File(_, _)) {
                    Some(&self.graph[idx])
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find cycles in the graph
    pub fn detect_cycles(&self) -> Vec<Vec<NodeIndex>> {
        use std::collections::HashSet;

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            graph: &ProjectGraph,
            node: NodeIndex,
            visited: &mut HashSet<NodeIndex>,
            rec_stack: &mut HashSet<NodeIndex>,
            path: &mut Vec<NodeIndex>,
            cycles: &mut Vec<Vec<NodeIndex>>,
        ) {
            visited.insert(node);
            rec_stack.insert(node);
            path.push(node);

            for edge in graph
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
            {
                let target = edge.target();
                if !visited.contains(&target) {
                    dfs(graph, target, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&target) {
                    if let Some(pos) = path.iter().position(|&n| n == target) {
                        let cycle: Vec<NodeIndex> = path[pos..].to_vec();
                        if !cycles.contains(&cycle) {
                            cycles.push(cycle);
                        }
                    }
                }
            }

            rec_stack.remove(&node);
            path.pop();
        }

        for node in self.graph.node_indices() {
            if !visited.contains(&node) {
                dfs(
                    self,
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// Get node type distribution
    pub fn node_type_distribution(&self) -> HashMap<String, usize> {
        let mut dist = HashMap::new();
        for idx in self.graph.node_indices() {
            let node_type = self.graph[idx].node_type();
            *dist.entry(node_type.to_string()).or_insert(0) += 1;
        }
        dist
    }
}

impl std::ops::Index<NodeIndex> for ProjectGraph {
    type Output = ProjectNode;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.graph[index]
    }
}

// ============================================================================
// Builder for ProjectGraph
// ============================================================================

pub struct ProjectGraphBuilder {
    graph: ProjectGraph,
    file_to_idx: HashMap<String, NodeIndex>,
    func_full_path_to_idx: HashMap<String, NodeIndex>,
}

impl ProjectGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: ProjectGraph::new(),
            file_to_idx: HashMap::new(),
            func_full_path_to_idx: HashMap::new(),
        }
    }

    pub fn add_file(mut self, path: String, contents: String) -> Self {
        let idx = self.graph.add_file(path.clone(), contents);
        self.file_to_idx.insert(path, idx);
        self
    }

    pub fn add_function(mut self, func: FunctionNode, file_path: &str) -> Self {
        let idx = self.graph.add_function(func.clone());
        self.func_full_path_to_idx
            .insert(func.full_path.clone(), idx);

        // Add contains edge: file -> function
        if let Some(&file_idx) = self.file_to_idx.get(file_path) {
            self.graph.add_contains(file_idx, idx);
        }

        self
    }

    pub fn add_call(mut self, from_full_path: &str, to_full_path: &str, edge: CallEdge) -> Self {
        if let (Some(&from), Some(&to)) = (
            self.func_full_path_to_idx.get(from_full_path),
            self.func_full_path_to_idx.get(to_full_path),
        ) {
            self.graph.add_call(from, to, edge);
        }
        self
    }

    pub fn build(self) -> ProjectGraph {
        self.graph
    }
}

impl Default for ProjectGraph {
    fn default() -> Self {
        Self::new()
    }
}
