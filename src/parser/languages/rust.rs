// src/parser/languages/rust.rs

//! Rust language parser implementation

use crate::parser::languages::shared::{LanguageParserConfig, SharedParser};
use crate::parser::languages::LanguageConfig;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo};
use tree_sitter::{Node, Tree};

pub struct RustParser;

impl RustParser {
    pub fn config() -> LanguageConfig {
        LanguageConfig {
            name: "Rust".to_string(),
            extensions: vec!["rs".to_string()],
            language_fn: || tree_sitter_rust::LANGUAGE.into(),
            function_kinds: vec!["function_item".to_string(), "method_item".to_string()],
            import_kinds: vec!["use_declaration".to_string()],
            type_kinds: vec![
                "struct_item".to_string(),
                "enum_item".to_string(),
                "trait_item".to_string(),
                "impl_item".to_string(),
                "type_alias".to_string(),
            ],
        }
    }

    pub fn parser_config() -> LanguageParserConfig {
        LanguageParserConfig {
            name: "Rust",
            function_kinds: vec!["function_item", "method_item"],
            import_kinds: vec!["use_declaration"],
            type_kinds: vec![
                "struct_item",
                "enum_item",
                "trait_item",
                "impl_item",
                "type_alias",
            ],
            branch_kinds: vec![
                "if_expression",
                "if_let_expression",
                "match_arm", // each arm is a branch, not the match_expression itself
                "for_expression",
                "while_expression",
                "while_let_expression",
                "loop_expression",
                "try_expression", // the `?` operator
            ],
            has_attributes: false,
            go_export_rules: false,
            has_decorators: false,
            has_export_statements: false,
        }
    }

    pub fn extract_functions(tree: &Tree, source: &str) -> Vec<FunctionInfo> {
        let config = Self::parser_config();
        SharedParser::extract_functions(tree, source, &config)
    }

    pub fn extract_imports(tree: &Tree, source: &str) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();

        for node in root.children(&mut cursor) {
            if node.kind() != "use_declaration" {
                continue;
            }

            let line = node.start_position().row + 1;
            let mut use_nodes = Vec::new();
            Self::collect_use_nodes(&node, &mut use_nodes);

            for use_node in use_nodes {
                Self::flatten_use_tree(use_node, source, "", &mut imports, line);
            }
        }

        imports
    }

    pub fn extract_types(tree: &Tree, source: &str) -> Vec<TypeInfo> {
        let config = Self::parser_config();
        SharedParser::extract_types(tree, source, &config)
    }

    fn collect_use_nodes<'a>(node: &Node<'a>, out: &mut Vec<Node<'a>>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "use_clause" | "use_list" | "scoped_use_list" | "use_as_clause" | "identifier"
                | "self" | "super" | "crate" | "metavariable" => out.push(child),
                _ => Self::collect_use_nodes(&child, out),
            }
        }
    }

    fn flatten_use_tree(
        node: Node,
        source: &str,
        prefix: &str,
        imports: &mut Vec<ImportInfo>,
        line: usize,
    ) {
        match node.kind() {
            "identifier" | "self" | "super" | "crate" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    let segment = text.trim();
                    if segment.is_empty() {
                        return;
                    }
                    if prefix.is_empty() {
                        imports.push(ImportInfo {
                            module: segment.to_string(),
                            items: vec![segment.to_string()],
                            line,
                        });
                    } else {
                        imports.push(ImportInfo {
                            module: prefix.to_string(),
                            items: vec![segment.to_string()],
                            line,
                        });
                    }
                }
            }
            "use_as_clause" => {
                let mut cursor = node.walk();
                let named: Vec<_> = node.named_children(&mut cursor).collect();
                if named.is_empty() {
                    return;
                }

                let alias = named
                    .last()
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());

                let target = named[0];
                let target_text = target
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                if target_text.is_empty() {
                    return;
                }

                if target.kind() == "scoped_identifier" {
                    if let Some((module, item)) = target_text.rsplit_once("::") {
                        let mut items = Vec::new();
                        if let Some(alias) = alias {
                            items.push(alias);
                        }
                        items.push(item.trim().to_string());
                        imports.push(ImportInfo {
                            module: module.trim().to_string(),
                            items,
                            line,
                        });
                    }
                    return;
                }

                let mut items = Vec::new();
                if let Some(alias) = alias {
                    items.push(alias);
                }
                items.push(target_text.clone());
                imports.push(ImportInfo {
                    module: prefix.to_string(),
                    items,
                    line,
                });
            }
            "scoped_identifier" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    let text = text.trim();
                    if let Some((module, item)) = text.rsplit_once("::") {
                        imports.push(ImportInfo {
                            module: module.trim().to_string(),
                            items: vec![item.trim().to_string()],
                            line,
                        });
                    } else if !text.is_empty() {
                        imports.push(ImportInfo {
                            module: prefix.to_string(),
                            items: vec![text.to_string()],
                            line,
                        });
                    }
                }
            }
            "use_list" | "scoped_use_list" => {
                let new_prefix = if node.kind() == "scoped_use_list" {
                    let fallback_path = {
                        let mut cursor = node.walk();
                        let named_path = node
                            .named_children(&mut cursor)
                            .find(|child| {
                                matches!(
                                    child.kind(),
                                    "identifier" | "self" | "super" | "crate" | "scoped_identifier"
                                )
                            })
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.trim().to_string());
                        named_path
                    };

                    node.child_by_field_name("path")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .or(fallback_path)
                        .map(|path| {
                            if prefix.is_empty() {
                                path
                            } else {
                                format!("{}::{}", prefix, path)
                            }
                        })
                        .unwrap_or_else(|| prefix.to_string())
                } else {
                    prefix.to_string()
                };

                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    let is_path_node = child.kind() == "identifier"
                        || child.kind() == "self"
                        || child.kind() == "super"
                        || child.kind() == "crate"
                        || child.kind() == "scoped_identifier";

                    if node.kind() == "scoped_use_list" && is_path_node {
                        continue;
                    }

                    Self::flatten_use_tree(child, source, &new_prefix, imports, line);
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    Self::flatten_use_tree(child, source, prefix, imports, line);
                }
            }
        }
    }
}
