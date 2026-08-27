// src/parser/languages/shared.rs

//! Shared parser implementation for all languages
//!
//! This module provides a generic implementation of function/import/type extraction
//! that works for all languages, with language-specific configurations.

use crate::parser::languages::common::*;
use crate::parser::tree_sitter::{FunctionInfo, ImportInfo, TypeInfo, TypeKind, VariableInfo};
use tree_sitter::{Node, Tree};

/// Language-specific configuration for the shared parser
pub struct LanguageParserConfig {
    /// Name of the language
    pub name: &'static str,
    /// Function node kinds
    pub function_kinds: Vec<&'static str>,
    /// Import node kinds
    pub import_kinds: Vec<&'static str>,
    /// Type node kinds
    pub type_kinds: Vec<&'static str>,
    /// Branch node kinds
    pub branch_kinds: Vec<&'static str>,
    /// Whether to parse C#-style attributes
    pub has_attributes: bool,
    /// Whether to parse Go-style exported names
    pub go_export_rules: bool,
    /// Whether to parse Python-style decorators
    pub has_decorators: bool,
    /// Whether to parse JS/TS-style export statements
    pub has_export_statements: bool,
}

/// Shared parser implementation
pub struct SharedParser;

impl SharedParser {
    /// Extract functions from the AST using the provided configuration
    pub fn extract_functions(
        tree: &Tree,
        source: &str,
        config: &LanguageParserConfig,
    ) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let root = tree.root_node();
        Self::walk_for_functions(root, source, config, None, None, &mut functions);
        functions
    }

    /// Extract imports from the AST using the provided configuration
    pub fn extract_imports(
        tree: &Tree,
        source: &str,
        config: &LanguageParserConfig,
    ) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();

        for node in root.children(&mut cursor) {
            if config.import_kinds.contains(&node.kind()) {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    let (module, items) = parse_import(text);
                    imports.push(ImportInfo {
                        module,
                        items,
                        line: node.start_position().row + 1,
                    });
                }
            }
        }

        imports
    }

    /// Extract types from the AST using the provided configuration
    pub fn extract_types(
        tree: &Tree,
        source: &str,
        config: &LanguageParserConfig,
    ) -> Vec<TypeInfo> {
        let mut types = Vec::new();
        let root = tree.root_node();
        Self::walk_for_types(root, source, config, &mut types);
        types
    }

    /// Walk the AST and extract functions
    fn walk_for_functions(
        node: Node,
        source: &str,
        config: &LanguageParserConfig,
        container: Option<&str>,
        trait_impl: Option<&str>,
        out: &mut Vec<FunctionInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_kind = child.kind();

            // Check if this node is a function
            let is_function = config.function_kinds.contains(&child_kind);

            // Special case: variable declarator with function value (JS/TS)
            let is_js_function = child_kind == "variable_declarator" && {
                let name_is_identifier = child
                    .child_by_field_name("name")
                    .map(|n| n.kind() == "identifier")
                    .unwrap_or(false);
                let value_is_function = child
                    .child_by_field_name("value")
                    .map(|v| {
                        matches!(
                            v.kind(),
                            "arrow_function" | "function_expression" | "function"
                        )
                    })
                    .unwrap_or(false);
                name_is_identifier && value_is_function
            };

            if is_function || is_js_function {
                if let Some(func) =
                    Self::parse_function(&child, source, config, container, trait_impl)
                {
                    out.push(func);
                }
            }

            // Track container (class/struct/trait) for method resolution
            let mut next_container = container;
            if Self::is_type_container(&child_kind) {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Ok(cname) = name_node.utf8_text(source.as_bytes()) {
                        next_container = Some(cname.trim());
                    }
                }
            }

            // Track trait implementation (Rust-specific)
            let mut next_trait = trait_impl;
            if child_kind == "impl_item" && config.name == "Rust" {
                // Try field first (impl Trait for Type)
                let ty = child
                    .child_by_field_name("type")
                    .and_then(|t| t.utf8_text(source.as_bytes()).ok());

                // If no type field, extract from the impl text
                let ty = if ty.is_some() {
                    ty
                } else {
                    // Get the full text of the impl_item
                    child
                        .utf8_text(source.as_bytes())
                        .ok()
                        .and_then(|text| {
                            let trimmed = text.trim_start();
                            if trimmed.starts_with("impl ") {
                                let after_impl = &trimmed[5..];
                                // Handle "impl Trait for Type"
                                if let Some(for_pos) = after_impl.find(" for ") {
                                    after_impl[for_pos + 5..]
                                        .split_whitespace()
                                        .next()
                                        .map(|s| s.trim_matches('{').trim().to_string())
                                } else {
                                    // "impl TypeName {"
                                    after_impl
                                        .split_whitespace()
                                        .next()
                                        .map(|s| s.trim_matches('{').trim().to_string())
                                }
                            } else {
                                None
                            }
                        })
                        .and_then(|s| {
                            // Need to store this String somewhere that outlives the closure
                            // We'll leak it temporarily - acceptable for parsing
                            Some(&*Box::leak(s.into_boxed_str()))
                        })
                };

                let tr = child
                    .child_by_field_name("trait")
                    .and_then(|t| t.utf8_text(source.as_bytes()).ok());

                next_container = ty;
                next_trait = tr;
            }

            Self::walk_for_functions(child, source, config, next_container, next_trait, out);
        }
    }
    /// Parse a single function node
    fn parse_function(
        node: &Node,
        source: &str,
        config: &LanguageParserConfig,
        container: Option<&str>,
        trait_impl: Option<&str>,
    ) -> Option<FunctionInfo> {
        let name = extract_function_name(node, source, config.name)?;
        let line = node.start_position().row + 1;

        let is_public = if config.go_export_rules {
            // Go: capitalized = public
            name.chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
        } else if config.has_export_statements {
            // JS/TS: check for export statements
            is_public_js(node, source)
        } else {
            // Check for pub/ public keyword
            is_public_generic(node, source, config.name)
        };

        let is_async = is_async(node, source);

        let resolved_container = container.map(|s| s.to_string());

        let return_type = node
            .child_by_field_name("return_type")
            .or_else(|| node.child_by_field_name("type"))
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());

        let params = node
            .child_by_field_name("parameters")
            .or_else(|| node.child_by_field_name("parameter_list"))
            .map(|p| parse_parameters(&p, source))
            .unwrap_or_default();

        let doc_comment = extract_doc_comment(node, source);

        let body_node = node
            .child_by_field_name("body")
            .or_else(|| node.child_by_field_name("arrow_expression_clause"))
            .or_else(|| node.child_by_field_name("expression"));

        let calls = body_node
            .map(|body| extract_calls(&body, source))
            .unwrap_or_default();

        let (body_start, body_end) = if let Some(body) = body_node {
            (body.start_byte(), body.end_byte())
        } else {
            (node.start_byte(), node.end_byte())
        };

        let complexity = 1 + body_node
            .map(|body| Self::count_branches(body, source, &config.branch_kinds))
            .unwrap_or(0);
        let role = infer_role(&name);
        let purpose = infer_purpose(&name, &return_type);

        let decorators = if config.has_decorators || config.has_attributes {
            extract_decorators(node, source)
        } else {
            Vec::new()
        };

        let is_test = has_test_attribute(node, source);
        let is_trait_method = is_trait_method(node, source);
        let is_trait_default = is_trait_default_method(node, source);
        let variables = body_node
            .map(|body| Self::extract_variables(&body, source))
            .unwrap_or_default();

        Some(FunctionInfo {
            name,
            line,
            is_public,
            is_async,
            params,
            return_type,
            doc_comment,
            calls,
            body_range: (body_start, body_end),
            body_start_line: line,
            body_end_line: node.end_position().row + 1,
            complexity,
            container: resolved_container,
            role,
            purpose,
            trait_impl: trait_impl.map(|s| s.to_string()),
            decorators,
            is_test,
            is_trait_method,
            is_trait_default,
            variables,
        })
    }

    /// Count real decision-point nodes under a function body (McCabe-style: 1 + branches)
    fn count_branches(node: Node, source: &str, branch_kinds: &[&str]) -> usize {
        let mut count = if branch_kinds.contains(&node.kind()) {
            1
        } else {
            0
        };

        // && / || share the generic "binary_expression" kind with everything else,
        // so they need a text check on the operator field rather than a kind match
        if node.kind() == "binary_expression" {
            if let Some(op) = node.child_by_field_name("operator") {
                if let Ok(text) = op.utf8_text(source.as_bytes()) {
                    if text == "&&" || text == "||" {
                        count += 1;
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            count += Self::count_branches(child, source, branch_kinds);
        }
        count
    }

    fn extract_variables(node: &Node, source: &str) -> Vec<VariableInfo> {
        let mut variables = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "let_declaration" || child.kind() == "variable_declaration" {
                let name = child
                    .child_by_field_name("pattern")
                    .and_then(|p| p.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("unknown")
                    .to_string();

                let type_hint = child
                    .child_by_field_name("type")
                    .and_then(|t| t.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());

                let initializer = child
                    .child_by_field_name("value")
                    .and_then(|v| v.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());

                variables.push(VariableInfo {
                    name,
                    type_hint,
                    initializer,
                });
            }

            // Recurse
            variables.extend(Self::extract_variables(&child, source));
        }

        variables
    }

    /// Walk the AST and extract types
    fn walk_for_types(
        node: Node,
        source: &str,
        config: &LanguageParserConfig,
        out: &mut Vec<TypeInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if config.type_kinds.contains(&child.kind()) {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                        let kind = Self::node_kind_to_type_kind(&child);
                        out.push(TypeInfo {
                            name: name.to_string(),
                            kind,
                            line: child.start_position().row + 1,
                        });
                    }
                }
            }
            Self::walk_for_types(child, source, config, out);
        }
    }

    /// Convert tree-sitter node kind to TypeKind
    fn node_kind_to_type_kind(node: &Node) -> TypeKind {
        match node.kind() {
            "struct_item" | "struct_declaration" | "struct_specifier" => TypeKind::Struct,
            "enum_item" | "enum_declaration" | "enum_specifier" => TypeKind::Enum,
            "trait_item" | "trait_declaration" => TypeKind::Trait,
            "impl_item" => TypeKind::Impl,
            "type_alias" | "type_alias_declaration" => TypeKind::TypeAlias,
            "interface_declaration" => TypeKind::Interface,
            "class_declaration" | "class_definition" | "class_specifier" => TypeKind::Class,
            _ => TypeKind::Struct,
        }
    }

    /// Check if a node kind is a type container (class/struct/trait)
    fn is_type_container(kind: &str) -> bool {
        matches!(
            kind,
            "class_declaration"
                | "class_definition"
                | "struct_declaration"
                | "struct_specifier"
                | "interface_declaration"
                | "trait_declaration"
                | "enum_declaration"
                | "enum_specifier"
                | "record_declaration"
                | "record_struct_declaration"
                | "class_specifier"
        )
    }
}

/// Check if a JS/TS function is public (has export)
fn is_public_js(node: &Node, source: &str) -> bool {
    let mut curr = Some(*node);
    while let Some(n) = curr {
        let kind = n.kind();
        if kind == "export_statement" || kind == "export_default" || kind == "export_declaration" {
            return true;
        }
        if let Ok(text) = n.utf8_text(source.as_bytes()) {
            let trimmed = text.trim_start();
            if trimmed.starts_with("export ") || trimmed.starts_with("export default ") {
                return true;
            }
        }
        curr = n.parent();
    }
    false
}

/// Generic public check for most languages
fn is_public_generic(node: &Node, source: &str, lang_name: &str) -> bool {
    if let Ok(text) = node.utf8_text(source.as_bytes()) {
        if text.contains("pub ") || text.contains("public ") {
            return true;
        }
    }

    // Python: not starting with _ (except __init__)
    if lang_name == "Python" {
        if let Some(name) = node.child_by_field_name("name") {
            if let Ok(name_text) = name.utf8_text(source.as_bytes()) {
                return !name_text.starts_with('_') || name_text.starts_with("__init__");
            }
        }
    }

    // Dart: not starting with _
    if lang_name == "Dart" {
        if let Some(name) = node.child_by_field_name("name") {
            if let Ok(name_text) = name.utf8_text(source.as_bytes()) {
                return !name_text.starts_with('_');
            }
        }
    }

    // PHP: public by default unless private/protected
    if lang_name == "PHP" {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            return !text.contains("private ") && !text.contains("protected ");
        }
    }

    // C++: extern/export or not static
    if lang_name == "CPP" {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            return text.contains("extern ")
                || text.contains("export ")
                || !text.contains("static ");
        }
    }

    false
}
