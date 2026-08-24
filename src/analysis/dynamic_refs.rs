// src/analysis/dynamic_refs.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

#[derive(Debug, Clone)]
pub struct DynamicReference {
    pub source_file: String,
    pub source_function: Option<String>,
    pub target_function: Option<String>,
    pub target_full_path: Option<String>,
    pub target_pattern: String,
    pub reference_type: DynamicRefType,
    pub confidence: f64,
    pub context: String,
    pub resolved: bool,
}

impl DynamicReference {
    /// Create a new framework dynamic reference (decorators, routes, etc.)
    pub fn new_framework(
        source_file: String,
        source_function: Option<String>,
        target_function: String,
        target_full_path: Option<String>,
        target_pattern: String,
        confidence: f64,
        context: String,
    ) -> Self {
        let resolved = target_full_path.is_some();
        Self {
            source_file,
            source_function,
            target_function: Some(target_function),
            target_full_path,
            target_pattern,
            reference_type: DynamicRefType::Framework,
            confidence,
            context,
            resolved,
        }
    }

    /// Create a new dynamic import reference
    pub fn new_dynamic_import(
        source_file: String,
        target_function: String,
        target_full_path: Option<String>,
        target_pattern: String,
        confidence: f64,
        context: String,
    ) -> Self {
        let resolved = target_full_path.is_some();
        Self {
            source_file,
            source_function: None,
            target_function: Some(target_function),
            target_full_path,
            target_pattern,
            reference_type: DynamicRefType::DynamicImport,
            confidence,
            context,
            resolved,
        }
    }

    /// Create a new reflection reference
    pub fn new_reflection(
        source_file: String,
        source_function: Option<String>,
        target_function: String,
        target_full_path: Option<String>,
        target_pattern: String,
        confidence: f64,
        context: String,
    ) -> Self {
        let resolved = target_full_path.is_some();
        Self {
            source_file,
            source_function,
            target_function: Some(target_function),
            target_full_path,
            target_pattern,
            reference_type: DynamicRefType::Reflection,
            confidence,
            context,
            resolved,
        }
    }

    /// Create from an extracted dynamic call
    pub fn from_extracted(
        file: &ParsedFile,
        dyn_call: &ExtractedDynamicCall,
        resolved_path: Option<String>,
    ) -> Self {
        let resolved = resolved_path.is_some();
        let ref_type = dyn_call.ref_type.clone();
        let confidence = dyn_call.confidence;
        let context = dyn_call.context.clone();
        let pattern = dyn_call.pattern.clone();
        let target_name = dyn_call.target_name.clone();

        Self {
            source_file: file.path.clone(),
            source_function: dyn_call.enclosing_function.clone(),
            target_function: Some(target_name),
            target_full_path: resolved_path,
            target_pattern: pattern,
            reference_type: ref_type,
            confidence,
            context,
            resolved,
        }
    }

    /// Check if this reference is resolved
    pub fn is_resolved(&self) -> bool {
        self.resolved
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DynamicRefType {
    Reflection,
    Callback,
    Framework,
    DynamicImport,
    DependencyInjection,
    StringDispatch,
    Unknown,
}

pub struct DynamicRefDetector;

impl DynamicRefDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_all(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Vec<DynamicReference> {
        let mut refs = Vec::new();

        // 1. Build indexed lookup structures for O(1) symbol resolution
        let mut name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        let mut lower_name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        let mut unqualified_to_paths: HashMap<String, Vec<String>> = HashMap::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            name_to_paths
                .entry(func.name.clone())
                .or_default()
                .push(func.full_path.clone());

            lower_name_to_paths
                .entry(func.name.to_lowercase())
                .or_default()
                .push(func.full_path.clone());

            // Map short/unqualified function name from full path
            if let Some(short_name) = func.full_path.rsplit("::").next() {
                unqualified_to_paths
                    .entry(short_name.to_string())
                    .or_default()
                    .push(func.full_path.clone());
            }
        }

        for file in files {
            // AST-level extraction per language
            let ast_dynamic_calls = Self::extract_dynamic_calls_via_ast(file);

            for dyn_call in ast_dynamic_calls {
                let resolved_path = Self::resolve_symbol(
                    &dyn_call.target_name,
                    &name_to_paths,
                    &lower_name_to_paths,
                    &unqualified_to_paths,
                );

                refs.push(DynamicReference::from_extracted(
                    file,
                    &dyn_call,
                    resolved_path,
                ));
            }

            for func_info in &file.functions {
                // Decorator & Framework inspection
                for decorator in &func_info.decorators {
                    let d_lower = decorator.to_lowercase();
                    let resolved_path = Self::resolve_symbol(
                        &func_info.name,
                        &name_to_paths,
                        &lower_name_to_paths,
                        &unqualified_to_paths,
                    );

                    let is_route = d_lower.contains("route")
                        || d_lower.contains("get")
                        || d_lower.contains("post")
                        || d_lower.contains("put")
                        || d_lower.contains("delete")
                        || d_lower.contains("mapping")
                        || d_lower.contains("controller")
                        || d_lower.contains("injectable")
                        || d_lower.contains("blueprint");

                    if is_route {
                        refs.push(DynamicReference::new_framework(
                            file.path.clone(),
                            Some(func_info.name.clone()),
                            func_info.name.clone(),
                            resolved_path.clone(),
                            decorator.clone(),
                            0.95,
                            format!("Decorated endpoint: {}", decorator),
                        ));
                    }
                }

                // React components and hooks
                if file.path.ends_with(".tsx") || file.path.ends_with(".jsx") {
                    let is_component = func_info
                        .name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);

                    let is_hook = func_info.name.starts_with("use");

                    if is_component || is_hook {
                        let resolved_path = Self::resolve_symbol(
                            &func_info.name,
                            &name_to_paths,
                            &lower_name_to_paths,
                            &unqualified_to_paths,
                        );

                        let (pattern, confidence, context) = if is_component {
                            ("JSXComponent", 0.90, "React Component")
                        } else {
                            ("ReactHook", 0.85, "React Hook")
                        };

                        refs.push(DynamicReference::new_framework(
                            file.path.clone(),
                            Some(func_info.name.clone()),
                            func_info.name.clone(),
                            resolved_path,
                            pattern.to_string(),
                            confidence,
                            context.to_string(),
                        ));
                    }
                }
            }

            // File-level imports
            for import in &file.imports {
                if import.module.contains("dynamic")
                    || import.module.contains("lazy")
                    || import.module.contains("plugin")
                    || import.module.contains("importlib")
                {
                    let resolved = Self::resolve_symbol(
                        &import.module,
                        &name_to_paths,
                        &lower_name_to_paths,
                        &unqualified_to_paths,
                    );

                    refs.push(DynamicReference::new_dynamic_import(
                        file.path.clone(),
                        import.module.clone(),
                        resolved,
                        import.module.clone(),
                        0.80,
                        format!("Dynamic import statement: {}", import.module),
                    ));
                }
            }
        }

        // Deduplicate references
        let mut seen = HashSet::new();
        refs.retain(|r| {
            seen.insert((
                r.source_file.clone(),
                r.target_full_path.clone().unwrap_or_default(),
                r.target_pattern.clone(),
                r.reference_type.clone(),
            ))
        });

        refs
    }

    /// AST Tree-Sitter based target extraction
    fn extract_dynamic_calls_via_ast(file: &ParsedFile) -> Vec<ExtractedDynamicCall> {
        let mut extracted = Vec::new();
        let lang = match file.language.as_str() {
            "python" => tree_sitter_python::language(),
            "go" => tree_sitter_go::language(),
            "javascript" => tree_sitter_javascript::language(),
            "typescript" => tree_sitter_typescript::language_typescript(),
            _ => return extracted,
        };

        let mut parser = Parser::new();
        if parser.set_language(&lang).is_err() {
            return extracted;
        }

        let tree = match parser.parse(&file.source, None) {
            Some(t) => t,
            None => return extracted,
        };

        match file.language.as_str() {
            "python" => {
                // Match getattr(obj, "method_name")
                let query_str = r#"
                    (call
                        function: (identifier) @fn_name (#eq? @fn_name "getattr")
                        arguments: (argument_list
                            (_)
                            (string) @target_str
                        )
                    )
                "#;
                Self::run_ast_query(
                    &query_str,
                    &lang,
                    tree.root_node(),
                    &file.source,
                    |target, node| {
                        let clean_target =
                            target.trim_matches(|c| c == '"' || c == '\'').to_string();
                        extracted.push(ExtractedDynamicCall {
                            enclosing_function: Self::find_enclosing_function(node, &file.source),
                            target_name: clean_target,
                            pattern: "getattr".to_string(),
                            ref_type: DynamicRefType::Reflection,
                            confidence: 0.85,
                            context: "Python getattr() reflection dispatch".to_string(),
                        });
                    },
                );
            }
            "go" => {
                // Match MethodByName("MethodName")
                let query_str = r#"
                    (call_expression
                        function: (selector_expression
                            field: (field_identifier) @method (#eq? @method "MethodByName")
                        )
                        arguments: (argument_list
                            (interpreted_string_literal) @target_str
                        )
                    )
                "#;
                Self::run_ast_query(
                    &query_str,
                    &lang,
                    tree.root_node(),
                    &file.source,
                    |target, node| {
                        let clean_target = target.trim_matches('"').to_string();
                        extracted.push(ExtractedDynamicCall {
                            enclosing_function: Self::find_enclosing_function(node, &file.source),
                            target_name: clean_target,
                            pattern: "MethodByName".to_string(),
                            ref_type: DynamicRefType::Reflection,
                            confidence: 0.90,
                            context: "Go reflect.MethodByName() dispatch".to_string(),
                        });
                    },
                );
            }
            "javascript" | "typescript" => {
                // 1. Match import("./path") or require("./path")
                let import_query = r#"
                                (call_expression
                                    function: (identifier) @fn (#match? @fn "^(import|require)$")
                                    arguments: (arguments
                                        (string (string_fragment) @target_str)
                                    )
                                )
                            "#;
                Self::run_ast_query(
                    import_query,
                    &lang,
                    tree.root_node(),
                    &file.source,
                    |target, node| {
                        let clean_target = target.trim_matches(|c| c == '"' || c == '\'');
                        let target_fn = clean_target
                            .split('/')
                            .last()
                            .and_then(|seg| seg.split('.').next())
                            .unwrap_or(clean_target)
                            .to_string();

                        extracted.push(ExtractedDynamicCall {
                            enclosing_function: Self::find_enclosing_function(node, &file.source),
                            target_name: target_fn,
                            pattern: "DynamicImport".to_string(),
                            ref_type: DynamicRefType::DynamicImport,
                            confidence: 0.80,
                            context: "Dynamic module import/require".to_string(),
                        });
                    },
                );

                // 2. Match Tauri invoke('command_name') and Electron IPC calls
                let ipc_query = r#"
                                (call_expression
                                    function: [
                                        (identifier) @fn_name (#match? @fn_name "^(invoke|emit|send)$")
                                        (member_expression
                                            property: (property_identifier) @prop_name (#match? @prop_name "^(invoke|send|emit|sendSync)$")
                                        )
                                    ]
                                    arguments: (arguments
                                        (string (string_fragment) @target_str)
                                    )
                                )
                            "#;
                Self::run_ast_query(
                    ipc_query,
                    &lang,
                    tree.root_node(),
                    &file.source,
                    |target, node| {
                        let clean_target =
                            target.trim_matches(|c| c == '"' || c == '\'').to_string();
                        extracted.push(ExtractedDynamicCall {
                            enclosing_function: Self::find_enclosing_function(node, &file.source),
                            target_name: clean_target.clone(),
                            pattern: format!("IPC:{}", clean_target),
                            ref_type: DynamicRefType::Framework,
                            confidence: 0.95,
                            context: "Tauri/Electron IPC command dispatch".to_string(),
                        });
                    },
                );
            }
            _ => {}
        }

        extracted
    }

    fn run_ast_query<F>(
        query_str: &str,
        language: &Language,
        root: Node,
        source: &str,
        mut handler: F,
    ) where
        F: FnMut(String, Node),
    {
        if let Ok(query) = Query::new(language, query_str) {
            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&query, root, source.as_bytes());

            for m in matches {
                for capture in m.captures {
                    let capture_name = &query.capture_names()[capture.index as usize];
                    if *capture_name == "target_str" {
                        if let Ok(text) = capture.node.utf8_text(source.as_bytes()) {
                            handler(text.to_string(), capture.node);
                        }
                    }
                }
            }
        }
    }

    fn find_enclosing_function(mut node: Node, source: &str) -> Option<String> {
        while let Some(parent) = node.parent() {
            let kind = parent.kind();
            if kind == "function_item"
                || kind == "function_declaration"
                || kind == "function_definition"
                || kind == "method_declaration"
                || kind == "method_definition"
            {
                if let Some(name_node) = parent.child_by_field_name("name") {
                    return name_node
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(|s| s.to_string());
                }
            }
            node = parent;
        }
        None
    }

    fn resolve_symbol(
        name: &str,
        name_to_paths: &HashMap<String, Vec<String>>,
        lower_name_to_paths: &HashMap<String, Vec<String>>,
        unqualified_to_paths: &HashMap<String, Vec<String>>,
    ) -> Option<String> {
        // 1. Exact match
        if let Some(paths) = name_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        // 2. Unqualified index match (O(1))
        if let Some(paths) = unqualified_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        // 3. Lowercase match
        let lower = name.to_lowercase();
        if let Some(paths) = lower_name_to_paths.get(&lower) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        // 4. Strip common accessors
        let stripped = name
            .trim_start_matches("get_")
            .trim_start_matches("set_")
            .trim_start_matches("is_")
            .trim_start_matches("has_");

        if stripped != name {
            if let Some(paths) = name_to_paths.get(stripped) {
                if paths.len() == 1 {
                    return Some(paths[0].clone());
                }
            }
        }

        None
    }

    pub fn generate_report(&self, refs: &[DynamicReference]) -> String {
        let mut output = String::new();
        output.push_str("## 🔄 Dynamic Reference Detection\n\n");
        if refs.is_empty() {
            output.push_str("✅ No dynamic references detected.\n");
            return output;
        }

        output.push_str(&format!("Found **{}** dynamic references:\n\n", refs.len()));
        let mut by_type: HashMap<DynamicRefType, Vec<&DynamicReference>> = HashMap::new();
        for r in refs {
            by_type.entry(r.reference_type.clone()).or_default().push(r);
        }

        for (ref_type, refs_by_type) in &by_type {
            output.push_str(&format!("### {:?} ({})\n\n", ref_type, refs_by_type.len()));
            for r in refs_by_type {
                let resolved_info = if let Some(path) = &r.target_full_path {
                    format!(" → resolved to `{}`", path)
                } else {
                    " ⚠️ unresolved".to_string()
                };
                output.push_str(&format!(
                    "- 🟢 **{}** (confidence: {:.0}%){}\n  - File: `{}`\n  - Context: {}\n",
                    r.target_pattern,
                    r.confidence * 100.0,
                    resolved_info,
                    r.source_file,
                    r.context
                ));
            }
            output.push('\n');
        }
        output
    }
}

pub struct ExtractedDynamicCall {
    enclosing_function: Option<String>,
    target_name: String,
    pattern: String,
    ref_type: DynamicRefType,
    confidence: f64,
    context: String,
}

impl Default for DynamicRefDetector {
    fn default() -> Self {
        Self::new()
    }
}
