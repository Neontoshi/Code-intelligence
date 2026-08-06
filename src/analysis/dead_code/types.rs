// src/analysis/dead_code/types.rs

use crate::graph::call_graph::CallGraph;
use crate::graph::type_graph::{TypeGraph, TypeKind};

#[derive(Debug, Clone)]
pub struct DeadTypeReport {
    pub unused_structs: Vec<DeadType>,
    pub unused_enums: Vec<DeadType>,
    pub unused_traits: Vec<DeadType>,
    pub unused_type_aliases: Vec<DeadType>,
    pub unused_impl_blocks: Vec<DeadType>,
}

#[derive(Debug, Clone)]
pub struct DeadType {
    pub name: String,
    pub file: String,
    pub kind: TypeKind,
    pub line: usize,
    pub confidence: f64,
    pub reason: String,
}

pub struct TypeDeadCodeDetector;

impl TypeDeadCodeDetector {
    pub fn detect_dead_types(_type_graph: &TypeGraph, _call_graph: &CallGraph) -> DeadTypeReport {
        // TODO: Implement once TypeGraph supports iteration
        DeadTypeReport {
            unused_structs: Vec::new(),
            unused_enums: Vec::new(),
            unused_traits: Vec::new(),
            unused_type_aliases: Vec::new(),
            unused_impl_blocks: Vec::new(),
        }
    }
}
