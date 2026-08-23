// src/analysis/dynamic_refs.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

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

        // Build a name-to-full-path index for symbol resolution
        let mut name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            name_to_paths
                .entry(func.name.clone())
                .or_default()
                .push(func.full_path.clone());
        }

        // Also index by lowercase name for case-insensitive matching
        let mut lower_name_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        for (name, paths) in &name_to_paths {
            lower_name_to_paths
                .entry(name.to_lowercase())
                .or_default()
                .extend(paths.clone());
        }

        for file in files {
            // Check file path for framework indicators
            let is_flask = file.path.contains(".py") && file.source.contains("flask");
            let is_go_reflect = file.path.contains(".go") && file.source.contains("reflect");

            for func_info in &file.functions {
                // 1. AST Decorators / Attributes - Now resolves symbols
                for decorator in &func_info.decorators {
                    let d_lower = decorator.to_lowercase();

                    // Try to resolve decorator to a specific function
                    let resolved_path =
                        Self::resolve_symbol(decorator, &name_to_paths, &lower_name_to_paths);

                    if d_lower.contains("route")
                        || d_lower.contains("get")
                        || d_lower.contains("post")
                        || d_lower.contains("put")
                        || d_lower.contains("delete")
                        || d_lower.contains("mapping")
                        || d_lower.contains("controller")
                        || d_lower.contains("injectable")
                        || d_lower.contains("app.route")
                        || d_lower.contains("blueprint")
                    {
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: Some(func_info.name.clone()),
                            target_full_path: resolved_path.clone(),
                            target_pattern: decorator.clone(),
                            reference_type: DynamicRefType::Framework,
                            confidence: 0.95,
                            context: format!("Decorated endpoint: {}", decorator),
                        });
                    }

                    // Spring annotations
                    if d_lower.contains("getmapping")
                        || d_lower.contains("postmapping")
                        || d_lower.contains("putmapping")
                        || d_lower.contains("deletemapping")
                        || d_lower.contains("requestmapping")
                        || d_lower.contains("restcontroller")
                        || d_lower.contains("service")
                        || d_lower.contains("repository")
                        || d_lower.contains("component")
                    {
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: Some(func_info.name.clone()),
                            target_full_path: resolved_path,
                            target_pattern: decorator.clone(),
                            reference_type: DynamicRefType::Framework,
                            confidence: 0.95,
                            context: format!("Spring annotation: {}", decorator),
                        });
                    }
                }

                // 2. React JSX / Component detection with symbol resolution
                if (file.path.ends_with(".tsx") || file.path.ends_with(".jsx"))
                    && func_info
                        .name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                {
                    let resolved =
                        Self::resolve_symbol(&func_info.name, &name_to_paths, &lower_name_to_paths);
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: Some(func_info.name.clone()),
                        target_function: Some(func_info.name.clone()),
                        target_full_path: resolved,
                        target_pattern: "JSXComponent".to_string(),
                        reference_type: DynamicRefType::Framework,
                        confidence: 0.90,
                        context: "React Component function".to_string(),
                    });
                }

                // React hooks with symbol resolution
                if func_info.name.starts_with("use")
                    && (file.path.ends_with(".tsx") || file.path.ends_with(".jsx"))
                {
                    let resolved =
                        Self::resolve_symbol(&func_info.name, &name_to_paths, &lower_name_to_paths);
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: Some(func_info.name.clone()),
                        target_function: Some(func_info.name.clone()),
                        target_full_path: resolved,
                        target_pattern: "ReactHook".to_string(),
                        reference_type: DynamicRefType::Framework,
                        confidence: 0.85,
                        context: "React Hook".to_string(),
                    });
                }

                // 3. String-based function dispatcher patterns - now resolves targets
                if func_info.body_range.1 > func_info.body_range.0 {
                    let body = &file.source[func_info.body_range.0..func_info.body_range.1];

                    // Python reflection with symbol extraction
                    if body.contains("getattr(") || body.contains("importlib") {
                        // ⭐ NEW: Try to extract the actual function name from getattr
                        let target_name = Self::extract_getattr_target(body);
                        let resolved = target_name.as_ref().and_then(|name| {
                            Self::resolve_symbol(name, &name_to_paths, &lower_name_to_paths)
                        });
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: target_name,
                            target_full_path: resolved,
                            target_pattern: "ReflectionDispatch".to_string(),
                            reference_type: DynamicRefType::Reflection,
                            confidence: 0.85,
                            context: "Reflection or dynamic symbol lookup present in function body"
                                .to_string(),
                        });
                    }

                    // Go reflection with symbol extraction
                    if body.contains("reflect.") || body.contains("reflect.ValueOf") {
                        // Try to extract the method name from reflection
                        let target_name = Self::extract_reflect_target(body);
                        let resolved = target_name.as_ref().and_then(|name| {
                            Self::resolve_symbol(name, &name_to_paths, &lower_name_to_paths)
                        });
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: target_name,
                            target_full_path: resolved,
                            target_pattern: "GoReflection".to_string(),
                            reference_type: DynamicRefType::Reflection,
                            confidence: 0.85,
                            context: "Go reflection usage".to_string(),
                        });
                    }

                    // Dynamic imports with symbol resolution
                    if body.contains("import(") || body.contains("require(") {
                        // Try to extract the module path from import
                        let target_name = Self::extract_import_target(body);
                        let resolved = target_name.as_ref().and_then(|name| {
                            Self::resolve_symbol(name, &name_to_paths, &lower_name_to_paths)
                        });
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: target_name,
                            target_full_path: resolved,
                            target_pattern: "DynamicImport".to_string(),
                            reference_type: DynamicRefType::DynamicImport,
                            confidence: 0.80,
                            context: "Dynamic import or require statement".to_string(),
                        });
                    }
                }
            }

            // 4. Dynamic Import Imports (file-level) with symbol resolution
            for import in &file.imports {
                if import.module.contains("dynamic")
                    || import.module.contains("lazy")
                    || import.module.contains("plugin")
                    || import.module.contains("importlib")
                {
                    let resolved =
                        Self::resolve_symbol(&import.module, &name_to_paths, &lower_name_to_paths);
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: Some(import.module.clone()),
                        target_full_path: resolved,
                        target_pattern: import.module.clone(),
                        reference_type: DynamicRefType::DynamicImport,
                        confidence: 0.80,
                        context: format!("Dynamic import statement: {}", import.module),
                    });
                }
            }

            // 5. Flask route detection with symbol resolution
            if is_flask {
                for func_info in &file.functions {
                    if func_info.decorators.iter().any(|d| d.contains("route")) {
                        let resolved = Self::resolve_symbol(
                            &func_info.name,
                            &name_to_paths,
                            &lower_name_to_paths,
                        );
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: Some(func_info.name.clone()),
                            target_full_path: resolved,
                            target_pattern: "FlaskRoute".to_string(),
                            reference_type: DynamicRefType::Framework,
                            confidence: 0.95,
                            context: "Flask route decorator".to_string(),
                        });
                    }
                }
            }

            // 6. Go reflection with symbol resolution
            if is_go_reflect {
                for func_info in &file.functions {
                    if func_info.body_range.1 > func_info.body_range.0 {
                        let body = &file.source[func_info.body_range.0..func_info.body_range.1];
                        if body.contains("reflect.") {
                            let target_name = Self::extract_reflect_target(body);
                            let resolved = target_name.as_ref().and_then(|name| {
                                Self::resolve_symbol(name, &name_to_paths, &lower_name_to_paths)
                            });
                            refs.push(DynamicReference {
                                source_file: file.path.clone(),
                                source_function: Some(func_info.name.clone()),
                                target_function: target_name,
                                target_full_path: resolved,
                                target_pattern: "GoReflection".to_string(),
                                reference_type: DynamicRefType::Reflection,
                                confidence: 0.85,
                                context: "Go reflection usage".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Remove duplicate references (same source, same target_full_path)
        let mut seen = std::collections::HashSet::new();
        refs.retain(|r| {
            let key = (
                r.source_file.clone(),
                r.target_full_path.clone().unwrap_or_default(),
                r.reference_type.clone(),
            );
            if seen.contains(&key) {
                false
            } else {
                seen.insert(key);
                true
            }
        });

        refs
    }

    fn resolve_symbol(
        name: &str,
        name_to_paths: &HashMap<String, Vec<String>>,
        lower_name_to_paths: &HashMap<String, Vec<String>>,
    ) -> Option<String> {
        // Try exact match first
        if let Some(paths) = name_to_paths.get(name) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        // Try lowercase match
        let lower = name.to_lowercase();
        if let Some(paths) = lower_name_to_paths.get(&lower) {
            if paths.len() == 1 {
                return Some(paths[0].clone());
            }
        }

        // Try to find a function that ends with this name
        for (full_name, paths) in name_to_paths {
            if full_name.ends_with(name) || full_name.ends_with(&format!("::{}", name)) {
                if paths.len() == 1 {
                    return Some(paths[0].clone());
                }
            }
        }

        None
    }
    fn extract_getattr_target(body: &str) -> Option<String> {
        // Look for patterns like getattr(module, "function_name")
        if let Some(start) = body.find("getattr(") {
            let after = &body[start + 8..];
            if let Some(comma_pos) = after.find(',') {
                let after_comma = &after[comma_pos + 1..];
                // Find the quoted string
                if let Some(quote_start) = after_comma.find('"') {
                    let after_quote = &after_comma[quote_start + 1..];
                    if let Some(quote_end) = after_quote.find('"') {
                        return Some(after_quote[..quote_end].to_string());
                    }
                }
                if let Some(quote_start) = after_comma.find('\'') {
                    let after_quote = &after_comma[quote_start + 1..];
                    if let Some(quote_end) = after_quote.find('\'') {
                        return Some(after_quote[..quote_end].to_string());
                    }
                }
            }
        }
        None
    }
    fn extract_reflect_target(body: &str) -> Option<String> {
        // Look for patterns like MethodByName("method_name")
        if let Some(start) = body.find("MethodByName(") {
            let after = &body[start + 13..];
            if let Some(quote_start) = after.find('"') {
                let after_quote = &after[quote_start + 1..];
                if let Some(quote_end) = after_quote.find('"') {
                    return Some(after_quote[..quote_end].to_string());
                }
            }
            if let Some(quote_start) = after.find('\'') {
                let after_quote = &after[quote_start + 1..];
                if let Some(quote_end) = after_quote.find('\'') {
                    return Some(after_quote[..quote_end].to_string());
                }
            }
        }
        None
    }

    fn extract_import_target(body: &str) -> Option<String> {
        // Look for patterns like import("module") or require("module")
        let patterns = ["import(", "require("];
        for pattern in patterns {
            if let Some(start) = body.find(pattern) {
                let after = &body[start + pattern.len()..];
                if let Some(quote_start) = after.find('"') {
                    let after_quote = &after[quote_start + 1..];
                    if let Some(quote_end) = after_quote.find('"') {
                        let module = after_quote[..quote_end].to_string();
                        // Extract just the filename/function name from the path
                        if let Some(last_part) = module.split('/').last() {
                            if let Some(func_name) = last_part.split('.').next() {
                                return Some(func_name.to_string());
                            }
                            return Some(last_part.to_string());
                        }
                        return Some(module);
                    }
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

impl Default for DynamicRefDetector {
    fn default() -> Self {
        Self::new()
    }
}
