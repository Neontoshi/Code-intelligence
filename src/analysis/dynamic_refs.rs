// src/analysis/dynamic_refs.rs

//! Dynamic reference detection for code that static analysis misses
//!
//! This module detects:
//! - Reflection usage
//! - Callbacks and event handlers
//! - Framework registration patterns
//! - Dynamic imports
//! - Dependency injection

use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

// ============================================================================
// Dynamic Reference Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct DynamicReference {
    pub source_file: String,
    pub source_function: Option<String>,
    pub target_pattern: String, // Pattern that matches the target
    pub reference_type: DynamicRefType,
    pub confidence: f64, // 0.0 - 1.0
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
// ============================================================================
// Detection Engine
// ============================================================================

pub struct DynamicRefDetector {
    // Language-specific patterns
    rust_patterns: Vec<RefPattern>,
    js_patterns: Vec<RefPattern>,
    python_patterns: Vec<RefPattern>,
    go_patterns: Vec<RefPattern>,
    java_patterns: Vec<RefPattern>,
}

impl DynamicRefDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            rust_patterns: Vec::new(),
            js_patterns: Vec::new(),
            python_patterns: Vec::new(),
            go_patterns: Vec::new(),
            java_patterns: Vec::new(),
        };

        detector.init_patterns();
        detector
    }

    fn init_patterns(&mut self) {
        // ================================================================
        // Rust Patterns
        // ================================================================
        self.rust_patterns.push(RefPattern {
            name: "Any trait".to_string(),
            pattern: "Any".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.7,
            language: "rust".to_string(),
        });

        self.rust_patterns.push(RefPattern {
            name: "TypeId".to_string(),
            pattern: "TypeId::of".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.8,
            language: "rust".to_string(),
        });

        self.rust_patterns.push(RefPattern {
            name: "Foreign function".to_string(),
            pattern: "extern \"C\"".to_string(),
            ref_type: DynamicRefType::Framework,
            confidence: 0.6,
            language: "rust".to_string(),
        });

        // ================================================================
        // JavaScript/TypeScript Patterns
        // ================================================================
        self.js_patterns.push(RefPattern {
            name: "Dynamic import".to_string(),
            pattern: "import(".to_string(),
            ref_type: DynamicRefType::DynamicImport,
            confidence: 0.9,
            language: "js".to_string(),
        });

        self.js_patterns.push(RefPattern {
            name: "Require".to_string(),
            pattern: "require(".to_string(),
            ref_type: DynamicRefType::DynamicImport,
            confidence: 0.8,
            language: "js".to_string(),
        });

        self.js_patterns.push(RefPattern {
            name: "Event listener".to_string(),
            pattern: "addEventListener(".to_string(),
            ref_type: DynamicRefType::Callback,
            confidence: 0.8,
            language: "js".to_string(),
        });

        self.js_patterns.push(RefPattern {
            name: "Reflect API".to_string(),
            pattern: "Reflect.".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.7,
            language: "js".to_string(),
        });

        self.js_patterns.push(RefPattern {
            name: "Property access".to_string(),
            pattern: "[]".to_string(),
            ref_type: DynamicRefType::StringDispatch,
            confidence: 0.5,
            language: "js".to_string(),
        });

        // ================================================================
        // Python Patterns
        // ================================================================
        self.python_patterns.push(RefPattern {
            name: "Getattr".to_string(),
            pattern: "getattr(".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.8,
            language: "python".to_string(),
        });

        self.python_patterns.push(RefPattern {
            name: "Setattr".to_string(),
            pattern: "setattr(".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.8,
            language: "python".to_string(),
        });

        self.python_patterns.push(RefPattern {
            name: "Importlib".to_string(),
            pattern: "importlib".to_string(),
            ref_type: DynamicRefType::DynamicImport,
            confidence: 0.8,
            language: "python".to_string(),
        });

        self.python_patterns.push(RefPattern {
            name: "Decorator registration".to_string(),
            pattern: "@".to_string(),
            ref_type: DynamicRefType::Framework,
            confidence: 0.6,
            language: "python".to_string(),
        });

        // ================================================================
        // Go Patterns
        // ================================================================
        self.go_patterns.push(RefPattern {
            name: "Reflection".to_string(),
            pattern: "reflect.".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.8,
            language: "go".to_string(),
        });

        self.go_patterns.push(RefPattern {
            name: "Interface assertion".to_string(),
            pattern: ".(type)".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.6,
            language: "go".to_string(),
        });

        // ================================================================
        // Java Patterns
        // ================================================================
        self.java_patterns.push(RefPattern {
            name: "Reflection".to_string(),
            pattern: "Class.forName".to_string(),
            ref_type: DynamicRefType::Reflection,
            confidence: 0.9,
            language: "java".to_string(),
        });

        self.java_patterns.push(RefPattern {
            name: "Annotation".to_string(),
            pattern: "@".to_string(),
            ref_type: DynamicRefType::Framework,
            confidence: 0.5,
            language: "java".to_string(),
        });
    }

    /// Detect dynamic references in a project
    pub fn detect_all(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Vec<DynamicReference> {
        let mut refs = Vec::new();

        for file in files {
            let language = file.language.as_str();
            let patterns = self.get_patterns_for_language(language);

            // Check function bodies for patterns
            for func_info in &file.functions {
                // Find the function in the call graph
                let _func = call_graph
                    .node_indices()
                    .find(|idx| call_graph[*idx].name == func_info.name)
                    .map(|idx| &call_graph[idx]);

                // Check source code for patterns
                let source = &file.source;
                for pattern in patterns {
                    if source.contains(&pattern.pattern) {
                        refs.push(DynamicReference {
                            source_file: file.path.clone(),
                            source_function: Some(func_info.name.clone()),
                            target_pattern: pattern.pattern.clone(),
                            reference_type: pattern.ref_type.clone(),
                            confidence: pattern.confidence,
                            context: format!("Found '{}' in file", pattern.pattern),
                        });
                    }
                }

                // Special: Check for framework-specific patterns
                self.detect_framework_patterns(&mut refs, file, func_info);
            }

            // Check imports for dynamic patterns
            for import in &file.imports {
                if self.is_dynamic_import(&import.module) {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: None,
                        target_pattern: import.module.clone(),
                        reference_type: DynamicRefType::DynamicImport,
                        confidence: 0.7,
                        context: format!("Dynamic import: {}", import.module),
                    });
                }
            }
        }

        refs
    }

    fn get_patterns_for_language(&self, language: &str) -> &[RefPattern] {
        match language {
            "Rust" => &self.rust_patterns,
            "JavaScript" | "TypeScript" => &self.js_patterns,
            "Python" => &self.python_patterns,
            "Go" => &self.go_patterns,
            "Java" => &self.java_patterns,
            _ => &[],
        }
    }

    fn is_dynamic_import(&self, module: &str) -> bool {
        module.contains("dynamic_import") || module.contains("lazy_") || module.contains("plugin")
    }

    fn detect_framework_patterns(
        &self,
        refs: &mut Vec<DynamicReference>,
        file: &ParsedFile,
        func_info: &crate::parser::tree_sitter::FunctionInfo,
    ) {
        // React/JSX components
        if file.path.ends_with(".tsx") || file.path.ends_with(".jsx") {
            if func_info
                .name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                refs.push(DynamicReference {
                    source_file: file.path.clone(),
                    source_function: Some(func_info.name.clone()),
                    target_pattern: "ReactComponent".to_string(),
                    reference_type: DynamicRefType::Framework,
                    confidence: 0.9,
                    context: "React component (capitalized)".to_string(),
                });
            }
        }

        // Spring Boot controllers
        if file.path.contains("Controller") && func_info.name.contains("handle") {
            refs.push(DynamicReference {
                source_file: file.path.clone(),
                source_function: Some(func_info.name.clone()),
                target_pattern: "SpringHandler".to_string(),
                reference_type: DynamicRefType::Framework,
                confidence: 0.8,
                context: "Spring controller handler".to_string(),
            });
        }

        // Python Flask/FastAPI routes
        if file.path.contains("routes") || file.path.contains("handlers") {
            if func_info.doc_comment.is_some() {
                let doc = func_info.doc_comment.as_ref().unwrap();
                if doc.contains("@app.route") || doc.contains("@router.") {
                    refs.push(DynamicReference {
                        source_file: file.path.clone(),
                        source_function: Some(func_info.name.clone()),
                        target_pattern: "RouteHandler".to_string(),
                        reference_type: DynamicRefType::Framework,
                        confidence: 0.9,
                        context: "Flask/FastAPI route handler".to_string(),
                    });
                }
            }
        }
    }

    /// Generate a report of dynamic references
    pub fn generate_report(&self, refs: &[DynamicReference]) -> String {
        let mut output = String::new();

        output.push_str("## 🔄 Dynamic Reference Detection\n\n");

        if refs.is_empty() {
            output.push_str("✅ No dynamic references detected.\n");
            return output;
        }

        output.push_str(&format!("Found **{}** dynamic references:\n\n", refs.len()));

        // Group by type
        let mut by_type: HashMap<DynamicRefType, Vec<&DynamicReference>> = HashMap::new();
        for r in refs {
            by_type.entry(r.reference_type.clone()).or_default().push(r);
        }

        for (ref_type, refs_by_type) in &by_type {
            output.push_str(&format!("### {:?} ({})\n\n", ref_type, refs_by_type.len()));

            for r in refs_by_type {
                let confidence_emoji = if r.confidence > 0.8 {
                    "🟢"
                } else if r.confidence > 0.5 {
                    "🟡"
                } else {
                    "🔴"
                };
                output.push_str(&format!(
                    "- {} **{}** (confidence: {:.0}%)\n",
                    confidence_emoji,
                    r.target_pattern,
                    r.confidence * 100.0
                ));
                if let Some(func) = &r.source_function {
                    output.push_str(&format!("  - Function: `{}`\n", func));
                }
                output.push_str(&format!("  - File: `{}`\n", r.source_file));
                output.push_str(&format!("  - Context: {}\n", r.context));
            }
            output.push('\n');
        }

        output
    }
}

// ============================================================================
// Pattern Definition
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RefPattern {
    name: String,
    pattern: String,
    ref_type: DynamicRefType,
    confidence: f64,
    language: String,
}

// ============================================================================
// Integration with Verdict Engine
// ============================================================================

/// Check if a function is referenced dynamically
pub fn is_dynamically_referenced(func: &FunctionNode, dynamic_refs: &[DynamicReference]) -> bool {
    dynamic_refs.iter().any(|r| {
        r.source_function
            .as_ref()
            .map(|f| f == &func.name)
            .unwrap_or(false)
            || r.target_pattern.contains(&func.name)
    })
}

/// Get dynamic references for a specific function
pub fn get_references_for_function<'a>(
    func: &FunctionNode,
    dynamic_refs: &'a [DynamicReference],
) -> Vec<&'a DynamicReference> {
    dynamic_refs
        .iter()
        .filter(|r| {
            r.source_function
                .as_ref()
                .map(|f| f == &func.name)
                .unwrap_or(false)
                || r.target_pattern.contains(&func.name)
        })
        .collect()
}

impl Default for DynamicRefDetector {
    fn default() -> Self {
        Self::new()
    }
}
