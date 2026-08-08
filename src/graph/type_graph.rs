// src/graph/type_graph.rs

use crate::define_graph;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TypeNode {
    pub name: String,
    pub file: String,
    pub line: usize,
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
    Impl,
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

define_graph!(TypeGraph, TypeNode, TypeEdge);

impl TypeGraph {
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

    fn get_related_types(&self, type_name: &str, direction: petgraph::Direction) -> Vec<&TypeNode> {
        if let Some(&idx) = self.node_index.get(type_name) {
            self.graph
                .edges_directed(idx, direction)
                .filter_map(|e| {
                    if matches!(e.weight().relationship, TypeRelationship::Extends) {
                        let node_idx = if direction == petgraph::Direction::Incoming {
                            e.source()
                        } else {
                            e.target()
                        };
                        Some(&self.graph[node_idx])
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_subtypes(&self, type_name: &str) -> Vec<&TypeNode> {
        self.get_related_types(type_name, petgraph::Direction::Incoming)
    }

    pub fn get_supertypes(&self, type_name: &str) -> Vec<&TypeNode> {
        self.get_related_types(type_name, petgraph::Direction::Outgoing)
    }

    pub fn inheritance_depth(&self, type_name: &str) -> usize {
        let mut depth = 0;
        let mut current = type_name.to_string();
        let mut visited = HashSet::new();

        while let Some(&_idx) = self.node_index.get(&current) {
            if visited.contains(&current) {
                break;
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

    // ================================================================
    // NEW METHODS
    // ================================================================

    pub fn iter_nodes(&self) -> impl Iterator<Item = &TypeNode> {
        self.graph.node_indices().map(|idx| &self.graph[idx])
    }

    pub fn get_type(&self, name: &str) -> Option<&TypeNode> {
        self.node_index
            .get(name)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    pub fn is_type_used_in_functions(
        &self,
        type_name: &str,
        call_graph: &crate::graph::call_graph::CallGraph,
    ) -> bool {
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            for param in &func.params {
                if param == type_name {
                    return true;
                }
            }
            for ret in &func.returns {
                if ret == type_name {
                    return true;
                }
            }
        }
        false
    }

    pub fn find_unused_types(
        &self,
        call_graph: &crate::graph::call_graph::CallGraph,
    ) -> Vec<&TypeNode> {
        self.iter_nodes()
            .filter(|node| !self.is_type_used_in_functions(&node.name, call_graph))
            .collect()
    }

    pub fn get_inheritance_depth(&self, type_name: &str) -> usize {
        let mut depth = 0;
        let mut current = type_name;
        let mut visited = HashSet::new();

        while let Some(_node) = self.get_type(current) {
            if visited.contains(current) {
                break;
            }
            visited.insert(current);

            let supertypes = self.get_supertypes(current);
            if supertypes.is_empty() {
                break;
            }

            current = &supertypes[0].name;
            depth += 1;
        }

        depth
    }

    pub fn max_inheritance_depth(&self) -> usize {
        self.iter_nodes()
            .map(|node| self.get_inheritance_depth(&node.name))
            .max()
            .unwrap_or(0)
    }

    pub fn leaf_types(&self) -> Vec<&TypeNode> {
        self.iter_nodes()
            .filter(|node| self.get_subtypes(&node.name).is_empty())
            .collect()
    }

    pub fn root_types(&self) -> Vec<&TypeNode> {
        self.iter_nodes()
            .filter(|node| self.get_supertypes(&node.name).is_empty())
            .collect()
    }

    pub fn has_subtypes(&self, type_name: &str) -> bool {
        !self
            .get_related_types(type_name, petgraph::Direction::Incoming)
            .is_empty()
    }

    pub fn has_supertypes(&self, type_name: &str) -> bool {
        !self
            .get_related_types(type_name, petgraph::Direction::Outgoing)
            .is_empty()
    }

    pub fn inheritance_chain(&self, type_name: &str) -> Vec<&TypeNode> {
        let mut chain = Vec::new();
        let mut current = type_name;
        let mut visited = HashSet::new();

        while let Some(node) = self.get_type(current) {
            if visited.contains(current) {
                break;
            }
            visited.insert(current);
            chain.push(node);

            let supertypes = self.get_supertypes(current);
            if supertypes.is_empty() {
                break;
            }
            current = &supertypes[0].name;
        }

        chain
    }

    pub fn get_all_types(&self) -> Vec<&TypeNode> {
        self.iter_nodes().collect()
    }

    pub fn get_types_by_kind(&self, kind: TypeKind) -> Vec<&TypeNode> {
        self.iter_nodes().filter(|node| node.kind == kind).collect()
    }

    pub fn has_type(&self, name: &str) -> bool {
        self.node_index.contains_key(name)
    }

    pub fn type_count(&self) -> usize {
        self.node_index.len()
    }
}

crate::impl_graph_metrics!(TypeGraph);
