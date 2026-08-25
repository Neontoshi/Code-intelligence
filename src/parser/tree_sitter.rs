use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

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
    pub body_start_line: usize,
    pub body_end_line: usize,
    pub container: Option<String>,
    pub role: FunctionRole,
    pub purpose: String,
    pub trait_impl: Option<String>,
    pub decorators: Vec<String>,
    pub is_test: bool,
    pub is_trait_method: bool,
    pub is_trait_default: bool,
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

#[derive(Clone)]
struct LanguageConfig {
    name: String,
    #[allow(dead_code)]
    extensions: Vec<String>,
    language_fn: fn() -> tree_sitter::Language,
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
            },
        );

        // Python
        langs.insert(
            "py".to_string(),
            LanguageConfig {
                name: "Python".to_string(),
                extensions: vec!["py".to_string()],
                language_fn: || tree_sitter_python::LANGUAGE.into(),
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
                language_fn: || tree_sitter_javascript::LANGUAGE.into(),
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

        // TypeScript (TS)
        langs.insert(
            "ts".to_string(),
            LanguageConfig {
                name: "TypeScript".to_string(),
                extensions: vec!["ts".to_string()],
                language_fn: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
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

        // TypeScript with JSX (TSX)
        langs.insert(
            "tsx".to_string(),
            LanguageConfig {
                name: "TypeScript".to_string(),
                extensions: vec!["tsx".to_string()],
                language_fn: || tree_sitter_typescript::LANGUAGE_TSX.into(),
                function_kinds: vec![
                    "function_declaration".to_string(),
                    "function_expression".to_string(),
                    "arrow_function".to_string(),
                    "method_definition".to_string(),
                    "generator_function_declaration".to_string(),
                    "function".to_string(),
                    "lexical_declaration".to_string(),
                    "variable_declaration".to_string(),
                    "variable_declarator".to_string(),
                    "export_statement".to_string(),
                    "export_default".to_string(),
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
                language_fn: || tree_sitter_go::LANGUAGE.into(),
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
                language_fn: || tree_sitter_java::LANGUAGE.into(),
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

        // Dart / Flutter
        langs.insert(
            "dart".to_string(),
            LanguageConfig {
                name: "Dart".to_string(),
                extensions: vec!["dart".to_string()],
                language_fn: || tree_sitter_dart::LANGUAGE.into(),
                function_kinds: vec![
                    "function_signature".to_string(),
                    "method_signature".to_string(),
                    "function_declaration".to_string(),
                    "method_declaration".to_string(),
                    "getter_signature".to_string(),
                    "setter_signature".to_string(),
                ],
                import_kinds: vec!["import_or_export".to_string(), "library_import".to_string()],
                type_kinds: vec![
                    "class_definition".to_string(),
                    "mixin_declaration".to_string(),
                    "extension_declaration".to_string(),
                    "enum_declaration".to_string(),
                ],
            },
        );

        // PHP
        langs.insert(
            "php".to_string(),
            LanguageConfig {
                name: "PHP".to_string(),
                extensions: vec!["php".to_string()],
                language_fn: || tree_sitter_php::LANGUAGE_PHP.into(),
                function_kinds: vec![
                    "function_definition".to_string(),
                    "method_declaration".to_string(),
                    "arrow_function".to_string(),
                    "anonymous_function_creation_expression".to_string(),
                ],
                import_kinds: vec![
                    "namespace_use_declaration".to_string(),
                    "require_expression".to_string(),
                    "include_expression".to_string(),
                ],
                type_kinds: vec![
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "trait_declaration".to_string(),
                    "enum_declaration".to_string(),
                ],
            },
        );

        // C++ (cpp, cc, cxx, hpp, h)
        let cpp_config = LanguageConfig {
            name: "CPP".to_string(),
            extensions: vec![
                "cpp".to_string(),
                "cc".to_string(),
                "cxx".to_string(),
                "hpp".to_string(),
                "h".to_string(),
            ],
            language_fn: || tree_sitter_cpp::LANGUAGE.into(),
            function_kinds: vec![
                "function_definition".to_string(),
                "declaration".to_string(),
                "template_declaration".to_string(),
                "field_declaration".to_string(),
            ],
            import_kinds: vec![
                "preproc_include".to_string(),
                "using_declaration".to_string(),
            ],
            type_kinds: vec![
                "class_specifier".to_string(),
                "struct_specifier".to_string(),
                "enum_specifier".to_string(),
                "type_definition".to_string(),
            ],
        };

        langs.insert("cpp".to_string(), cpp_config);
        langs.insert(
            "cc".to_string(),
            LanguageConfig {
                extensions: vec!["cc".to_string()],
                ..langs.get("cpp").unwrap().clone()
            },
        );
        langs.insert(
            "cxx".to_string(),
            LanguageConfig {
                extensions: vec!["cxx".to_string()],
                ..langs.get("cpp").unwrap().clone()
            },
        );
        langs.insert(
            "hpp".to_string(),
            LanguageConfig {
                extensions: vec!["hpp".to_string()],
                ..langs.get("cpp").unwrap().clone()
            },
        );
        langs.insert(
            "h".to_string(),
            LanguageConfig {
                extensions: vec!["h".to_string()],
                ..langs.get("cpp").unwrap().clone()
            },
        );

        // C# / .NET
        langs.insert(
            "cs".to_string(),
            LanguageConfig {
                name: "CSharp".to_string(),
                extensions: vec!["cs".to_string()],
                language_fn: || tree_sitter_c_sharp::LANGUAGE.into(),
                function_kinds: vec![
                    "method_declaration".to_string(),
                    "constructor_declaration".to_string(),
                    "destructor_declaration".to_string(),
                    "local_function_statement".to_string(),
                    "operator_declaration".to_string(),
                    "conversion_operator_declaration".to_string(),
                    "property_declaration".to_string(),
                    "indexer_declaration".to_string(),
                    "arrow_expression_clause".to_string(),
                ],
                import_kinds: vec!["using_directive".to_string()],
                type_kinds: vec![
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "struct_declaration".to_string(),
                    "enum_declaration".to_string(),
                    "record_declaration".to_string(),
                    "record_struct_declaration".to_string(),
                    "namespace_declaration".to_string(),
                    "file_scoped_namespace_declaration".to_string(),
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
            if child.kind() == "decorator" || child.kind() == "attribute_list" {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    let cleaned = text.trim_matches(|c| c == '@' || c == '[' || c == ']');
                    decorators.push(cleaned.to_string());
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

        let (functions, imports, types) = if config.name == "CSharp" {
            Self::extract_csharp(&tree, &source)
        } else {
            let funcs = Self::extract_functions(&tree, &source, config);
            let imps = Self::extract_imports(&tree, &source, config);
            let typs = Self::extract_types(&tree, &source, config);
            (funcs, imps, typs)
        };

        Ok(ParsedFile {
            path: path.to_string_lossy().to_string(),
            language: config.name.clone(),
            functions,
            imports,
            types,
            source,
        })
    }

    // ============================================================
    // Dedicated C# AST Extractor
    // ============================================================

    fn extract_csharp(
        tree: &Tree,
        source: &str,
    ) -> (Vec<FunctionInfo>, Vec<ImportInfo>, Vec<TypeInfo>) {
        let mut functions = Vec::new();
        let mut imports = Vec::new();
        let mut types = Vec::new();
        let root = tree.root_node();

        Self::walk_csharp_node(
            root,
            source,
            None,
            None,
            &mut functions,
            &mut imports,
            &mut types,
        );
        (functions, imports, types)
    }

    fn walk_csharp_node(
        node: Node,
        source: &str,
        current_namespace: Option<&str>,
        current_type: Option<&str>,
        functions: &mut Vec<FunctionInfo>,
        imports: &mut Vec<ImportInfo>,
        types: &mut Vec<TypeInfo>,
    ) {
        let kind = node.kind();
        let mut next_namespace = current_namespace;
        let mut next_type = current_type;

        match kind {
            "using_directive" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    let (module, items) = Self::parse_import(text);
                    imports.push(ImportInfo {
                        module,
                        items,
                        line: node.start_position().row + 1,
                    });
                }
            }
            "namespace_declaration" | "file_scoped_namespace_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(ns_text) = name_node.utf8_text(source.as_bytes()) {
                        next_namespace = Some(ns_text.trim());
                    }
                }
            }
            "class_declaration"
            | "interface_declaration"
            | "struct_declaration"
            | "enum_declaration"
            | "record_declaration"
            | "record_struct_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(type_name) = name_node.utf8_text(source.as_bytes()) {
                        let clean_name = type_name.trim();
                        let type_kind = match kind {
                            "interface_declaration" => TypeKind::Interface,
                            "enum_declaration" => TypeKind::Enum,
                            "struct_declaration" | "record_struct_declaration" => TypeKind::Struct,
                            _ => TypeKind::Class,
                        };

                        types.push(TypeInfo {
                            name: clean_name.to_string(),
                            kind: type_kind,
                            line: node.start_position().row + 1,
                        });

                        next_type = Some(clean_name);
                    }
                }
            }
            "method_declaration"
            | "constructor_declaration"
            | "destructor_declaration"
            | "local_function_statement"
            | "operator_declaration"
            | "conversion_operator_declaration"
            | "property_declaration" => {
                if let Some(func) =
                    Self::parse_csharp_function(&node, source, next_namespace, next_type)
                {
                    functions.push(func);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_csharp_node(
                child,
                source,
                next_namespace,
                next_type,
                functions,
                imports,
                types,
            );
        }
    }

    fn parse_csharp_function(
        node: &Node,
        source: &str,
        namespace: Option<&str>,
        container_type: Option<&str>,
    ) -> Option<FunctionInfo> {
        let name = node
            .child_by_field_name("name")
            .and_then(|n| {
                if n.kind() == "generic_name" {
                    n.child_by_field_name("name")
                } else {
                    Some(n)
                }
            })
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                node.children(&mut node.walk())
                    .find(|c| c.kind() == "identifier")
                    .and_then(|c| c.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.trim().to_string())
            })?;

        if name.is_empty() {
            return None;
        }

        let line = node.start_position().row + 1;
        let full_text = node.utf8_text(source.as_bytes()).unwrap_or("");

        let is_interface_member = node
            .parent()
            .map(|p| {
                p.kind() == "interface_declaration"
                    || p.parent()
                        .map(|pp| pp.kind() == "interface_declaration")
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        let is_public = is_interface_member
            || full_text.contains("public ")
            || full_text.contains("protected ");

        let is_async = full_text.contains("async ");

        let return_type = node
            .child_by_field_name("type")
            .or_else(|| node.child_by_field_name("returns"))
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .map(|s| s.trim().to_string());

        let params = node
            .child_by_field_name("parameters")
            .or_else(|| node.child_by_field_name("parameter_list"))
            .map(|p| Self::parse_parameters(&p, source))
            .unwrap_or_default();

        let doc_comment = Self::extract_doc_comment(node, source);

        let body_node = node
            .child_by_field_name("body")
            .or_else(|| {
                node.children(&mut node.walk())
                    .find(|c| c.kind() == "block")
            })
            .or_else(|| node.child_by_field_name("arrow_expression_clause"))
            .or_else(|| {
                node.children(&mut node.walk())
                    .find(|c| c.kind() == "arrow_expression_clause")
            });

        let calls = body_node
            .map(|b| Self::extract_calls(&b, source))
            .unwrap_or_default();

        let (body_start, body_end) = if let Some(body) = body_node {
            (body.start_byte(), body.end_byte())
        } else {
            (node.start_byte(), node.end_byte())
        };

        let resolved_container = match (namespace, container_type) {
            (Some(ns), Some(ty)) => Some(format!("{}::{}", ns, ty)),
            (None, Some(ty)) => Some(ty.to_string()),
            _ => None,
        };

        let role = Self::infer_role(&name, &params);
        let purpose = Self::infer_purpose(&name, &params, &return_type);

        let decorators = Self::extract_decorators(node, source);

        let is_test = Self::has_test_attribute(node, source)
            || full_text.contains("[Fact]")
            || full_text.contains("[Theory]")
            || full_text.contains("[Test]");

        let is_trait_method = is_interface_member;
        let is_trait_default = is_interface_member && body_node.is_some();

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
            container: resolved_container,
            role,
            purpose,
            trait_impl: None,
            decorators,
            is_test,
            is_trait_method,
            is_trait_default,
        })
    }

    // ============================================================
    // Universal AST Functions Extractor (Rust, Python, JS/TS, Go, Java, Dart, PHP, C++)
    // ============================================================

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
            let child_kind = child.kind();

            let matches_function_kind = if child_kind == "variable_declarator" {
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
            } else {
                config.function_kinds.iter().any(|k| k == child_kind)
            };

            if matches_function_kind {
                if let Some(func) = Self::parse_function(
                    &child,
                    source,
                    container,
                    trait_impl,
                    config.name.as_str(),
                ) {
                    out.push(func);
                }
            }

            let (mut next_container, mut next_trait) = (container, trait_impl);

            if child_kind == "impl_item" {
                let ty = child
                    .child_by_field_name("type")
                    .and_then(|t| t.utf8_text(source.as_bytes()).ok());
                let tr = child
                    .child_by_field_name("trait")
                    .and_then(|t| t.utf8_text(source.as_bytes()).ok());
                next_container = ty;
                next_trait = tr;
            } else if matches!(
                child_kind,
                "class_declaration"
                    | "class_definition"
                    | "struct_specifier"
                    | "class_specifier"
                    | "struct_declaration"
                    | "interface_declaration"
                    | "record_declaration"
            ) {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Ok(cname) = name_node.utf8_text(source.as_bytes()) {
                        next_container = Some(cname.trim());
                    }
                }
            }

            Self::walk_for_functions(child, source, config, next_container, next_trait, out);
        }
    }

    fn extract_function_name(node: &Node, source: &str, lang_name: &str) -> Option<String> {
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

    fn parse_function(
        node: &Node,
        source: &str,
        container: Option<&str>,
        trait_impl: Option<&str>,
        lang_name: &str,
    ) -> Option<FunctionInfo> {
        let name = Self::extract_function_name(node, source, lang_name)?;

        let line = node.start_position().row + 1;
        let is_public = Self::is_public(node, source, lang_name, &name);

        let mut resolved_container = container.map(|s| s.to_string());
        if node.kind() == "method_declaration" && resolved_container.is_none() {
            if let Some(receiver) = node.child_by_field_name("receiver") {
                if let Ok(rec_text) = receiver.utf8_text(source.as_bytes()) {
                    let clean = rec_text
                        .trim_matches(|c| c == '(' || c == ')' || c == '*' || c == '&' || c == ' ')
                        .split_whitespace()
                        .last()
                        .unwrap_or("");
                    if !clean.is_empty() {
                        resolved_container = Some(clean.to_string());
                    }
                }
            }
        }

        let is_async = Self::is_async(node, source);
        let return_type = node
            .child_by_field_name("return_type")
            .or_else(|| node.child_by_field_name("type"))
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());

        let params = node
            .child_by_field_name("parameters")
            .or_else(|| node.child_by_field_name("parameter_list"))
            .map(|p| Self::parse_parameters(&p, source))
            .unwrap_or_default();

        let doc_comment = Self::extract_doc_comment(node, source);

        let body_node = node
            .child_by_field_name("body")
            .or_else(|| node.child_by_field_name("arrow_expression_clause"))
            .or_else(|| node.child_by_field_name("expression"));

        let calls = body_node
            .map(|body| Self::extract_calls(&body, source))
            .unwrap_or_default();

        let (body_start, body_end) = if let Some(body) = body_node {
            (body.start_byte(), body.end_byte())
        } else {
            (node.start_byte(), node.end_byte())
        };

        let role = Self::infer_role(&name, &params);
        let purpose = Self::infer_purpose(&name, &params, &return_type);

        let decorators = Self::extract_decorators(node, source);

        let body_start_line = node.start_position().row + 1;
        let body_end_line = node.end_position().row + 1;

        let is_test = Self::has_test_attribute(node, source);
        let is_trait_method = Self::is_trait_method(node, source);
        let is_trait_default = Self::is_trait_default_method(node, source);

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
            body_start_line,
            body_end_line,
            container: resolved_container,
            role,
            purpose,
            trait_impl: trait_impl.map(|s| s.to_string()),
            decorators,
            is_test,
            is_trait_method,
            is_trait_default,
        })
    }

    fn has_test_attribute(node: &Node, source: &str) -> bool {
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

    fn scan_token_tree_for_calls(node: &Node, source: &str, calls: &mut Vec<String>) {
        let mut buf = String::new();

        for child in node.children(&mut node.walk()) {
            let kind = child.kind();
            let text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();

            if kind == "identifier" || kind == "self" || text == "Self" {
                buf.push_str(text);
            } else if text == "::" {
                buf.push_str("::");
            } else if kind == "token_tree" && text.starts_with('(') {
                if !buf.is_empty() && !buf.ends_with("::") {
                    calls.push(buf.clone());
                }
                buf.clear();
                Self::scan_token_tree_for_calls(&child, source, calls);
            } else {
                buf.clear();
                if kind == "token_tree" {
                    Self::scan_token_tree_for_calls(&child, source, calls);
                }
            }
        }
    }

    fn is_trait_method(node: &Node, _source: &str) -> bool {
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

    fn is_trait_default_method(node: &Node, source: &str) -> bool {
        if !Self::is_trait_method(node, source) {
            return false;
        }
        node.child_by_field_name("body").is_some()
    }

    fn parse_parameters(node: &Node, source: &str) -> Vec<ParamInfo> {
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

    fn extract_calls(node: &Node, source: &str) -> Vec<String> {
        let mut calls = Vec::new();
        Self::walk_for_calls_with_context(node, source, &mut calls, None);
        Self::walk_for_jsx_components(node, source, &mut calls);
        Self::extract_closure_calls(node, source, &mut calls);

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

    fn extract_closure_calls(node: &Node, source: &str, calls: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "closure_expression" || child.kind() == "closure" {
                if let Some(body) = child.child_by_field_name("body") {
                    Self::walk_for_calls_with_context(&body, source, calls, None);
                }
            }
            if child.kind() == "argument_list" || child.kind() == "arguments" {
                let mut arg_cursor = child.walk();
                for arg in child.children(&mut arg_cursor) {
                    if arg.kind() == "identifier" || arg.kind() == "scoped_identifier" {
                        if let Ok(name) = arg.utf8_text(source.as_bytes()) {
                            calls.push(name.to_string());
                        }
                    }
                }
            }
            Self::extract_closure_calls(&child, source, calls);
        }
    }

    fn walk_for_jsx_components(node: &Node, source: &str, calls: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "jsx_element" || child.kind() == "jsx_self_closing_element" {
                let attr_container = child.child_by_field_name("open_tag").unwrap_or(child);

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

                Self::extract_jsx_attribute_refs(&attr_container, source, calls);
            }
            Self::walk_for_jsx_components(&child, source, calls);
        }
    }

    fn extract_jsx_attribute_refs(attr_container: &Node, source: &str, calls: &mut Vec<String>) {
        let mut cursor = attr_container.walk();
        for attr in attr_container.children(&mut cursor) {
            if attr.kind() != "jsx_attribute" {
                continue;
            }

            let mut attr_cursor = attr.walk();
            for value in attr.children(&mut attr_cursor) {
                if value.kind() == "jsx_expression" {
                    if let Some(inner) = value.named_child(0) {
                        if inner.kind() == "identifier" {
                            if let Ok(name) = inner.utf8_text(source.as_bytes()) {
                                calls.push(name.to_string());
                            }
                        }
                    }
                }
            }
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
                "call_expression" | "invocation_expression" => {
                    if let Some(func) = child
                        .child_by_field_name("function")
                        .or_else(|| child.child_by_field_name("expression"))
                    {
                        if func.kind() == "field_expression"
                            || func.kind() == "member_access_expression"
                        {
                            let receiver = func
                                .child_by_field_name("value")
                                .or_else(|| func.child_by_field_name("expression"))
                                .and_then(|v| v.utf8_text(source.as_bytes()).ok())
                                .unwrap_or("")
                                .trim();

                            if let Some(method) = func
                                .child_by_field_name("field")
                                .or_else(|| func.child_by_field_name("name"))
                            {
                                if let Ok(method_name) = method.utf8_text(source.as_bytes()) {
                                    if receiver == "self" || receiver == "this" {
                                        calls.push(format!("self::{}", method_name));
                                    } else if receiver == "Self" {
                                        calls.push(format!("Self::{}", method_name));
                                    } else if receiver.contains("::") {
                                        calls.push(format!("{}::{}", receiver, method_name));
                                    } else {
                                        calls.push(format!("{}.{}", receiver, method_name));
                                        if let Some(container) = receiver_type {
                                            calls.push(format!("{}::{}", container, method_name));
                                        }
                                    }
                                }
                            }
                        } else if func.kind() == "scoped_identifier"
                            || func.kind() == "qualified_identifier"
                        {
                            if let Ok(name) = func.utf8_text(source.as_bytes()) {
                                calls.push(name.to_string());
                            }
                        } else if let Ok(name) = func.utf8_text(source.as_bytes()) {
                            calls.push(name.to_string());
                        }
                    }
                }
                "method_call" | "method_invocation" => {
                    if let Some(receiver) = child.child_by_field_name("receiver") {
                        let receiver_text = receiver
                            .utf8_text(source.as_bytes())
                            .unwrap_or("self")
                            .trim();

                        if let Some(method) = child.child_by_field_name("method") {
                            if let Ok(method_name) = method.utf8_text(source.as_bytes()) {
                                if receiver_text == "self" || receiver_text == "this" {
                                    calls.push(format!("self::{}", method_name));
                                } else if receiver_text == "Self" {
                                    calls.push(format!("Self::{}", method_name));
                                } else if receiver_text.contains("::") {
                                    calls.push(format!("{}::{}", receiver_text, method_name));
                                } else {
                                    calls.push(format!("{}.{}", receiver_text, method_name));
                                    if let Some(container) = receiver_type {
                                        calls.push(format!("{}::{}", container, method_name));
                                    }
                                }
                            }
                        }
                    }
                }
                "chain_expression" | "chained_call" => {
                    Self::walk_for_calls_with_context(&child, source, calls, receiver_type);
                }
                "index_expression" => {
                    calls.push("op::index".to_string());
                }
                "pair" => {
                    if let Some(value) = child.child_by_field_name("value") {
                        if value.kind() == "identifier" {
                            if let Ok(name) = value.utf8_text(source.as_bytes()) {
                                calls.push(name.to_string());
                            }
                        }
                    }
                }
                "shorthand_property_identifier" => {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        calls.push(name.to_string());
                    }
                }
                "macro_invocation" => {
                    if let Some(token_tree) = child
                        .children(&mut child.walk())
                        .find(|c| c.kind() == "token_tree")
                    {
                        Self::scan_token_tree_for_calls(&token_tree, source, calls);
                    }
                }
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
                "field_expression" | "member_access_expression" => {
                    if let (Some(value), Some(field)) = (
                        child
                            .child_by_field_name("value")
                            .or_else(|| child.child_by_field_name("expression")),
                        child
                            .child_by_field_name("field")
                            .or_else(|| child.child_by_field_name("name")),
                    ) {
                        if let (Ok(receiver), Ok(method_name)) = (
                            value.utf8_text(source.as_bytes()),
                            field.utf8_text(source.as_bytes()),
                        ) {
                            let receiver = receiver.trim();
                            if receiver == "self" || receiver == "this" {
                                calls.push(format!("self::{}", method_name));
                            } else if receiver == "Self" {
                                calls.push(format!("Self::{}", method_name));
                            } else if receiver.contains("::") {
                                calls.push(format!("{}::{}", receiver, method_name));
                            } else {
                                calls.push(format!("{}.{}", receiver, method_name));
                                if let Some(container) = receiver_type {
                                    calls.push(format!("{}::{}", container, method_name));
                                }
                            }
                        }
                    }
                }
                "let_declaration" | "variable_declaration" => {
                    if let Some(pattern) = child.child_by_field_name("pattern") {
                        if let Some(type_node) = child.child_by_field_name("type") {
                            if let (Ok(_var_name), Ok(type_name)) = (
                                pattern.utf8_text(source.as_bytes()),
                                type_node.utf8_text(source.as_bytes()),
                            ) {
                                Self::walk_for_calls_with_context(
                                    &child,
                                    source,
                                    calls,
                                    Some(type_name.trim()),
                                );
                                continue;
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
        if trimmed.starts_with("using ") {
            let rest = trimmed
                .trim_start_matches("using ")
                .trim_end_matches(';')
                .trim();
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

    fn is_public(node: &Node, source: &str, lang_name: &str, func_name: &str) -> bool {
        match lang_name {
            "Rust" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    text.contains("pub ")
                } else {
                    false
                }
            }
            "Go" => func_name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false),
            "Python" => !func_name.starts_with('_') || func_name.starts_with("__init__"),
            "Java" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    text.contains("public ")
                } else {
                    false
                }
            }
            "Dart" => !func_name.starts_with('_'),
            "PHP" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    text.contains("public ")
                        || (!text.contains("private ") && !text.contains("protected "))
                } else {
                    true
                }
            }
            "CPP" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    text.contains("extern ")
                        || text.contains("export ")
                        || !text.contains("static ")
                } else {
                    true
                }
            }
            "CSharp" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    text.contains("public ")
                        || text.contains("protected ")
                        || text.contains("internal ")
                } else {
                    false
                }
            }
            "JavaScript" | "TypeScript" => {
                let mut curr = Some(*node);
                while let Some(n) = curr {
                    let kind = n.kind();
                    if kind == "export_statement"
                        || kind == "export_default"
                        || kind == "export_declaration"
                    {
                        return true;
                    }
                    if let Ok(text) = n.utf8_text(source.as_bytes()) {
                        let trimmed = text.trim_start();
                        if trimmed.starts_with("export ") || trimmed.starts_with("export default ")
                        {
                            return true;
                        }
                    }
                    curr = n.parent();
                }
                false
            }
            _ => false,
        }
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
