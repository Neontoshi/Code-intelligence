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

    /// Collect all types that are actually used in the codebase.
    /// Language-agnostic - works for Rust, TypeScript, Python, Java, Go, etc.
    fn collect_used_types(type_graph: &TypeGraph, call_graph: &CallGraph) -> HashSet<String> {
        let mut used_types = HashSet::new();

        // 1. Check function signatures (params and returns)
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

        // 2. Check type graph for relationships (language-agnostic)
        for node in type_graph.iter_nodes() {
            // Check if this type is a supertype of another type
            if type_graph.has_subtypes(&node.name) {
                used_types.insert(node.name.clone());
            }

            // Check fields for structs/enums/classes/interfaces
            if matches!(
                node.kind,
                TypeKind::Struct | TypeKind::Enum | TypeKind::Class | TypeKind::Interface
            ) {
                for field in &node.fields {
                    Self::extract_type_names(&field.field_type, &mut used_types);
                }
            }

            // Check generic parameters
            for generic in &node.generics {
                Self::extract_type_names(generic, &mut used_types);
            }

            // Check implementors (trait impls, interface impls)
            let implementors = type_graph.get_subtypes(&node.name);
            if !implementors.is_empty() {
                used_types.insert(node.name.clone());
                for impl_type in implementors {
                    used_types.insert(impl_type.name.clone());
                }
            }
        }

        // 3. Check impl blocks
        for node in type_graph.iter_nodes() {
            if let TypeKind::Impl = node.kind {
                Self::extract_type_names(&node.name, &mut used_types);
            }
        }

        used_types
    }

    /// Extract type names from a type string - language-agnostic.
    /// Handles:
    /// - Generics: `Type<T>`, `Type<T, U>`, `Type<T extends Base>`
    /// - Arrays: `Type[]`, `[T]`, `Array<T>`
    /// - Unions: `Type1 | Type2`, `Type1 & Type2`
    /// - Nullable: `Type?`, `?Type`, `optional Type`
    /// - Module paths: `module.Type`, `namespace.Type`
    /// - Type annotations: `: Type`, `-> Type`, `as Type`
    /// - Implements: `implements Type`, `extends Type`
    fn extract_type_names(type_str: &str, used_types: &mut HashSet<String>) {
        const MAX_ITERATIONS: usize = 5000; // hard safety cap

        let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        let mut seen: HashSet<String> = HashSet::new();
        queue.push_back(type_str.to_string());

        let mut iterations = 0;
        while let Some(type_str) = queue.pop_front() {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                eprintln!(
                        "⚠️ extract_type_names: hit iteration cap, likely malformed type string: {:.80}",
                        type_str
                    );
                break;
            }

            if type_str.is_empty() || !seen.insert(type_str.clone()) {
                continue; // empty, or we've already processed this exact string
            }

            let clean = type_str
                .trim()
                .trim_start_matches('?')
                .trim_start_matches('&')
                .trim_start_matches('*')
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim_start_matches("mut ")
                .trim_start_matches("const ")
                .trim_start_matches("readonly ")
                .trim_start_matches("optional ")
                .trim_start_matches("async ")
                .trim_start_matches("await ")
                .trim_start_matches("typeof ")
                .trim_start_matches("keyof ")
                .trim_start_matches("readonly ");

            let primitives = [
                "u8",
                "u16",
                "u32",
                "u64",
                "u128",
                "i8",
                "i16",
                "i32",
                "i64",
                "i128",
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
            ];

            if let Some(open_bracket) = clean.find('<') {
                let base = &clean[..open_bracket];
                if let Some(base_clean) = base.split("::").last().or_else(|| base.split('.').last())
                {
                    let base_clean = base_clean.trim();
                    if !primitives.contains(&base_clean) && !base_clean.is_empty() {
                        used_types.insert(base_clean.to_string());
                    }
                }

                if let Some(close_bracket) = clean.rfind('>') {
                    if close_bracket > open_bracket {
                        let generics = &clean[open_bracket + 1..close_bracket];
                        let mut depth = 0;
                        let mut current = String::new();

                        for ch in generics.chars() {
                            match ch {
                                '<' => depth += 1,
                                '>' => depth -= 1,
                                ',' | '|' | '&' if depth == 0 => {
                                    if !current.is_empty() {
                                        queue.push_back(std::mem::take(&mut current));
                                    }
                                    continue;
                                }
                                _ => {}
                            }
                            if depth >= 0 || ch != ',' {
                                current.push(ch);
                            }
                        }
                        if !current.is_empty() {
                            queue.push_back(current);
                        }
                    }
                }
            } else {
                let last = if clean.contains('.') {
                    clean.split('.').last()
                } else if clean.contains("::") {
                    clean.split("::").last()
                } else if clean.contains('/') {
                    clean.split('/').last()
                } else {
                    Some(clean)
                };

                if let Some(last) = last {
                    let last = last
                        .trim()
                        .trim_start_matches('"')
                        .trim_end_matches('"')
                        .trim_start_matches('\'')
                        .trim_end_matches('\'')
                        .trim_start_matches('`')
                        .trim_end_matches('`');

                    if !primitives.contains(&last)
                        && !last.is_empty()
                        && !last.chars().all(|c| c.is_ascii_digit())
                    {
                        used_types.insert(last.to_string());
                    }
                }
            }

            let patterns = [
                ":",
                "->",
                "=>",
                "as",
                "implements",
                "extends",
                "typeof",
                "keyof",
            ];
            for pattern in patterns {
                if let Some(pos) = type_str.find(pattern) {
                    let after = type_str[pos + pattern.len()..].to_string();
                    if after != type_str {
                        queue.push_back(after);
                    }
                }
            }

            if let Some(start) = type_str.find('{') {
                if let Some(end) = type_str.rfind('}') {
                    if end > start {
                        let inner = &type_str[start + 1..end];
                        for part in inner.split(',') {
                            if let Some(colon) = part.find(':') {
                                queue.push_back(part[colon + 1..].to_string());
                            }
                        }
                    }
                }
            }

            if type_str.contains("[]") || type_str.starts_with('[') {
                let inner = type_str
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();
                if inner != type_str {
                    queue.push_back(inner);
                }
            }

            if let Some(arrow) = type_str.find("=>") {
                let return_type = type_str[arrow + 2..].to_string();
                if return_type != type_str {
                    queue.push_back(return_type);
                }
            }

            if type_str.contains('|') {
                for part in type_str.split('|') {
                    if part != type_str {
                        queue.push_back(part.to_string());
                    }
                }
            }

            if type_str.contains('&') {
                for part in type_str.split('&') {
                    if part != type_str {
                        queue.push_back(part.to_string());
                    }
                }
            }
        }
    }

    fn calculate_type_confidence(type_graph: &TypeGraph, type_name: &str) -> f64 {
        let mut confidence: f64 = 0.7;

        // Check if type is in test directory
        if type_name.contains("test") || type_name.contains("Test") || type_name.contains("Test") {
            confidence -= 0.2;
        }

        // Check if type has generics (may be used more widely)
        if type_name.contains('<') || type_name.contains('[') {
            confidence += 0.1;
        }

        // Check inheritance depth
        let depth = type_graph.get_inheritance_depth(type_name);
        if depth > 0 {
            confidence -= 0.2;
        }

        // Check if type has subtypes
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
