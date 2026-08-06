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
    pub fn detect_dead_types(type_graph: &TypeGraph, call_graph: &CallGraph) -> DeadTypeReport {
        let mut unused_structs = Vec::new();
        let mut unused_enums = Vec::new();
        let mut unused_traits = Vec::new();
        let mut unused_type_aliases = Vec::new();
        let mut unused_impl_blocks = Vec::new();

        // Collect all type names used in function signatures
        let used_types = Self::collect_used_types(call_graph);

        // Check each type in the graph
        for node in type_graph.iter_nodes() {
            let type_name = &node.name;
            let type_kind = &node.kind;
            let file = &node.file;
            let line = node.line; // Now this exists!

            // Check if this type is used anywhere
            let is_used = used_types.contains(type_name);

            if !is_used {
                let confidence = Self::calculate_type_confidence(type_graph, type_name);

                let dead_type = DeadType {
                    name: type_name.clone(),
                    file: file.clone(),
                    kind: type_kind.clone(),
                    line,
                    confidence,
                    reason: Self::get_reason(type_kind, confidence),
                };

                // Classify by kind
                match type_kind {
                    TypeKind::Struct => unused_structs.push(dead_type),
                    TypeKind::Enum => unused_enums.push(dead_type),
                    TypeKind::Trait => unused_traits.push(dead_type),
                    TypeKind::TypeAlias => unused_type_aliases.push(dead_type),
                    TypeKind::Impl => unused_impl_blocks.push(dead_type),
                    _ => { /* Other types - ignore for now */ }
                }
            }
        }

        DeadTypeReport {
            unused_structs,
            unused_enums,
            unused_traits,
            unused_type_aliases,
            unused_impl_blocks,
        }
    }

    fn collect_used_types(call_graph: &CallGraph) -> Vec<String> {
        let mut used_types = Vec::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Check parameter types
            for param in &func.params {
                used_types.push(param.clone());
            }

            // Check return types
            for ret in &func.returns {
                used_types.push(ret.clone());
            }

            // Check doc comments for type mentions
            if let Some(doc) = &func.doc_comment {
                // Extract potential type names from doc comments
                // This is a simplified version
                for word in doc.split_whitespace() {
                    // Check if it looks like a type name (starts with uppercase)
                    if word.chars().next().map_or(false, |c| c.is_uppercase()) {
                        // Clean up the word (remove punctuation)
                        let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
                        if !clean_word.is_empty() {
                            used_types.push(clean_word.to_string());
                        }
                    }
                }
            }
        }

        // Remove duplicates
        used_types.sort();
        used_types.dedup();
        used_types
    }

    fn calculate_type_confidence(type_graph: &TypeGraph, type_name: &str) -> f64 {
        let mut confidence: f64 = 0.7; // Base confidence

        // Check if the type is in a public API (starts with pub)
        // This is a simplified check
        if type_name.starts_with("Pub") || type_name.starts_with("pub") {
            confidence -= 0.3;
        }

        // If it's in a test module, lower confidence
        if type_name.contains("test") || type_name.contains("Test") {
            confidence -= 0.2;
        }

        // If it's a generic type, it might be used more widely
        if type_name.contains('<') {
            confidence += 0.1;
        }

        // Check if the type has any supertypes (inheritance)
        let depth = type_graph.get_inheritance_depth(type_name);
        if depth > 0 {
            confidence -= 0.2; // If it inherits from something, it might be important
        }

        confidence.max(0.0).min(1.0)
    }

    fn get_reason(kind: &TypeKind, confidence: f64) -> String {
        let kind_str = match kind {
            TypeKind::Struct => "struct",
            TypeKind::Enum => "enum",
            TypeKind::Trait => "trait",
            TypeKind::TypeAlias => "type alias",
            TypeKind::Impl => "impl block",
            TypeKind::Interface => "interface",
            TypeKind::Class => "class",
            TypeKind::Union => "union",
        };

        if confidence > 0.8 {
            format!("Unused {} with high confidence of being dead", kind_str)
        } else if confidence > 0.5 {
            format!("Unused {} with moderate confidence", kind_str)
        } else {
            format!("Unused {} but might be used indirectly", kind_str)
        }
    }
}
