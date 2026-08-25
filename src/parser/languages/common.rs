// src/parser/languages/common.rs

//! Common utilities for language parsers

use crate::parser::tree_sitter::{FunctionRole, ParamInfo};
use tree_sitter::Node;

/// Parse parameters from a parameter list node
pub fn parse_parameters(node: &Node, source: &str) -> Vec<ParamInfo> {
    let mut params = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "parameter" || child.kind() == "typed_parameter" {
            let name = child
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .unwrap_or_else(|| {
                    child
                        .children(&mut child.walk())
                        .filter(|c| c.kind() == "identifier")
                        .last()
                        .and_then(|c| c.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("unknown")
                })
                .to_string();

            let type_hint = child
                .child_by_field_name("type")
                .and_then(|t| t.utf8_text(source.as_bytes()).ok())
                .map(|s| s.to_string());

            params.push(ParamInfo { name, type_hint });
        }
    }

    params
}

/// Extract function name from a node
pub fn extract_function_name(node: &Node, source: &str, lang_name: &str) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        if name_node.kind() == "generic_name" {
            if let Some(ident) = name_node.child_by_field_name("name") {
                if let Ok(name) = ident.utf8_text(source.as_bytes()) {
                    return Some(name.trim().to_string());
                }
            }
        }
        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
            let clean = name.trim();
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }

    if lang_name == "CPP" {
        if let Some(decl) = node.child_by_field_name("declarator") {
            let mut cur = decl;
            while cur.kind() == "function_declarator"
                || cur.kind() == "pointer_declarator"
                || cur.kind() == "reference_declarator"
            {
                if let Some(inner) = cur.child_by_field_name("declarator") {
                    cur = inner;
                } else if let Some(first_child) = cur.named_child(0) {
                    cur = first_child;
                } else {
                    break;
                }
            }
            if let Ok(text) = cur.utf8_text(source.as_bytes()) {
                let clean = text.trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let k = child.kind();
        if k == "identifier" || k == "property_identifier" || k == "type_identifier" {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                let clean = text.trim();
                let keywords = [
                    "public",
                    "private",
                    "protected",
                    "internal",
                    "static",
                    "virtual",
                    "override",
                    "async",
                    "void",
                    "task",
                    "int",
                    "string",
                    "bool",
                    "extern",
                    "const",
                    "readonly",
                    "sealed",
                    "partial",
                    "abstract",
                    "class",
                    "struct",
                ];
                if !clean.is_empty() && !keywords.contains(&clean.to_lowercase().as_str()) {
                    return Some(clean.to_string());
                }
            }
        }
    }

    None
}

/// Extract calls from a node
pub fn extract_calls(node: &Node, source: &str) -> Vec<String> {
    let mut calls = Vec::new();
    walk_for_calls(node, source, &mut calls);
    let mut seen = std::collections::HashSet::new();
    calls.retain(|call| {
        if seen.contains(call) {
            false
        } else {
            seen.insert(call.clone());
            true
        }
    });
    calls
}

fn walk_for_calls(node: &Node, source: &str, calls: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "call_expression" | "invocation_expression" => {
                if let Some(func) = child
                    .child_by_field_name("function")
                    .or_else(|| child.child_by_field_name("expression"))
                {
                    if let Ok(name) = func.utf8_text(source.as_bytes()) {
                        calls.push(name.to_string());
                    }
                }
            }
            "method_call" | "method_invocation" => {
                if let Some(method) = child.child_by_field_name("method") {
                    if let Ok(name) = method.utf8_text(source.as_bytes()) {
                        calls.push(name.to_string());
                    }
                }
            }
            "scoped_identifier" | "qualified_identifier" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    if text.contains("::") {
                        if let Some(parent) = child.parent() {
                            if parent.kind() == "call_expression" {
                                calls.push(text.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        walk_for_calls(&child, source, calls);
    }
}

/// Extract doc comment from a node
pub fn extract_doc_comment(node: &Node, source: &str) -> Option<String> {
    let start = node.start_position().row;
    if start == 0 {
        return None;
    }

    let lines: Vec<&str> = source.lines().collect();
    let mut doc_lines = Vec::new();

    for line_num in (0..start).rev() {
        if let Some(line) = lines.get(line_num) {
            let trimmed = line.trim();
            if trimmed.starts_with("///")
                || trimmed.starts_with("//!")
                || trimmed.starts_with("/**")
                || trimmed.starts_with(" *")
            {
                doc_lines.push(
                    trimmed
                        .trim_start_matches("/// ")
                        .trim_start_matches("//! ")
                        .trim_start_matches(" * ")
                        .trim_start_matches("/**")
                        .trim_end_matches("*/"),
                );
            } else if !trimmed.is_empty() {
                break;
            }
        }
    }

    doc_lines.reverse();
    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

/// Extract decorators from a node
pub fn extract_decorators(node: &Node, source: &str) -> Vec<String> {
    let mut decorators = Vec::new();

    let start_byte = node.start_byte();
    let text_before = if start_byte > 0 && start_byte <= source.len() {
        &source[..start_byte]
    } else {
        return decorators;
    };

    let decorator_pattern = regex::Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_.]*)\s*(?:\()?").unwrap();

    for cap in decorator_pattern.captures_iter(text_before) {
        if let Some(matched) = cap.get(1) {
            decorators.push(matched.as_str().to_string());
        }
    }

    let mut child_cursor = node.walk();
    for child in node.children(&mut child_cursor) {
        if child.kind() == "decorator" || child.kind() == "attribute_list" {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                let cleaned = text.trim_matches(|c| c == '@' || c == '[' || c == ']');
                decorators.push(cleaned.to_string());
            }
        }
    }

    decorators
}

/// Check if a function is public (generic version)
pub fn is_public_generic(node: &Node, source: &str) -> bool {
    if let Ok(text) = node.utf8_text(source.as_bytes()) {
        if text.contains("pub ") || text.contains("public ") {
            return true;
        }
    }
    false
}

/// Check if a function is async
pub fn is_async(node: &Node, source: &str) -> bool {
    node.utf8_text(source.as_bytes())
        .map(|t| t.contains("async"))
        .unwrap_or(false)
}

/// Check if a function has test attribute
pub fn has_test_attribute(node: &Node, source: &str) -> bool {
    let start_byte = node.start_byte();
    let text_before = if start_byte > 0 && start_byte <= source.len() {
        &source[..start_byte]
    } else {
        return false;
    };

    let test_patterns = [
        "#[test]",
        "#[tokio::test]",
        "#[async_std::test]",
        "#[wasm_bindgen_test]",
        "#[test_case]",
        "#[bench]",
        "#[criterion]",
    ];

    for pattern in test_patterns {
        if text_before.contains(pattern) {
            return true;
        }
    }

    false
}

/// Check if a function is a trait method
pub fn is_trait_method(node: &Node, _source: &str) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        let kind = parent.kind();
        if kind == "trait_item" || kind == "trait_declaration" {
            return true;
        }
        current = parent.parent();
    }
    false
}

/// Check if a function is a trait default method
pub fn is_trait_default_method(node: &Node, source: &str) -> bool {
    if !is_trait_method(node, source) {
        return false;
    }
    node.child_by_field_name("body").is_some()
}

/// Infer role from function name
pub fn infer_role(name: &str) -> FunctionRole {
    let lower = name.to_lowercase();
    if lower.contains("main") || lower.contains("entry") {
        FunctionRole::EntryPoint
    } else if lower.contains("handler") || lower.contains("controller") {
        FunctionRole::Handler
    } else if lower.contains("service") || lower.contains("domain") {
        FunctionRole::Service
    } else if lower.contains("repo") || lower.contains("repository") || lower.contains("dao") {
        FunctionRole::Repository
    } else if lower.contains("util") || lower.contains("helper") {
        FunctionRole::Utility
    } else if lower.contains("validate") || lower.contains("check") {
        FunctionRole::Validator
    } else if lower.contains("factory") || lower.contains("create") || lower.contains("build") {
        FunctionRole::Factory
    } else if lower.contains("convert") || lower.contains("transform") || lower.contains("map") {
        FunctionRole::Converter
    } else if lower.contains("middleware") {
        FunctionRole::Middleware
    } else {
        FunctionRole::Unknown
    }
}

/// Infer purpose from function name
pub fn infer_purpose(name: &str, return_type: &Option<String>) -> String {
    let action = action_from_name(name);
    let subject = subject_from_name(name).unwrap_or("data");

    if let Some(ret) = return_type {
        format!("{} {} and returns {}", action, subject, ret)
    } else {
        format!("{} {}", action, subject)
    }
}

fn action_from_name(name: &str) -> &str {
    let lower = name.to_lowercase();
    if lower.starts_with("get") {
        "Gets"
    } else if lower.starts_with("set") {
        "Sets"
    } else if lower.starts_with("create") || lower.starts_with("build") {
        "Creates"
    } else if lower.starts_with("update") || lower.starts_with("modify") {
        "Updates"
    } else if lower.starts_with("delete") || lower.starts_with("remove") {
        "Deletes"
    } else if lower.starts_with("validate") {
        "Validates"
    } else if lower.starts_with("parse") {
        "Parses"
    } else if lower.starts_with("convert") || lower.starts_with("transform") {
        "Converts"
    } else if lower.starts_with("handle") {
        "Handles"
    } else if lower.starts_with("process") {
        "Processes"
    } else if lower.starts_with("init") || lower.starts_with("initialize") {
        "Initializes"
    } else {
        "Performs"
    }
}

fn subject_from_name(name: &str) -> Option<&str> {
    let parts: Vec<&str> = name.split('_').collect();
    if parts.len() >= 2 {
        Some(parts[1])
    } else {
        None
    }
}

/// Parse import statement
pub fn parse_import(text: &str) -> (String, Vec<String>) {
    let trimmed = text.trim();
    if trimmed.starts_with("use ") {
        let rest = &trimmed[4..].trim();
        if let Some(alias_pos) = rest.find(" as ") {
            let actual = &rest[..alias_pos];
            return (actual.to_string(), vec![actual.to_string()]);
        }
        return (rest.to_string(), vec![rest.to_string()]);
    }
    if trimmed.starts_with("import ") {
        let rest = &trimmed[7..].trim();
        return (rest.to_string(), vec![rest.to_string()]);
    }
    if trimmed.starts_with("using ") {
        let rest = trimmed
            .trim_start_matches("using ")
            .trim_end_matches(';')
            .trim();
        return (rest.to_string(), vec![rest.to_string()]);
    }
    (trimmed.to_string(), vec![])
}
