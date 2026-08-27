// src/analysis/dynamic_refs/mod.rs

//! Unified dynamic reference detection

pub mod common;

pub use common::{DynamicRefType, DynamicReference, ExtractedDynamicCall};

use crate::analysis::framework_registry::FrameworkRegistry;
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet};

pub struct DynamicRefDetector {
    framework_registry: FrameworkRegistry,
}

impl DynamicRefDetector {
    pub fn new() -> Self {
        Self {
            framework_registry: FrameworkRegistry::new(),
        }
    }

    pub fn detect_all(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Vec<DynamicReference> {
        let mut all_refs = Vec::new();
        let context = DetectionContext::new(call_graph, files);

        for file in files {
            let language = file.language.to_lowercase();
            let source = &file.source;
            let refs = self.detect_in_file(file, &language, source, &context);
            all_refs.extend(refs);
        }

        // Deduplicate
        let mut seen = HashSet::new();
        all_refs.retain(|r| {
            let key = (
                r.source_file.clone(),
                r.target_full_path.clone().unwrap_or_default(),
                r.target_pattern.clone(),
                r.reference_type.clone(),
            );
            seen.insert(key)
        });

        all_refs
    }

    fn detect_in_file(
        &self,
        file: &ParsedFile,
        language: &str,
        source: &str,
        context: &DetectionContext,
    ) -> Vec<DynamicReference> {
        let mut refs = Vec::new();

        // Common patterns across all languages

        // Reflection
        self.detect_reflection(file, language, source, context, &mut refs);

        // Dynamic imports
        self.detect_dynamic_imports(file, language, source, context, &mut refs);

        // Framework patterns from registry
        if self
            .framework_registry
            .is_dynamic_behavior(language, source)
        {
            refs.push(DynamicReference::new_framework(
                file.path.clone(),
                None,
                "framework_dynamic".to_string(),
                None,
                "framework_dynamic_behavior".to_string(),
                0.90,
                format!("Framework dynamic behavior detected in {}", language),
            ));
        }

        // Language-specific patterns
        match language {
            "rust" => {
                if source.contains("dyn ") || source.contains("Box<dyn") || source.contains("&dyn")
                {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("dyn".to_string()),
                        target_full_path: None,
                        target_pattern: "dyn Trait".to_string(),
                        reference_type: DynamicRefType::TraitDispatch,
                        confidence: 0.85,
                        context: "Rust trait object dynamic dispatch".to_string(),
                        resolved: false,
                        kind: "trait_dispatch".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
                if source.contains("#[no_mangle]") || source.contains("extern \"C\"") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("ffi_export".to_string()),
                        target_full_path: None,
                        target_pattern: "extern \"C\"".to_string(),
                        reference_type: DynamicRefType::FFI,
                        confidence: 0.95,
                        context: "Rust FFI exported function".to_string(),
                        resolved: false,
                        kind: "ffi".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
            }
            "go" => {
                if source.contains("reflect.") || source.contains("MethodByName") {
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
                if source.contains("interface{}") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("interface".to_string()),
                        target_full_path: None,
                        target_pattern: "interface{}".to_string(),
                        reference_type: DynamicRefType::TraitDispatch,
                        confidence: 0.80,
                        context: "Go interface dynamic dispatch".to_string(),
                        resolved: false,
                        kind: "interface_dispatch".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
                if source.contains("go func") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("goroutine".to_string()),
                        target_full_path: None,
                        target_pattern: "go func".to_string(),
                        reference_type: DynamicRefType::RuntimeRegistration,
                        confidence: 0.85,
                        context: "Go goroutine runtime registration".to_string(),
                        resolved: false,
                        kind: "goroutine".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
            }
            "python" => {
                if source.contains("getattr(")
                    || source.contains("setattr(")
                    || source.contains("hasattr(")
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
                if source.contains("importlib") || source.contains("__import__") {
                    refs.push(DynamicReference::new_dynamic_import(
                        file.path.clone(),
                        "importlib".to_string(),
                        None,
                        "importlib".to_string(),
                        0.85,
                        "Python dynamic import".to_string(),
                    ));
                }
                if source.contains("@app.route") || source.contains("@router.") {
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
            "java" => {
                if source.contains("Class.forName")
                    || source.contains("getMethod")
                    || source.contains("invoke(")
                {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "reflection".to_string(),
                        None,
                        "Class.forName".to_string(),
                        0.90,
                        "Java reflection".to_string(),
                    ));
                }
                if source.contains("@Autowired") || source.contains("@Inject") {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "dependency_injection".to_string(),
                        None,
                        "@Autowired".to_string(),
                        0.95,
                        "Dependency injection".to_string(),
                    ));
                }
            }
            "cpp" => {
                if source.contains("virtual") || source.contains("override") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("virtual_method".to_string()),
                        target_full_path: None,
                        target_pattern: "virtual method".to_string(),
                        reference_type: DynamicRefType::DynamicDispatch,
                        confidence: 0.90,
                        context: "Virtual method dynamic dispatch".to_string(),
                        resolved: false,
                        kind: "virtual_method".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
                if source.contains("std::function") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("function_pointer".to_string()),
                        target_full_path: None,
                        target_pattern: "std::function".to_string(),
                        reference_type: DynamicRefType::FunctionPointer,
                        confidence: 0.85,
                        context: "Function pointer".to_string(),
                        resolved: false,
                        kind: "function_pointer".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
                if source.contains("dlopen") || source.contains("dlsym") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("dlopen".to_string()),
                        target_full_path: None,
                        target_pattern: "dlopen".to_string(),
                        reference_type: DynamicRefType::IPC,
                        confidence: 0.95,
                        context: "Dynamic library loading".to_string(),
                        resolved: false,
                        kind: "dlopen".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
            }
            "csharp" => {
                if source.contains("reflection") || source.contains("Activator.CreateInstance") {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "reflection".to_string(),
                        None,
                        "Activator.CreateInstance".to_string(),
                        0.85,
                        "C# reflection".to_string(),
                    ));
                }
                if source.contains("dynamic") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("dynamic".to_string()),
                        target_full_path: None,
                        target_pattern: "dynamic".to_string(),
                        reference_type: DynamicRefType::DynamicDispatch,
                        confidence: 0.80,
                        context: "C# dynamic dispatch".to_string(),
                        resolved: false,
                        kind: "dynamic_dispatch".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
            }
            "javascript" | "typescript" => {
                if source.contains("import(") {
                    refs.push(DynamicReference::new_dynamic_import(
                        file.path.clone(),
                        "dynamic_import".to_string(),
                        None,
                        "import(".to_string(),
                        0.90,
                        "Dynamic import".to_string(),
                    ));
                }
                if source.contains("eval(") || source.contains("new Function(") {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "eval".to_string(),
                        None,
                        "eval".to_string(),
                        0.95,
                        "Dynamic code execution".to_string(),
                    ));
                }
                if source.contains("require(") {
                    refs.push(DynamicReference::new_dynamic_import(
                        file.path.clone(),
                        "require".to_string(),
                        None,
                        "require".to_string(),
                        0.80,
                        "CommonJS require".to_string(),
                    ));
                }
            }
            "php" => {
                if source.contains("call_user_func") || source.contains("call_user_func_array") {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "call_user_func".to_string(),
                        None,
                        "call_user_func".to_string(),
                        0.90,
                        "PHP dynamic function call".to_string(),
                    ));
                }
                if source.contains("ReflectionClass") || source.contains("ReflectionMethod") {
                    refs.push(DynamicReference::new_reflection(
                        file.path.clone(),
                        None,
                        "ReflectionClass".to_string(),
                        None,
                        "ReflectionClass".to_string(),
                        0.85,
                        "PHP reflection".to_string(),
                    ));
                }
                if source.contains("$container->get") || source.contains("app(") {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "di_container".to_string(),
                        None,
                        "DI container".to_string(),
                        0.90,
                        "PHP dependency injection".to_string(),
                    ));
                }
            }
            "dart" => {
                if source.contains("Function") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some("function_callback".to_string()),
                        target_full_path: None,
                        target_pattern: "Function".to_string(),
                        reference_type: DynamicRefType::FunctionPointer,
                        confidence: 0.80,
                        context: "Dart function reference".to_string(),
                        resolved: false,
                        kind: "function_reference".to_string(),
                        location: None,
                        source: "static_analysis".to_string(),
                    });
                }
                if source.contains("Navigator.push") || source.contains("showDialog") {
                    refs.push(DynamicReference::new_framework(
                        file.path.clone(),
                        None,
                        "navigator".to_string(),
                        None,
                        "Navigator.push".to_string(),
                        0.85,
                        "Flutter navigation".to_string(),
                    ));
                }
            }
            _ => {}
        }

        refs
    }

    fn detect_reflection(
        &self,
        file: &ParsedFile,
        _language: &str,
        source: &str,
        _context: &DetectionContext,
        refs: &mut Vec<DynamicReference>,
    ) {
        if source.contains("reflect") || source.contains("Reflection") {
            refs.push(DynamicReference::new_reflection(
                file.path.clone(),
                None,
                "reflection".to_string(),
                None,
                "reflection".to_string(),
                0.85,
                "Reflection detected".to_string(),
            ));
        }
    }

    fn detect_dynamic_imports(
        &self,
        file: &ParsedFile,
        _language: &str,
        source: &str,
        _context: &DetectionContext,
        refs: &mut Vec<DynamicReference>,
    ) {
        if source.contains("import(")
            || source.contains("__import__")
            || source.contains("importlib")
        {
            refs.push(DynamicReference::new_dynamic_import(
                file.path.clone(),
                "dynamic_import".to_string(),
                None,
                "dynamic_import".to_string(),
                0.85,
                "Dynamic import detected".to_string(),
            ));
        }
    }

    pub fn generate_report(&self, refs: &[DynamicReference]) -> String {
        let mut output = String::new();
        output.push_str("## Dynamic Reference Detection\n\n");

        if refs.is_empty() {
            output.push_str("No dynamic references detected.\n");
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
                    format!(" -> resolved to `{}`", path)
                } else {
                    " unresolved".to_string()
                };
                output.push_str(&format!(
                    "- **{}** (confidence: {:.0}%){}\n  - File: `{}`\n  - Kind: {}\n  - Context: {}\n",
                    r.target_pattern,
                    r.confidence * 100.0,
                    resolved_info,
                    r.source_file,
                    r.kind,
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

pub struct DetectionContext<'a> {
    pub call_graph: &'a CallGraph,
    pub files: &'a [ParsedFile],
    pub name_to_paths: HashMap<String, Vec<String>>,
    pub lower_name_to_paths: HashMap<String, Vec<String>>,
    pub unqualified_to_paths: HashMap<String, Vec<String>>,
}

impl<'a> DetectionContext<'a> {
    pub fn new(call_graph: &'a CallGraph, files: &'a [ParsedFile]) -> Self {
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

        Self {
            call_graph,
            files,
            name_to_paths,
            lower_name_to_paths,
            unqualified_to_paths,
        }
    }

    pub fn resolve_symbol(&self, name: &str) -> Option<String> {
        if let Some(paths) = self.name_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        if let Some(paths) = self.unqualified_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        let lower = name.to_lowercase();
        if let Some(paths) = self.lower_name_to_paths.get(&lower) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        None
    }
}
