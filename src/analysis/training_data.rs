// src/analysis/training_data.rs

//! Training data generation for ML-based dead code detection

use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::ml::feature_schema::{FeatureVectorBuilder, FEATURE_SCHEMA};
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
    pub source: String,
    // ⭐ NEW METADATA FIELDS
    pub repository_id: Option<String>,
    pub commit_hash: Option<String>,
    pub dataset_split: Option<String>, // "train", "val", "test"
    pub label_reason: Option<String>, // "root", "has_callers", "test_function", "library_export", "truly_dead"
    pub label_version: Option<u32>,   // Version of labeling logic
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrainingLabel {
    Alive,
    Dead,
    Unknown,
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

    // HASH FIELDS (for deduplication)
    pub signature_hash: String,
    pub body_hash: String,

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

    // Type context features
    pub type_name: Option<String>,
    pub type_path: Option<String>,
    pub is_method: bool,
    pub is_trait_impl: bool,
    pub trait_name: Option<String>,
    pub is_associated: bool,
}

impl FunctionFeatures {
    pub fn from_function(func: &FunctionNode, _call_graph: &CallGraph) -> Self {
        let name_lower = func.name.to_lowercase();

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

        let file_extension = func.file.split('.').last().unwrap_or("").to_string();
        let contains_trait_impl = func.trait_impl.is_some();
        let call_depth = func.depth;

        let type_info = Self::extract_type_info(func);

        use crate::optimize::dedup::core::compute_signature_hash;

        // For body_hash, we'd need source, but we can use a placeholder
        // The deduplication function will need to compute it properly
        let sig_hash = compute_signature_hash(func);

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
            signature_hash: sig_hash.clone(),
            body_hash: sig_hash.clone(),
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
            type_name: type_info.type_name,
            type_path: type_info.type_path,
            is_method: type_info.is_method,
            is_trait_impl: type_info.is_trait_impl,
            trait_name: type_info.trait_name,
            is_associated: type_info.is_associated,
        }
    }

    fn extract_type_info(func: &FunctionNode) -> TypeInfo {
        let is_trait_impl = func.trait_impl.is_some();
        let trait_name = func.trait_impl.clone();

        let is_method = func
            .params
            .first()
            .map(|p| p == "self" || p == "&self" || p == "&mut self")
            .unwrap_or(false);

        let is_associated = matches!(func.name.as_str(), "new" | "default" | "from");

        // full_path is built as "file::Container::function" when the function
        // lives inside an impl/container block, or "file::function" otherwise
        // (see CallGraphBuilder::build). File paths use '/' not '::', so this
        // split is unambiguous and works for every function in the codebase —
        // not just the handful of files a hardcoded list happened to cover.
        let mut type_name = None;
        let mut type_path = None;
        let segments: Vec<&str> = func.full_path.rsplitn(3, "::").collect();
        if segments.len() == 3 {
            let container = segments[1];
            let file = segments[2];
            type_name = Some(container.to_string());
            type_path = Some(format!("{}::{}", file, container));
        }

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
    /// Convert features to a numeric vector using the schema
    pub fn to_feature_vector(&self) -> Vec<f64> {
        let mut builder = FeatureVectorBuilder::new();

        // Graph features (4)
        builder
            .push_normalized(self.fan_in as f64, 50.0)
            .push_normalized(self.fan_out as f64, 50.0)
            .push_normalized(self.call_depth as f64, 10.0)
            .push_bool(self.is_cycle);

        // Signature features (4)
        builder
            .push_normalized(self.param_count as f64, 10.0)
            .push_normalized(self.return_count as f64, 5.0)
            .push_bool(self.is_public)
            .push_bool(self.is_async);

        // Complexity (1)
        builder.push_normalized(self.complexity, 50.0);

        // Name contains (21)
        builder
            .push_bool(self.name_contains_use)
            .push_bool(self.name_contains_test)
            .push_bool(self.name_contains_init)
            .push_bool(self.name_contains_get)
            .push_bool(self.name_contains_set)
            .push_bool(self.name_contains_new)
            .push_bool(self.name_contains_create)
            .push_bool(self.name_contains_build)
            .push_bool(self.name_contains_parse)
            .push_bool(self.name_contains_validate)
            .push_bool(self.name_contains_handle)
            .push_bool(self.name_contains_process)
            .push_bool(self.name_contains_convert)
            .push_bool(self.name_contains_commit)
            .push_bool(self.name_contains_reveal)
            .push_bool(self.name_contains_submit)
            .push_bool(self.name_contains_upload)
            .push_bool(self.name_contains_download)
            .push_bool(self.name_contains_fetch)
            .push_bool(self.name_contains_verify)
            .push_bool(self.name_contains_audit);

        // Name starts/ends (5)
        builder
            .push_bool(self.starts_with_use)
            .push_bool(self.starts_with_test)
            .push_bool(self.starts_with_bench)
            .push_bool(self.ends_with_test)
            .push_normalized(self.name_length as f64, 50.0);

        // File context (5)
        builder
            .push_bool(self.is_in_test_file)
            .push_bool(self.is_in_benches)
            .push_bool(self.is_in_meta)
            .push_bool(self.is_in_examples)
            .push_bool(self.is_generated);

        // Type context (6)
        builder
            .push_bool(self.is_method)
            .push_bool(self.is_trait_impl)
            .push_bool(self.is_associated)
            .push_opt(self.type_name.as_ref().map(|s| s.len() as f64 / 20.0), 0.0)
            .push_opt(self.trait_name.as_ref().map(|s| s.len() as f64 / 20.0), 0.0)
            .push_bool(self.type_name == self.trait_name);

        let features = builder.build();

        // Validate against schema in debug mode
        if cfg!(debug_assertions) {
            if let Err(e) = FEATURE_SCHEMA.validate_vector(&features) {
                panic!("Feature vector validation failed: {}", e);
            }
        }

        features
    }
}

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
            repository_id: None,
            commit_hash: None,
            dataset_split: None,
            label_reason: Some("whitelist".to_string()),
            label_version: Some(1),
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
            repository_id: None,
            commit_hash: None,
            dataset_split: None,
            label_reason: Some("analysis".to_string()),
            label_version: Some(1),
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

            // Determine the label reason based on how we classified it
            let label_reason = if is_test_function {
                "test_function".to_string()
            } else if is_whitelisted_fn(func) {
                "whitelist".to_string()
            } else if is_dead_fn(func) {
                "truly_dead".to_string()
            } else {
                "unknown".to_string()
            };

            let example = TrainingExample {
                function_name: func.name.clone(),
                full_path: func.full_path.clone(),
                file: func.file.clone(),
                language: TrainingExample::detect_language(&func.file),
                features: FunctionFeatures::from_function(func, call_graph),
                label: label.clone(),
                confidence,
                source: label_reason.clone(),
                repository_id: None,
                commit_hash: None,
                dataset_split: None,
                label_reason: Some(label_reason),
                label_version: Some(1),
            };

            self.examples.push(example.clone());
            self.update_stats(&example);
        }
    }

    /// Add a high-confidence labeled example
    pub fn add_high_confidence_example(
        &mut self,
        func: &FunctionNode,
        call_graph: &CallGraph,
        label: TrainingLabel,
        confidence: f64,
        source_label: &str, // Renamed from `source` to avoid conflict
    ) {
        let example = TrainingExample {
            function_name: func.name.clone(),
            full_path: func.full_path.clone(),
            file: func.file.clone(),
            language: TrainingExample::detect_language(&func.file),
            features: FunctionFeatures::from_function(func, call_graph),
            label: label.clone(),
            confidence,
            source: source_label.to_string(),
            repository_id: None,
            commit_hash: None,
            dataset_split: None,
            label_reason: Some(source_label.to_string()),
            label_version: Some(1),
        };

        self.examples.push(example.clone());
        self.update_stats(&example);
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

    pub fn to_jsonl(&self) -> String {
        self.examples
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }

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
