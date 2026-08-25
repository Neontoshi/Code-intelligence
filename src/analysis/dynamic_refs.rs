// src/analysis/dynamic_refs.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DynamicRefType {
    Reflection,
    Callback,
    Framework,
    DynamicImport,
    DependencyInjection,
    StringDispatch,
    DynamicDispatch,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    pub fn from_extracted(
        file: &ParsedFile,
        dyn_call: &ExtractedDynamicCall,
        resolved_path: Option<String>,
    ) -> Self {
        let resolved = resolved_path.is_some();
        Self {
            source_file: file.path.clone(),
            source_function: dyn_call.enclosing_function.clone(),
            target_function: Some(dyn_call.target_name.clone()),
            target_full_path: resolved_path,
            target_pattern: dyn_call.pattern.clone(),
            reference_type: dyn_call.ref_type.clone(),
            confidence: dyn_call.confidence,
            context: dyn_call.context.clone(),
            resolved,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }
}

pub struct ExtractedDynamicCall {
    pub enclosing_function: Option<String>,
    pub target_name: String,
    pub pattern: String,
    pub ref_type: DynamicRefType,
    pub confidence: f64,
    pub context: String,
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

        // 1. Build indexed lookup structures for symbol resolution
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

            if let Some(short_name) = func.full_path.rsplit("::").next() {
                unqualified_to_paths
                    .entry(short_name.to_string())
                    .or_default()
                    .push(func.full_path.clone());
            }
        }

        for file in files {
            // AST Tree-sitter dynamic call extraction
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

            // Decorator and framework metadata inspection
            for func_info in &file.functions {
                for decorator in &func_info.decorators {
                    let d_lower = decorator.to_lowercase();
                    let resolved_path = Self::resolve_symbol(
                        &func_info.name,
                        &name_to_paths,
                        &lower_name_to_paths,
                        &unqualified_to_paths,
                    );

                    let is_framework = d_lower.contains("route")
                        || d_lower.contains("get")
                        || d_lower.contains("post")
                        || d_lower.contains("put")
                        || d_lower.contains("delete")
                        || d_lower.contains("patch")
                        || d_lower.contains("mapping")
                        || d_lower.contains("controller")
                        || d_lower.contains("service")
                        || d_lower.contains("repository")
                        || d_lower.contains("injectable")
                        || d_lower.contains("blueprint")
                        || d_lower.contains("api");

                    if is_framework {
                        refs.push(DynamicReference::new_framework(
                            file.path.clone(),
                            Some(func_info.name.clone()),
                            func_info.name.clone(),
                            resolved_path.clone(),
                            decorator.clone(),
                            0.95,
                            format!("Decorated endpoint/service: {}", decorator),
                        ));
                    }
                }

                // JS/TS React components & hooks
                let is_js_ts = file.path.ends_with(".tsx")
                    || file.path.ends_with(".jsx")
                    || file.path.ends_with(".ts")
                    || file.path.ends_with(".js")
                    || file.language.to_lowercase() == "typescript"
                    || file.language.to_lowercase() == "javascript";

                if is_js_ts {
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

                // Dart / Flutter Widget & State lifecycle
                if file.path.ends_with(".dart") || file.language.to_lowercase() == "dart" {
                    if func_info.name == "build"
                        || func_info.name == "initState"
                        || func_info.name == "dispose"
                        || func_info.name == "didUpdateWidget"
                        || func_info.name == "createState"
                    {
                        let resolved_path = Self::resolve_symbol(
                            &func_info.name,
                            &name_to_paths,
                            &lower_name_to_paths,
                            &unqualified_to_paths,
                        );

                        refs.push(DynamicReference::new_framework(
                            file.path.clone(),
                            Some(func_info.name.clone()),
                            func_info.name.clone(),
                            resolved_path,
                            "FlutterLifecycle".to_string(),
                            0.95,
                            "Flutter Widget/State lifecycle method".to_string(),
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

            // ============================================================
            // Polyglot Fallback & Pattern Scanners (All 10 Languages)
            // ============================================================
            let content = &file.source;
            let lang_lower = file.language.to_lowercase();

            // 1. Python
            if lang_lower.contains("python") || file.path.ends_with(".py") {
                if content.contains("getattr(")
                    || content.contains("setattr(")
                    || content.contains("hasattr(")
                    || content.contains("importlib")
                    || content.contains("__import__")
                {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "getattr".to_string(),
                        None,
                        "getattr".to_string(),
                        0.85,
                        "Python reflection dispatch".to_string(),
                    ));
                }
                if content.contains("@app.route") || content.contains("@router.") {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "app.route".to_string(),
                        None,
                        "@app.route".to_string(),
                        0.95,
                        "Flask/FastAPI route handler".to_string(),
                    ));
                }
            }

            // 2. JavaScript / TypeScript
            if lang_lower.contains("javascript")
                || lang_lower.contains("typescript")
                || file.path.ends_with(".js")
                || file.path.ends_with(".ts")
                || file.path.ends_with(".tsx")
                || file.path.ends_with(".jsx")
            {
                if content.contains("import(") || content.contains("require(") {
                    refs.push(DynamicReference::new_dynamic_import(
                        file.path.clone(),
                        "dynamic_import".to_string(),
                        None,
                        "import()".to_string(),
                        0.85,
                        "Dynamic ES module import".to_string(),
                    ));
                }
            }

            // 3. Go
            if lang_lower.contains("go") || file.path.ends_with(".go") {
                if content.contains("reflect.")
                    || content.contains("\"reflect\"")
                    || content.contains("MethodByName")
                {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "reflect".to_string(),
                        None,
                        "reflect.ValueOf".to_string(),
                        0.90,
                        "Go reflect package usage".to_string(),
                    ));
                }
            }

            // 4. Rust
            if lang_lower.contains("rust") || file.path.ends_with(".rs") {
                if content.contains("dyn ")
                    || content.contains("Box<dyn")
                    || content.contains("&dyn")
                {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("dyn".to_string()),
                        target_full_path: None,
                        target_pattern: "dyn Trait".to_string(),
                        reference_type: DynamicRefType::DynamicDispatch,
                        confidence: 0.85,
                        context: "Rust trait object dynamic dispatch".to_string(),
                        resolved: false,
                    });
                }
            }

            // 5. PHP
            if lang_lower.contains("php") || file.path.ends_with(".php") {
                if content.contains("call_user_func")
                    || content.contains("call_user_func_array")
                    || content.contains("forward_static_call")
                    || content.contains("ReflectionClass")
                    || content.contains("ReflectionMethod")
                {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "call_user_func".to_string(),
                        None,
                        "call_user_func".to_string(),
                        0.90,
                        "PHP dynamic call dispatch".to_string(),
                    ));
                }
            }

            // 6. C# / .NET
            if lang_lower.contains("csharp")
                || lang_lower.contains("c#")
                || file.path.ends_with(".cs")
            {
                if content.contains("typeof(") && content.contains(".GetMethod(")
                    || content.contains("Activator.CreateInstance")
                    || content.contains("MethodInfo.Invoke")
                {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "GetMethod".to_string(),
                        None,
                        "Reflection.Invoke".to_string(),
                        0.90,
                        "C# System.Reflection invocation".to_string(),
                    ));
                }
                if content.contains("[HttpGet")
                    || content.contains("[HttpPost")
                    || content.contains("[HttpPut")
                    || content.contains("[HttpDelete")
                    || content.contains("[ApiController]")
                {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "ApiController".to_string(),
                        None,
                        "[ApiController]".to_string(),
                        0.95,
                        "ASP.NET Core route action".to_string(),
                    ));
                }
            }

            // 7. Java
            if lang_lower.contains("java") || file.path.ends_with(".java") {
                if content.contains(".getMethod(")
                    || content.contains(".invoke(")
                    || content.contains("Class.forName(")
                {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "getMethod".to_string(),
                        None,
                        "java.lang.reflect".to_string(),
                        0.90,
                        "Java reflection invocation".to_string(),
                    ));
                }
                if content.contains("@GetMapping")
                    || content.contains("@PostMapping")
                    || content.contains("@RequestMapping")
                    || content.contains("@RestController")
                {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "RestController".to_string(),
                        None,
                        "@RestController".to_string(),
                        0.95,
                        "Spring / Jakarta Web controller endpoint".to_string(),
                    ));
                }
            }

            // 8. Dart
            if lang_lower.contains("dart") || file.path.ends_with(".dart") {
                if content.contains("Widget build(")
                    || content.contains("initState()")
                    || content.contains("dispose()")
                {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "Widget.build".to_string(),
                        None,
                        "FlutterWidget".to_string(),
                        0.95,
                        "Flutter widget lifecycle override".to_string(),
                    ));
                }
            }

            // 9. C++
            if lang_lower.contains("cpp")
                || lang_lower.contains("c++")
                || file.path.ends_with(".cpp")
                || file.path.ends_with(".hpp")
                || file.path.ends_with(".cc")
            {
                if content.contains("extern \"C\"")
                    || content.contains("Q_INVOKABLE")
                    || content.contains("EMSCRIPTEN_KEEPALIVE")
                {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "extern_c".to_string(),
                        None,
                        "extern \"C\"".to_string(),
                        0.95,
                        "C++ FFI / Native exported entry point".to_string(),
                    ));
                }
                if content.contains("virtual ") || content.contains("override") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("virtual".to_string()),
                        target_full_path: None,
                        target_pattern: "virtual method".to_string(),
                        reference_type: DynamicRefType::DynamicDispatch,
                        confidence: 0.85,
                        context: "C++ virtual polymorphic dispatch".to_string(),
                        resolved: false,
                    });
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

    fn extract_dynamic_calls_via_ast(file: &ParsedFile) -> Vec<ExtractedDynamicCall> {
        let mut extracted = Vec::new();
        let lang_lower = file.language.to_lowercase();

        let lang: Language = if lang_lower.contains("python") {
            tree_sitter_python::LANGUAGE.into()
        } else if lang_lower.contains("go") {
            tree_sitter_go::LANGUAGE.into()
        } else if lang_lower.contains("typescript") {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        } else if lang_lower.contains("javascript") {
            tree_sitter_javascript::LANGUAGE.into()
        } else if lang_lower.contains("php") {
            tree_sitter_php::LANGUAGE_PHP.into()
        } else if lang_lower.contains("cpp") {
            tree_sitter_cpp::LANGUAGE.into()
        } else {
            return extracted;
        };

        let mut parser = Parser::new();
        if parser.set_language(&lang).is_err() {
            return extracted;
        }

        let tree = match parser.parse(&file.source, None) {
            Some(t) => t,
            None => return extracted,
        };

        if lang_lower.contains("python") {
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
                query_str,
                &lang,
                tree.root_node(),
                &file.source,
                |target, node| {
                    let clean_target = target.trim_matches(|c| c == '"' || c == '\'').to_string();
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
        } else if lang_lower.contains("go") {
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
                query_str,
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
        } else if lang_lower.contains("php") {
            let query_str = r#"
                (function_call_expression
                    function: (name) @fn (#eq? @fn "call_user_func")
                    arguments: (arguments
                        (string (string_content) @target_str)
                    )
                )
            "#;
            Self::run_ast_query(
                query_str,
                &lang,
                tree.root_node(),
                &file.source,
                |target, node| {
                    let clean_target = target.trim_matches(|c| c == '"' || c == '\'').to_string();
                    extracted.push(ExtractedDynamicCall {
                        enclosing_function: Self::find_enclosing_function(node, &file.source),
                        target_name: clean_target,
                        pattern: "call_user_func".to_string(),
                        ref_type: DynamicRefType::Reflection,
                        confidence: 0.90,
                        context: "PHP call_user_func dispatch".to_string(),
                    });
                },
            );
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
        use streaming_iterator::StreamingIterator;

        if let Ok(query) = Query::new(language, query_str) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, root, source.as_bytes());

            while let Some(m) = matches.next() {
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
        if let Some(paths) = name_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        if let Some(paths) = unqualified_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        let lower = name.to_lowercase();
        if let Some(paths) = lower_name_to_paths.get(&lower) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
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

impl Default for DynamicRefDetector {
    fn default() -> Self {
        Self::new()
    }
}
