use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Node, Parser, Tree};

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: String,
    pub language: String,
    pub functions: Vec<FunctionInfo>,
    pub imports: Vec<ImportInfo>,
    pub types: Vec<TypeInfo>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub params: Vec<ParamInfo>,
    pub return_type: Option<String>,
    pub doc_comment: Option<String>,
    pub calls: Vec<String>,
    pub body_range: (usize, usize),
    pub container: Option<String>,
    pub role: FunctionRole,
    pub purpose: String,
    pub trait_impl: Option<String>,
    pub decorators: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionRole {
    EntryPoint,
    Handler,
    Service,
    Repository,
    Utility,
    Validator,
    Factory,
    Converter,
    Middleware,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub type_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub module: String,
    pub items: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
    Impl,
    TypeAlias,
    Interface,
    Class,
}

pub struct TreeSitterParser {
    languages: HashMap<String, LanguageConfig>,
}

struct LanguageConfig {
    name: String,
    #[allow(dead_code)]
    extensions: Vec<String>,
    language_fn: fn() -> Language,
    function_kinds: Vec<String>,
    import_kinds: Vec<String>,
    type_kinds: Vec<String>,
}

impl TreeSitterParser {
    pub fn new() -> Self {
        Self {
            languages: Self::configure_languages(),
        }
    }

    fn configure_languages() -> HashMap<String, LanguageConfig> {
        let mut langs = HashMap::new();

        // Rust
        langs.insert(
            "rs".to_string(),
            LanguageConfig {
                name: "Rust".to_string(),
                extensions: vec!["rs".to_string()],
                language_fn: tree_sitter_rust::language,
                function_kinds: vec!["function_item".to_string(), "method_item".to_string()],
                import_kinds: vec!["use_declaration".to_string()],
                type_kinds: vec![
                    "struct_item".to_string(),
                    "enum_item".to_string(),
                    "trait_item".to_string(),
                    "impl_item".to_string(),
                    "type_alias".to_string(),
                ],
            },
        );

        // Python
        langs.insert(
            "py".to_string(),
            LanguageConfig {
                name: "Python".to_string(),
                extensions: vec!["py".to_string()],
                language_fn: tree_sitter_python::language,
                function_kinds: vec![
                    "function_definition".to_string(),
                    "async_function_definition".to_string(),
                    "method_definition".to_string(),
                ],
                import_kinds: vec![
                    "import_statement".to_string(),
                    "import_from_statement".to_string(),
                ],
                type_kinds: vec!["class_definition".to_string()],
            },
        );

        // JavaScript (JS + JSX)
        langs.insert(
            "js".to_string(),
            LanguageConfig {
                name: "JavaScript".to_string(),
                extensions: vec!["js".to_string(), "jsx".to_string()],
                language_fn: tree_sitter_javascript::language,
                function_kinds: vec![
                    "function_declaration".to_string(),
                    "function_expression".to_string(),
                    "arrow_function".to_string(),
                    "method_definition".to_string(),
                    "generator_function_declaration".to_string(),
                ],
                import_kinds: vec![
                    "import_statement".to_string(),
                    "export_statement".to_string(),
                ],
                type_kinds: vec!["class_declaration".to_string()],
            },
        );

        // TypeScript (TS) - without JSX
        langs.insert(
            "ts".to_string(),
            LanguageConfig {
                name: "TypeScript".to_string(),
                extensions: vec!["ts".to_string()],
                language_fn: tree_sitter_typescript::language_typescript,
                function_kinds: vec![
                    "function_declaration".to_string(),
                    "function_expression".to_string(),
                    "arrow_function".to_string(),
                    "method_definition".to_string(),
                    "generator_function_declaration".to_string(),
                    "function".to_string(),
                    "lexical_declaration".to_string(),
                ],
                import_kinds: vec![
                    "import_statement".to_string(),
                    "import".to_string(),
                    "export_statement".to_string(),
                    "export".to_string(),
                ],
                type_kinds: vec![
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "type_alias_declaration".to_string(),
                    "enum_declaration".to_string(),
                    "type_parameter".to_string(),
                ],
            },
        );

        // TypeScript with JSX (TSX) - ⭐ NEW
        langs.insert(
            "tsx".to_string(),
            LanguageConfig {
                name: "TypeScript".to_string(),
                extensions: vec!["tsx".to_string()],
                language_fn: tree_sitter_typescript::language_tsx,
                function_kinds: vec![
                    // Standard function declarations
                    "function_declaration".to_string(),
                    "function_expression".to_string(),
                    "arrow_function".to_string(),
                    "method_definition".to_string(),
                    "generator_function_declaration".to_string(),
                    // ⭐ CRITICAL: React component patterns
                    "function".to_string(),
                    "lexical_declaration".to_string(),
                    "variable_declaration".to_string(),
                    "variable_declarator".to_string(),
                    "export_statement".to_string(),
                    "export_default".to_string(),
                    // Class components
                    "class_declaration".to_string(),
                    "class".to_string(),
                    "method_definition".to_string(),
                ],
                import_kinds: vec![
                    "import_statement".to_string(),
                    "import".to_string(),
                    "export_statement".to_string(),
                    "export".to_string(),
                ],
                type_kinds: vec![
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "type_alias_declaration".to_string(),
                    "enum_declaration".to_string(),
                    "type_parameter".to_string(),
                ],
            },
        );

        // Go
        langs.insert(
            "go".to_string(),
            LanguageConfig {
                name: "Go".to_string(),
                extensions: vec!["go".to_string()],
                language_fn: tree_sitter_go::language,
                function_kinds: vec![
                    "function_declaration".to_string(),
                    "method_declaration".to_string(),
                ],
                import_kinds: vec!["import_declaration".to_string()],
                type_kinds: vec!["type_declaration".to_string()],
            },
        );

        // Java
        langs.insert(
            "java".to_string(),
            LanguageConfig {
                name: "Java".to_string(),
                extensions: vec!["java".to_string()],
                language_fn: tree_sitter_java::language,
                function_kinds: vec![
                    "method_declaration".to_string(),
                    "constructor_declaration".to_string(),
                    "lambda_expression".to_string(),
                ],
                import_kinds: vec!["import_declaration".to_string()],
                type_kinds: vec![
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "enum_declaration".to_string(),
                    "record_declaration".to_string(),
                ],
            },
        );

        langs
    }

    fn detect_language(&self, path: &Path) -> Option<&LanguageConfig> {
        let ext = path.extension()?.to_str()?;
        self.languages.get(ext)
    }

    fn extract_decorators(node: &Node, source: &str) -> Vec<String> {
        let mut decorators = Vec::new();

        let start_byte = node.start_byte();
        let text_before = if start_byte > 0 && start_byte <= source.len() {
            &source[..start_byte]
        } else {
            ""
        };

        let decorator_pattern = regex::Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_.]*)\s*(?:\()?").unwrap();

        for cap in decorator_pattern.captures_iter(text_before) {
            if let Some(matched) = cap.get(1) {
                decorators.push(matched.as_str().to_string());
            }
        }

        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            if child.kind() == "decorator" {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    decorators.push(text.trim_start_matches('@').to_string());
                }
            }
        }

        decorators
    }

    pub fn parse_file(&self, path: &Path) -> Result<ParsedFile, String> {
        let config = self
            .detect_language(path)
            .ok_or_else(|| format!("Unsupported file: {:?}", path))?;

        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

        let mut parser = Parser::new();
        let language = (config.language_fn)();
        parser
            .set_language(&language)
            .map_err(|e| format!("Failed to set language: {}", e))?;

        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| "Failed to parse".to_string())?;

        let functions = Self::extract_functions(&tree, &source, config);
        let imports = Self::extract_imports(&tree, &source, config);
        let types = Self::extract_types(&tree, &source, config);

        Ok(ParsedFile {
            path: path.to_string_lossy().to_string(),
            language: config.name.clone(),
            functions,
            imports,
            types,
            source,
        })
    }

    fn extract_functions(tree: &Tree, source: &str, config: &LanguageConfig) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let root = tree.root_node();
        Self::walk_for_functions(root, source, config, None, None, &mut functions);
        functions
    }

    fn walk_for_functions(
        node: Node,
        source: &str,
        config: &LanguageConfig,
        container: Option<&str>,
        trait_impl: Option<&str>,
        out: &mut Vec<FunctionInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if config.function_kinds.contains(&child.kind().to_string()) {
                if let Some(func) = Self::parse_function(&child, source, container, trait_impl) {
                    out.push(func);
                }
            }
            let (next_container, next_trait) = if child.kind() == "impl_item" {
                let ty = child
                    .child_by_field_name("type")
                    .and_then(|t| t.utf8_text(source.as_bytes()).ok());
                let tr = child
                    .child_by_field_name("trait")
                    .and_then(|t| t.utf8_text(source.as_bytes()).ok());
                (ty, tr)
            } else {
                (container, trait_impl)
            };
            Self::walk_for_functions(child, source, config, next_container, next_trait, out);
        }
    }

    fn parse_function(
        node: &Node,
        source: &str,
        container: Option<&str>,
        trait_impl: Option<&str>,
    ) -> Option<FunctionInfo> {
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?
            .to_string();

        let line = node.start_position().row + 1;
        let is_public = Self::is_public(node, source);
        let is_async = Self::is_async(node, source);
        let return_type = node
            .child_by_field_name("return_type")
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());

        let params = node
            .child_by_field_name("parameters")
            .map(|p| Self::parse_parameters(&p, source))
            .unwrap_or_default();

        let doc_comment = Self::extract_doc_comment(node, source);

        let calls = node
            .child_by_field_name("body")
            .map(|body| Self::extract_calls(&body, source))
            .unwrap_or_default();

        let role = Self::infer_role(&name, &params);
        let purpose = Self::infer_purpose(&name, &params, &return_type);

        let decorators = Self::extract_decorators(node, source);

        Some(FunctionInfo {
            name,
            line,
            is_public,
            is_async,
            params,
            return_type,
            doc_comment,
            calls,
            body_range: (node.start_byte(), node.end_byte()),
            container: container.map(|s| s.to_string()),
            role,
            purpose,
            trait_impl: trait_impl.map(|s| s.to_string()),
            decorators,
        })
    }

    fn parse_parameters(node: &Node, source: &str) -> Vec<ParamInfo> {
        let mut params = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "parameter" || child.kind() == "typed_parameter" {
                let name = child
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("unknown")
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

    // Add this function after extract_calls()

    fn extract_calls(node: &Node, source: &str) -> Vec<String> {
        let mut calls = Vec::new();
        Self::walk_for_calls_with_context(node, source, &mut calls, None);
        Self::walk_for_jsx_components(node, source, &mut calls);

        calls
    }

    // ⭐ NEW: Walk the AST for JSX components
    fn walk_for_jsx_components(node: &Node, source: &str, calls: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Check for JSX element (self-closing or with children)
            if child.kind() == "jsx_element" || child.kind() == "jsx_self_closing_element" {
                if let Some(open_tag) = child.child_by_field_name("open_tag") {
                    if let Some(tag_name) = open_tag.child_by_field_name("name") {
                        if let Ok(name) = tag_name.utf8_text(source.as_bytes()) {
                            if name
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                            {
                                calls.push(format!("jsx::{}", name));
                            }
                        }
                    }
                }
            }
            Self::walk_for_jsx_components(&child, source, calls);
        }
    }

    fn walk_for_calls_with_context(
        node: &Node,
        source: &str,
        calls: &mut Vec<String>,
        receiver_type: Option<&str>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                // CALL EXPRESSION: function_name() or self.method() or Type::method()
                "call_expression" => {
                    if let Some(func) = child.child_by_field_name("function") {
                        // Case 1: Field expression — x.method() or self.method()
                        if func.kind() == "field_expression" {
                            let receiver = func
                                .child_by_field_name("value")
                                .and_then(|v| v.utf8_text(source.as_bytes()).ok())
                                .unwrap_or("")
                                .trim();

                            if let Some(method) = func.child_by_field_name("field") {
                                if let Ok(method_name) = method.utf8_text(source.as_bytes()) {
                                    if receiver == "self" {
                                        calls.push(format!("self::{}", method_name));
                                    } else if receiver == "Self" {
                                        calls.push(format!("Self::{}", method_name));
                                    } else if receiver.contains("::") {
                                        calls.push(format!("{}::{}", receiver, method_name));
                                    } else {
                                        calls.push(format!("{}.{}", receiver, method_name));
                                    }
                                }
                            }
                        }
                        // Case 2: Scoped/qualified identifier — Type::function()
                        else if func.kind() == "scoped_identifier"
                            || func.kind() == "qualified_identifier"
                        {
                            if let Ok(name) = func.utf8_text(source.as_bytes()) {
                                calls.push(name.to_string());
                            }
                        }
                        // Case 3: Plain call — function_name()
                        else if let Ok(name) = func.utf8_text(source.as_bytes()) {
                            calls.push(name.to_string());
                        }
                    }
                }
                // METHOD CALL: Generic method call detection
                "method_call" | "method_invocation" => {
                    if let Some(receiver) = child.child_by_field_name("receiver") {
                        let receiver_text = receiver
                            .utf8_text(source.as_bytes())
                            .unwrap_or("self")
                            .trim();

                        if let Some(method) = child.child_by_field_name("method") {
                            if let Ok(method_name) = method.utf8_text(source.as_bytes()) {
                                if receiver_text == "self" {
                                    calls.push(format!("self::{}", method_name));
                                } else if receiver_text == "Self" {
                                    calls.push(format!("Self::{}", method_name));
                                } else if receiver_text.contains("::") {
                                    calls.push(format!("{}::{}", receiver_text, method_name));
                                } else {
                                    calls.push(format!("{}.{}", receiver_text, method_name));
                                }
                            }
                        }
                    }
                }
                // CHAIN EXPRESSION: x.y().z()
                "chain_expression" | "chained_call" => {
                    Self::walk_for_calls_with_context(&child, source, calls, receiver_type);
                }
                // INDEX EXPRESSION: a[b] → op::index
                "index_expression" => {
                    calls.push("op::index".to_string());
                }
                // BINARY EXPRESSION: a + b → op::add
                "binary_expression" => {
                    if let Some(op_node) = child.child_by_field_name("operator") {
                        if let Ok(op) = op_node.utf8_text(source.as_bytes()) {
                            let trait_method = match op {
                                "+" => Some("add"),
                                "-" => Some("sub"),
                                "*" => Some("mul"),
                                "/" => Some("div"),
                                "%" => Some("rem"),
                                _ => None,
                            };
                            if let Some(m) = trait_method {
                                calls.push(format!("op::{}", m));
                            }
                        }
                    }
                }
                // SCOPED IDENTIFIER: Type::method (standalone)
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
                // FIELD EXPRESSION: self.method (standalone)
                "field_expression" => {
                    if let (Some(value), Some(field)) = (
                        child.child_by_field_name("value"),
                        child.child_by_field_name("field"),
                    ) {
                        if let (Ok(receiver), Ok(method_name)) = (
                            value.utf8_text(source.as_bytes()),
                            field.utf8_text(source.as_bytes()),
                        ) {
                            let receiver = receiver.trim();
                            if receiver == "self" {
                                calls.push(format!("self::{}", method_name));
                            } else if receiver == "Self" {
                                calls.push(format!("Self::{}", method_name));
                            } else if receiver.contains("::") {
                                calls.push(format!("{}::{}", receiver, method_name));
                            } else {
                                calls.push(format!("{}.{}", receiver, method_name));
                            }
                        }
                    }
                }
                _ => {}
            }
            Self::walk_for_calls_with_context(&child, source, calls, receiver_type);
        }
    }

    fn extract_imports(tree: &Tree, source: &str, config: &LanguageConfig) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();

        for node in root.children(&mut cursor) {
            if config.import_kinds.contains(&node.kind().to_string()) {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    let (module, items) = Self::parse_import(text);
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

    fn parse_import(text: &str) -> (String, Vec<String>) {
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
        (trimmed.to_string(), vec![])
    }

    fn extract_types(tree: &Tree, source: &str, config: &LanguageConfig) -> Vec<TypeInfo> {
        let mut types = Vec::new();
        let root = tree.root_node();
        Self::walk_for_types(root, source, config, &mut types);
        types
    }

    fn walk_for_types(node: Node, source: &str, config: &LanguageConfig, out: &mut Vec<TypeInfo>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if config.type_kinds.contains(&child.kind().to_string()) {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                        let kind = match child.kind() {
                            "struct_item" => TypeKind::Struct,
                            "enum_item" => TypeKind::Enum,
                            "trait_item" => TypeKind::Trait,
                            "impl_item" => TypeKind::Impl,
                            "class_definition" | "class_declaration" => TypeKind::Class,
                            "interface_declaration" => TypeKind::Interface,
                            _ => TypeKind::Struct,
                        };
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

    fn infer_role(name: &str, _params: &[ParamInfo]) -> FunctionRole {
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
        } else if lower.contains("convert") || lower.contains("transform") || lower.contains("map")
        {
            FunctionRole::Converter
        } else if lower.contains("middleware") {
            FunctionRole::Middleware
        } else {
            FunctionRole::Unknown
        }
    }

    fn infer_purpose(name: &str, _params: &[ParamInfo], return_type: &Option<String>) -> String {
        let action = Self::action_from_name(name);
        let subject = Self::subject_from_name(name).unwrap_or("data");

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

    fn is_public(node: &Node, source: &str) -> bool {
        node.utf8_text(source.as_bytes())
            .map(|t| t.contains("pub "))
            .unwrap_or(false)
    }

    fn is_async(node: &Node, source: &str) -> bool {
        node.utf8_text(source.as_bytes())
            .map(|t| t.contains("async"))
            .unwrap_or(false)
    }

    fn extract_doc_comment(node: &Node, source: &str) -> Option<String> {
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
}
