// src/analysis/dynamic_refs.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DynamicReference {
    pub source_file: String,
    pub source_function: Option<String>,
    pub target_function: Option<String>,
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
        _call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Vec<DynamicReference> {
        let mut refs = Vec::new();

        for file in files {
            // Check file path for framework indicators
            let is_flask = file.path.contains(".py") && file.source.contains("flask");
            let is_go_reflect = file.path.contains(".go") && file.source.contains("reflect");

            for func_info in &file.functions {
                // 1. AST Decorators / Attributes
                for decorator in &func_info.decorators {
                    let d_lower = decorator.to_lowercase();
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
                            target_pattern: decorator.clone(),
                            reference_type: DynamicRefType::Framework,
                            confidence: 0.95,
                            context: format!("Spring annotation: {}", decorator),
                        });
                    }
                }

                // 2. React JSX / Component detection
                if (file.path.ends_with(".tsx") || file.path.ends_with(".jsx"))
                    && func_info
                        .name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: Some(func_info.name.clone()),
                        target_function: Some(func_info.name.clone()),
                        target_pattern: "JSXComponent".to_string(),
                        reference_type: DynamicRefType::Framework,
                        confidence: 0.90,
                        context: "React Component function".to_string(),
                    });
                }

                // React hooks
                if func_info.name.starts_with("use")
                    && (file.path.ends_with(".tsx") || file.path.ends_with(".jsx"))
                {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: Some(func_info.name.clone()),
                        target_function: Some(func_info.name.clone()),
                        target_pattern: "ReactHook".to_string(),
                        reference_type: DynamicRefType::Framework,
                        confidence: 0.85,
                        context: "React Hook".to_string(),
                    });
                }

                // 3. String-based function dispatcher patterns
                if func_info.body_range.1 > func_info.body_range.0 {
                    let body = &file.source[func_info.body_range.0..func_info.body_range.1];

                    // Python reflection
                    if body.contains("getattr(") || body.contains("importlib") {
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: None,
                            target_pattern: "ReflectionDispatch".to_string(),
                            reference_type: DynamicRefType::Reflection,
                            confidence: 0.85,
                            context: "Reflection or dynamic symbol lookup present in function body"
                                .to_string(),
                        });
                    }

                    // Go reflection
                    if body.contains("reflect.") || body.contains("reflect.ValueOf") {
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: None,
                            target_pattern: "GoReflection".to_string(),
                            reference_type: DynamicRefType::Reflection,
                            confidence: 0.85,
                            context: "Go reflection usage".to_string(),
                        });
                    }

                    // Dynamic imports (JavaScript/TypeScript)
                    if body.contains("import(") || body.contains("require(") {
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: None,
                            target_pattern: "DynamicImport".to_string(),
                            reference_type: DynamicRefType::DynamicImport,
                            confidence: 0.80,
                            context: "Dynamic import or require statement".to_string(),
                        });
                    }
                }
            }

            // 4. Dynamic Import Imports (file-level)
            for import in &file.imports {
                if import.module.contains("dynamic")
                    || import.module.contains("lazy")
                    || import.module.contains("plugin")
                    || import.module.contains("importlib")
                {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_function: None,
                        target_pattern: import.module.clone(),
                        reference_type: DynamicRefType::DynamicImport,
                        confidence: 0.80,
                        context: format!("Dynamic import statement: {}", import.module),
                    });
                }
            }

            // 5. Flask route detection at file level
            if is_flask {
                for func_info in &file.functions {
                    if func_info.decorators.iter().any(|d| d.contains("route")) {
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_function: Some(func_info.name.clone()),
                            target_pattern: "FlaskRoute".to_string(),
                            reference_type: DynamicRefType::Framework,
                            confidence: 0.95,
                            context: "Flask route decorator".to_string(),
                        });
                    }
                }
            }

            // 6. Go reflection at file level
            if is_go_reflect {
                for func_info in &file.functions {
                    if func_info.body_range.1 > func_info.body_range.0 {
                        let body = &file.source[func_info.body_range.0..func_info.body_range.1];
                        if body.contains("reflect.") {
                            refs.push(DynamicReference {
                                source_file: file.path.clone(),
                                source_function: Some(func_info.name.clone()),
                                target_function: None,
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

        refs
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
                output.push_str(&format!(
                    "- 🟢 **{}** (confidence: {:.0}%)\n  - File: `{}`\n  - Context: {}\n",
                    r.target_pattern,
                    r.confidence * 100.0,
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
