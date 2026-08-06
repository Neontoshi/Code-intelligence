// src/analysis/training_data.rs

//! Training data generation for ML-based dead code detection

use crate::graph::call_graph::{CallGraph, FunctionNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    pub function_name: String,
    pub full_path: String,
    pub file: String,
    pub language: String,
    pub features: FunctionFeatures,
    pub label: TrainingLabel,
    pub confidence: f64,
    pub source: String, // "whitelist", "manual", "auto"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrainingLabel {
    Alive,   // Confirmed alive (from whitelist or manual)
    Dead,    // Confirmed dead (removed or verified)
    Unknown, // Not yet classified
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionFeatures {
    // Static features
    pub param_count: usize,
    pub return_count: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub name_length: usize,
    pub starts_with_use: bool,
    pub starts_with_test: bool,
    pub starts_with_bench: bool,
    pub ends_with_test: bool,
    pub contains_trait_impl: bool,

    // Dynamic features
    pub fan_in: usize,
    pub fan_out: usize,
    pub complexity: f64,
    pub call_depth: usize,
    pub is_cycle: bool,

    // Context features
    pub file_extension: String,
    pub is_in_test_file: bool,
    pub is_in_benches: bool,
    pub is_in_meta: bool,
    pub is_in_examples: bool,
    pub is_generated: bool,

    // Name patterns
    pub name_contains_use: bool,
    pub name_contains_test: bool,
    pub name_contains_init: bool,
    pub name_contains_get: bool,
    pub name_contains_set: bool,
    pub name_contains_new: bool,
    pub name_contains_create: bool,
    pub name_contains_build: bool,
    pub name_contains_parse: bool,
    pub name_contains_validate: bool,
    pub name_contains_handle: bool,
    pub name_contains_process: bool,
    pub name_contains_convert: bool,
    pub name_contains_commit: bool,
    pub name_contains_reveal: bool,
    pub name_contains_submit: bool,
    pub name_contains_upload: bool,
    pub name_contains_download: bool,
    pub name_contains_fetch: bool,
    pub name_contains_verify: bool,
    pub name_contains_audit: bool,

    // ⭐ NEW: Type context features
    pub type_name: Option<String>,  // "Allocator", "MemoryPool", etc.
    pub type_path: Option<String>,  // Full path to the type
    pub is_method: bool,            // True if this is a method (has self)
    pub is_trait_impl: bool,        // True if this is a trait implementation
    pub trait_name: Option<String>, // Name of the trait if implemented
    pub is_associated: bool,        // True if associated function (like new())
}

impl FunctionFeatures {
    pub fn from_function(func: &FunctionNode, _call_graph: &CallGraph) -> Self {
        let name_lower = func.name.to_lowercase();

        // Determine if it's a test file
        let is_in_test_file = func.file.contains("/tests/")
            || func.file.contains("/test/")
            || func.file.ends_with("_test.rs")
            || func.file.ends_with("_test.go");

        let is_in_benches = func.file.contains("/benches/");
        let is_in_meta = func.file.contains("/.meta/");
        let is_in_examples = func.file.contains("/examples/");
        let is_generated = func.file.contains(".gen.go")
            || func.file.contains("_gen.go")
            || func.file.contains(".pb.go");

        // File extension
        let file_extension = func.file.split('.').last().unwrap_or("").to_string();

        // Check if it's a trait impl
        let contains_trait_impl = func.trait_impl.is_some();

        // Get call depth
        let call_depth = func.depth;

        // ⭐ NEW: Extract type context
        let type_info = Self::extract_type_info(func);

        Self {
            param_count: func.params.len(),
            return_count: func.returns.len(),
            is_public: func.is_public,
            is_async: func.is_async,
            name_length: func.name.len(),
            starts_with_use: func.name.starts_with("use"),
            starts_with_test: func.name.starts_with("test_") || func.name.starts_with("Test"),
            starts_with_bench: func.name.starts_with("bench_")
                || func.name.starts_with("Benchmark"),
            ends_with_test: func.name.ends_with("_test"),
            contains_trait_impl,
            fan_in: func.fan_in,
            fan_out: func.fan_out,
            complexity: func.complexity,
            call_depth,
            is_cycle: func.is_cycle,
            file_extension,
            is_in_test_file,
            is_in_benches,
            is_in_meta,
            is_in_examples,
            is_generated,
            name_contains_use: name_lower.contains("use"),
            name_contains_test: name_lower.contains("test"),
            name_contains_init: name_lower.contains("init"),
            name_contains_get: name_lower.contains("get"),
            name_contains_set: name_lower.contains("set"),
            name_contains_new: name_lower.contains("new"),
            name_contains_create: name_lower.contains("create"),
            name_contains_build: name_lower.contains("build"),
            name_contains_parse: name_lower.contains("parse"),
            name_contains_validate: name_lower.contains("validate"),
            name_contains_handle: name_lower.contains("handle"),
            name_contains_process: name_lower.contains("process"),
            name_contains_convert: name_lower.contains("convert"),
            name_contains_commit: name_lower.contains("commit"),
            name_contains_reveal: name_lower.contains("reveal"),
            name_contains_submit: name_lower.contains("submit"),
            name_contains_upload: name_lower.contains("upload"),
            name_contains_download: name_lower.contains("download"),
            name_contains_fetch: name_lower.contains("fetch"),
            name_contains_verify: name_lower.contains("verify"),
            name_contains_audit: name_lower.contains("audit"),
            // ⭐ NEW: Type context
            type_name: type_info.type_name,
            type_path: type_info.type_path,
            is_method: type_info.is_method,
            is_trait_impl: type_info.is_trait_impl,
            trait_name: type_info.trait_name,
            is_associated: type_info.is_associated,
        }
    }

    /// ⭐ NEW: Extract type information from function context
    fn extract_type_info(func: &FunctionNode) -> TypeInfo {
        let mut type_name = None;
        let mut type_path = None;
        let mut is_method = false;
        let mut is_trait_impl = false;
        let mut trait_name = None;
        let mut is_associated = false;

        // 1. Check if it's a trait implementation
        if let Some(trait_impl_name) = &func.trait_impl {
            is_trait_impl = true;
            trait_name = Some(trait_impl_name.clone());
        }

        // 2. Check if it's a method (has self parameter)
        if func
            .params
            .first()
            .map(|p| p == "self" || p == "&self" || p == "&mut self")
            .unwrap_or(false)
        {
            is_method = true;
        }

        // 3. Check if it's an associated function (like new())
        if func.name == "new" || func.name == "default" || func.name == "from" {
            is_associated = true;
        }

        // 4. Extract type from file path heuristics
        let file = &func.file;
        let name = &func.name;

        // Look for common patterns
        if let Some(type_name_from_file) = Self::parse_type_from_file(file, name) {
            type_name = Some(type_name_from_file);
            type_path = Some(format!("{}::{}", file, type_name.as_ref().unwrap()));
        }

        // If it's a trait impl, use the trait name as the type name
        if type_name.is_none() && is_trait_impl {
            type_name = trait_name.clone();
            type_path = trait_name.clone().map(|t| format!("trait::{}", t));
        }

        TypeInfo {
            type_name,
            type_path,
            is_method,
            is_trait_impl,
            trait_name,
            is_associated,
        }
    }

    /// Parse type name from file path heuristics
    fn parse_type_from_file(file: &str, func_name: &str) -> Option<String> {
        // Check if the file is alloc.rs
        if file.ends_with("alloc.rs") {
            if func_name == "reset" {
                if file.contains("Allocator") {
                    return Some("Allocator".to_string());
                } else if file.contains("MemoryPool") {
                    return Some("MemoryPool".to_string());
                }
            }
            return None;
        }

        // Graph types
        if file.contains("graph/")
            && (func_name == "index" || func_name == "node_count" || func_name == "edge_count")
        {
            if file.contains("call_graph") {
                return Some("CallGraph".to_string());
            } else if file.contains("dependency_graph") {
                return Some("DependencyGraph".to_string());
            } else if file.contains("project_graph") {
                return Some("ProjectGraph".to_string());
            } else if file.contains("type_graph") {
                return Some("TypeGraph".to_string());
            }
        }

        // LLM providers
        if file.contains("llm/providers/") {
            if file.contains("ollama") {
                return Some("OllamaProvider".to_string());
            } else if file.contains("openai") {
                return Some("OpenAIProvider".to_string());
            } else if file.contains("anthropic") {
                return Some("AnthropicProvider".to_string());
            } else if file.contains("mock") {
                return Some("MockProvider".to_string());
            }
        }

        None
    }

    /// Convert features to a numeric vector for ML
    pub fn to_feature_vector(&self) -> Vec<f64> {
        vec![
            // Original features
            self.param_count as f64 / 10.0,
            self.return_count as f64 / 5.0,
            if self.is_public { 1.0 } else { 0.0 },
            if self.is_async { 1.0 } else { 0.0 },
            self.name_length as f64 / 50.0,
            if self.starts_with_use { 1.0 } else { 0.0 },
            if self.starts_with_test { 1.0 } else { 0.0 },
            if self.starts_with_bench { 1.0 } else { 0.0 },
            if self.ends_with_test { 1.0 } else { 0.0 },
            if self.contains_trait_impl { 1.0 } else { 0.0 },
            self.fan_in as f64 / 50.0,
            self.fan_out as f64 / 50.0,
            self.complexity / 50.0,
            self.call_depth as f64 / 10.0,
            if self.is_cycle { 1.0 } else { 0.0 },
            if self.is_in_test_file { 1.0 } else { 0.0 },
            if self.is_in_benches { 1.0 } else { 0.0 },
            if self.is_in_meta { 1.0 } else { 0.0 },
            if self.is_in_examples { 1.0 } else { 0.0 },
            if self.is_generated { 1.0 } else { 0.0 },
            if self.name_contains_use { 1.0 } else { 0.0 },
            if self.name_contains_test { 1.0 } else { 0.0 },
            if self.name_contains_init { 1.0 } else { 0.0 },
            if self.name_contains_get { 1.0 } else { 0.0 },
            if self.name_contains_set { 1.0 } else { 0.0 },
            if self.name_contains_new { 1.0 } else { 0.0 },
            if self.name_contains_create { 1.0 } else { 0.0 },
            if self.name_contains_build { 1.0 } else { 0.0 },
            if self.name_contains_parse { 1.0 } else { 0.0 },
            if self.name_contains_validate {
                1.0
            } else {
                0.0
            },
            if self.name_contains_handle { 1.0 } else { 0.0 },
            if self.name_contains_process { 1.0 } else { 0.0 },
            if self.name_contains_convert { 1.0 } else { 0.0 },
            // ⭐ NEW: Type context features
            if self.is_method { 1.0 } else { 0.0 },
            if self.is_trait_impl { 1.0 } else { 0.0 },
            if self.is_associated { 1.0 } else { 0.0 },
            self.type_name
                .as_ref()
                .map(|s| s.len() as f64 / 20.0)
                .unwrap_or(0.0),
            self.trait_name
                .as_ref()
                .map(|s| s.len() as f64 / 20.0)
                .unwrap_or(0.0),
            if self.type_name == self.trait_name {
                1.0
            } else {
                0.0
            },
        ]
    }
}

/// ⭐ NEW: Type information container
#[derive(Debug, Clone)]
struct TypeInfo {
    type_name: Option<String>,
    type_path: Option<String>,
    is_method: bool,
    is_trait_impl: bool,
    trait_name: Option<String>,
    is_associated: bool,
}

impl TrainingExample {
    pub fn new_alive(func: &FunctionNode, call_graph: &CallGraph) -> Self {
        Self {
            function_name: func.name.clone(),
            full_path: func.full_path.clone(),
            file: func.file.clone(),
            language: Self::detect_language(&func.file),
            features: FunctionFeatures::from_function(func, call_graph),
            label: TrainingLabel::Alive,
            confidence: 0.95,
            source: "whitelist".to_string(),
        }
    }

    pub fn new_dead(func: &FunctionNode, call_graph: &CallGraph) -> Self {
        Self {
            function_name: func.name.clone(),
            full_path: func.full_path.clone(),
            file: func.file.clone(),
            language: Self::detect_language(&func.file),
            features: FunctionFeatures::from_function(func, call_graph),
            label: TrainingLabel::Dead,
            confidence: 0.95,
            source: "analysis".to_string(),
        }
    }

    pub fn detect_language(file: &str) -> String {
        if file.ends_with(".rs") {
            "rust".to_string()
        } else if file.ends_with(".go") {
            "go".to_string()
        } else if file.ends_with(".py") {
            "python".to_string()
        } else if file.ends_with(".js") || file.ends_with(".jsx") {
            "javascript".to_string()
        } else if file.ends_with(".ts") || file.ends_with(".tsx") {
            "typescript".to_string()
        } else if file.ends_with(".java") {
            "java".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

/// Training data collection
#[derive(Debug, Clone, Default)]
pub struct TrainingDataCollector {
    pub examples: Vec<TrainingExample>,
    pub stats: TrainingStats,
}

#[derive(Debug, Clone, Default)]
pub struct TrainingStats {
    pub total_functions: usize,
    pub alive_count: usize,
    pub dead_count: usize,
    pub unknown_count: usize,
    pub by_language: HashMap<String, usize>,
}

impl TrainingDataCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect_from_analysis(
        &mut self,
        call_graph: &CallGraph,
        is_whitelisted_fn: impl Fn(&FunctionNode) -> bool,
        is_dead_fn: impl Fn(&FunctionNode) -> bool,
    ) {
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Check if it's a test function
            let is_test_function = func.name.starts_with("test_")
                || func.name.starts_with("Test")
                || func.name.starts_with("bench_")
                || func.name.starts_with("Benchmark")
                || func.file.contains("/tests/")
                || func.file.ends_with("_test.rs")
                || func.file.ends_with("_test.go");

            let (label, confidence) = if is_test_function {
                (TrainingLabel::Alive, 0.95)
            } else if is_whitelisted_fn(func) {
                (TrainingLabel::Alive, 0.95)
            } else if is_dead_fn(func) {
                (TrainingLabel::Dead, 0.85)
            } else {
                (TrainingLabel::Unknown, 0.0)
            };

            let example = TrainingExample {
                function_name: func.name.clone(),
                full_path: func.full_path.clone(),
                file: func.file.clone(),
                language: TrainingExample::detect_language(&func.file),
                features: FunctionFeatures::from_function(func, call_graph),
                label: label.clone(),
                confidence,
                source: "auto".to_string(),
            };

            self.examples.push(example.clone());
            self.update_stats(&example);
        }
    }

    fn update_stats(&mut self, example: &TrainingExample) {
        self.stats.total_functions += 1;
        *self
            .stats
            .by_language
            .entry(example.language.clone())
            .or_insert(0) += 1;

        match example.label {
            TrainingLabel::Alive => self.stats.alive_count += 1,
            TrainingLabel::Dead => self.stats.dead_count += 1,
            TrainingLabel::Unknown => self.stats.unknown_count += 1,
        }
    }

    /// Export to JSONL format (one JSON per line)
    pub fn to_jsonl(&self) -> String {
        self.examples
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Export to pretty JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.examples)
    }

    pub fn get_alive_examples(&self) -> Vec<&TrainingExample> {
        self.examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Alive)
            .collect()
    }

    pub fn get_dead_examples(&self) -> Vec<&TrainingExample> {
        self.examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Dead)
            .collect()
    }
}
