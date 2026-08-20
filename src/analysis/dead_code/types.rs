// src/analysis/dead_code/types.rs

use crate::graph::call_graph::CallGraph;
use crate::graph::type_graph::{TypeGraph, TypeKind};
use std::collections::HashSet;

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

        let used_types = Self::collect_used_types(type_graph, call_graph);

        for node in type_graph.iter_nodes() {
            let type_name = &node.name;
            let type_kind = &node.kind;
            let file = &node.file;
            let line = node.line;

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

                match type_kind {
                    TypeKind::Struct => unused_structs.push(dead_type),
                    TypeKind::Enum => unused_enums.push(dead_type),
                    TypeKind::Trait => unused_traits.push(dead_type),
                    TypeKind::TypeAlias => unused_type_aliases.push(dead_type),
                    TypeKind::Impl => unused_impl_blocks.push(dead_type),
                    _ => {}
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

    fn collect_used_types(type_graph: &TypeGraph, call_graph: &CallGraph) -> HashSet<String> {
        let mut used_types = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            for param in &func.params {
                Self::extract_type_names(param, &mut used_types);
            }

            for ret in &func.returns {
                Self::extract_type_names(ret, &mut used_types);
            }

            if let Some(trait_name) = &func.trait_impl {
                Self::extract_type_names(trait_name, &mut used_types);
            }
        }

        for node in type_graph.iter_nodes() {
            if type_graph.has_subtypes(&node.name) {
                used_types.insert(node.name.clone());
            }

            if matches!(
                node.kind,
                TypeKind::Struct | TypeKind::Enum | TypeKind::Class | TypeKind::Interface
            ) {
                for field in &node.fields {
                    Self::extract_type_names(&field.field_type, &mut used_types);
                }
            }

            for generic in &node.generics {
                Self::extract_type_names(generic, &mut used_types);
            }

            let implementors = type_graph.get_subtypes(&node.name);
            if !implementors.is_empty() {
                used_types.insert(node.name.clone());
                for impl_type in implementors {
                    used_types.insert(impl_type.name.clone());
                }
            }
        }

        for node in type_graph.iter_nodes() {
            if let TypeKind::Impl = node.kind {
                Self::extract_type_names(&node.name, &mut used_types);
            }
        }

        used_types
    }

    /// Single-pass zero-allocation lexical scanner to extract identifiers from type signatures
    fn extract_type_names(type_str: &str, used_types: &mut HashSet<String>) {
        let primitives: HashSet<&'static str> = [
            "u8",
            "u16",
            "u32",
            "u64",
            "u128",
            "usize",
            "i8",
            "i16",
            "i32",
            "i64",
            "i128",
            "isize",
            "f32",
            "f64",
            "bool",
            "char",
            "str",
            "string",
            "number",
            "boolean",
            "symbol",
            "bigint",
            "undefined",
            "null",
            "any",
            "unknown",
            "never",
            "void",
            "int",
            "float",
            "complex",
            "bytes",
            "list",
            "tuple",
            "dict",
            "set",
            "frozenset",
            "byte",
            "short",
            "long",
            "double",
            "rune",
            "error",
            "size_t",
            "ssize_t",
            "intptr_t",
            "uintptr_t",
            "String",
            "Number",
            "Boolean",
            "Object",
            "Array",
            "Function",
            "Promise",
            "Error",
            "Date",
            "RegExp",
            "mut",
            "const",
            "readonly",
            "self",
            "Self",
            "dyn",
            "impl",
            "typeof",
            "keyof",
            "extends",
            "implements",
        ]
        .into_iter()
        .collect();

        let mut token_start = None;
        let bytes = type_str.as_bytes();

        for (i, &b) in bytes.iter().enumerate() {
            let is_ident_char = b.is_ascii_alphanumeric() || b == b'_';

            if is_ident_char {
                if token_start.is_none() {
                    token_start = Some(i);
                }
            } else if let Some(start) = token_start {
                let token = &type_str[start..i];
                if !token.chars().all(|c| c.is_ascii_digit()) && !primitives.contains(token) {
                    used_types.insert(token.to_string());
                }
                token_start = None;
            }
        }

        if let Some(start) = token_start {
            let token = &type_str[start..];
            if !token.chars().all(|c| c.is_ascii_digit()) && !primitives.contains(token) {
                used_types.insert(token.to_string());
            }
        }
    }

    fn calculate_type_confidence(type_graph: &TypeGraph, type_name: &str) -> f64 {
        let mut confidence: f64 = 0.7;

        if type_name.contains("test") || type_name.contains("Test") {
            confidence -= 0.2;
        }

        if type_name.contains('<') || type_name.contains('[') {
            confidence += 0.1;
        }

        let depth = type_graph.get_inheritance_depth(type_name);
        if depth > 0 {
            confidence -= 0.2;
        }

        if type_graph.has_subtypes(type_name) {
            confidence -= 0.2;
        }

        confidence.max(0.0).min(1.0)
    }

    fn get_reason(kind: &TypeKind, confidence: f64) -> String {
        let kind_str = match kind {
            TypeKind::Struct => "struct",
            TypeKind::Enum => "enum",
            TypeKind::Trait => "trait/interface",
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
