use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TypeNode {
    pub name: String,
    pub file: String,
    pub kind: TypeKind,
    pub fields: Vec<Field>,
    pub methods: Vec<String>,
    pub supertypes: Vec<String>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
    Interface,
    Class,
    TypeAlias,
    Union,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct TypeEdge {
    pub relationship: TypeRelationship,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeRelationship {
    Extends,
    Implements,
    Uses,
    Contains,
    References,
}

#[derive(Debug)]
pub struct TypeGraph {
    graph: DiGraph<TypeNode, TypeEdge>,
    node_index: HashMap<String, NodeIndex>,
}

impl TypeGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_index: HashMap::new(),
        }
    }

    pub fn add_type(&mut self, type_node: TypeNode) -> NodeIndex {
        let key = type_node.name.clone();
        if let Some(&idx) = self.node_index.get(&key) {
            return idx;
        }

        let idx = self.graph.add_node(type_node);
        self.node_index.insert(key, idx);
        idx
    }

    pub fn add_relationship(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        relationship: TypeRelationship,
        line: usize,
    ) {
        let edge = TypeEdge { relationship, line };
        self.graph.add_edge(from, to, edge);
    }

    pub fn add_relationship_by_name(
        &mut self,
        from: &str,
        to: &str,
        relationship: TypeRelationship,
        line: usize,
    ) {
        if let (Some(&from_idx), Some(&to_idx)) =
            (self.node_index.get(from), self.node_index.get(to))
        {
            self.add_relationship(from_idx, to_idx, relationship, line);
        }
    }

    pub fn get_subtypes(&self, type_name: &str) -> Vec<&TypeNode> {
        if let Some(&idx) = self.node_index.get(type_name) {
            self.graph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .filter_map(|e| {
                    if matches!(e.weight().relationship, TypeRelationship::Extends) {
                        Some(&self.graph[e.source()])
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_supertypes(&self, type_name: &str) -> Vec<&TypeNode> {
        if let Some(&idx) = self.node_index.get(type_name) {
            self.graph
                .edges_directed(idx, petgraph::Direction::Outgoing)
                .filter_map(|e| {
                    if matches!(e.weight().relationship, TypeRelationship::Extends) {
                        Some(&self.graph[e.target()])
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn inheritance_depth(&self, type_name: &str) -> usize {
        let mut depth = 0;
        let mut current = type_name.to_string();
        let mut visited = HashSet::new();

        while let Some(&_idx) = self.node_index.get(&current) {
            if visited.contains(&current) {
                break; // Cycle detected
            }
            visited.insert(current.clone());

            let supertypes = self.get_supertypes(&current);
            if supertypes.is_empty() {
                break;
            }

            depth += 1;
            current = supertypes[0].name.clone();
        }

        depth
    }

    pub fn inheritance_tree(&self, root: &str) -> Vec<&TypeNode> {
        let mut tree = Vec::new();
        if let Some(&idx) = self.node_index.get(root) {
            tree.push(&self.graph[idx]);
            self.collect_subtypes(idx, &mut tree);
        }
        tree
    }

    fn collect_subtypes<'a>(&'a self, parent: NodeIndex, tree: &mut Vec<&'a TypeNode>) {
        let children: Vec<_> = self
            .graph
            .edges_directed(parent, petgraph::Direction::Incoming)
            .filter(|e| matches!(e.weight().relationship, TypeRelationship::Extends))
            .map(|e| e.source())
            .collect();

        for child in children {
            tree.push(&self.graph[child]);
            self.collect_subtypes(child, tree);
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
