// src/analysis/training_data.rs

use crate::analysis::verdict_source::label_source::LabelSource;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::ml::feature_schema::FeatureVectorBuilder;
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
    pub repository_id: Option<String>,
    pub commit_hash: Option<String>,
    pub dataset_split: Option<String>,
    pub label_reason: Option<String>,
    pub label_version: Option<u32>,
    pub label_source: LabelSource,
    pub generated_by_model: Option<String>,
    pub verified_by: Option<String>,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrainingLabel {
    Alive,
    Dead,
    Unknown,
}

// src/analysis/training_data.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionFeatures {
    // ================================================================
    // EXISTING FIELDS (keep all)
    // ================================================================
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
    pub signature_hash: String,
    pub body_hash: String,
    pub fan_in: usize,
    pub fan_out: usize,
    pub complexity: f64,
    pub call_depth: usize,
    pub is_cycle: bool,
    pub file_extension: String,
    pub is_in_test_file: bool,
    pub is_in_benches: bool,
    pub is_in_meta: bool,
    pub is_in_examples: bool,
    pub is_generated: bool,
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
    pub type_name: Option<String>,
    pub type_path: Option<String>,
    pub is_method: bool,
    pub is_trait_impl: bool,
    pub trait_name: Option<String>,
    pub is_associated: bool,

    // ================================================================
    // NEW SIGNATURE FEATURES (3)
    // ================================================================
    pub is_generator: bool,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_override: bool,

    // ================================================================
    // NEW COMPLEXITY FEATURES (3)
    // ================================================================
    pub cognitive_complexity: usize,
    pub line_count: usize,
    pub token_count: usize,

    // ================================================================
    // NEW NAME PATTERNS (expanded)
    // ================================================================
    // Additional contains patterns
    pub name_contains_main: bool,
    pub name_contains_start: bool,
    pub name_contains_run: bool,
    pub name_contains_load: bool,
    pub name_contains_save: bool,
    pub name_contains_read: bool,
    pub name_contains_write: bool,
    pub name_contains_open: bool,
    pub name_contains_close: bool,
    pub name_contains_connect: bool,
    pub name_contains_send: bool,
    pub name_contains_receive: bool,
    pub name_contains_delete: bool,
    pub name_contains_update: bool,
    pub name_contains_patch: bool,
    pub name_contains_put: bool,
    pub name_contains_post: bool,
    pub name_contains_list: bool,
    pub name_contains_find: bool,
    pub name_contains_search: bool,
    pub name_contains_filter: bool,
    pub name_contains_map: bool,
    pub name_contains_reduce: bool,
    pub name_contains_clone: bool,
    pub name_contains_copy: bool,
    pub name_contains_move: bool,
    pub name_contains_swap: bool,
    pub name_contains_sort: bool,
    pub name_contains_is: bool,
    pub name_contains_has: bool,
    pub name_contains_can: bool,
    pub name_contains_should: bool,
    pub name_contains_will: bool,
    pub name_contains_do: bool,
    pub name_contains_make: bool,
    pub name_contains_take: bool,
    pub name_contains_give: bool,
    pub name_contains_call: bool,
    pub name_contains_apply: bool,
    pub name_contains_register: bool,
    pub name_contains_unregister: bool,
    pub name_contains_subscribe: bool,
    pub name_contains_unsubscribe: bool,

    // Starts with patterns (additional)
    pub starts_with_get: bool,
    pub starts_with_set: bool,
    pub starts_with_is: bool,
    pub starts_with_has: bool,
    pub starts_with_can: bool,
    pub starts_with_should: bool,
    pub starts_with_will: bool,
    pub starts_with_on: bool,
    pub starts_with_handle: bool,
    pub starts_with_process: bool,
    pub starts_with_parse: bool,
    pub starts_with_create: bool,
    pub starts_with_build: bool,
    pub starts_with_make: bool,
    pub starts_with_do: bool,
    pub starts_with_apply: bool,

    // Ends with patterns (additional)
    pub ends_with_handler: bool,
    pub ends_with_processor: bool,
    pub ends_with_service: bool,
    pub ends_with_repository: bool,
    pub ends_with_controller: bool,
    pub ends_with_manager: bool,
    pub ends_with_factory: bool,
    pub ends_with_builder: bool,
    pub ends_with_validator: bool,
    pub ends_with_converter: bool,
    pub ends_with_mapper: bool,
    pub ends_with_filter: bool,
    pub ends_with_loader: bool,
    pub ends_with_saver: bool,
    pub ends_with_creator: bool,
    pub ends_with_updater: bool,
    pub ends_with_deleter: bool,
    pub ends_with_finder: bool,
    pub ends_with_parser: bool,
    pub ends_with_renderer: bool,
    pub ends_with_serializer: bool,

    // ================================================================
    // NEW LANGUAGE FEATURES (1 - others are in feature vector)
    // ================================================================
    pub language: String,

    // ================================================================
    // NEW FRAMEWORK FEATURES (20)
    // ================================================================
    pub is_flask_route: bool,
    pub is_fastapi_route: bool,
    pub is_express_route: bool,
    pub is_nextjs_route: bool,
    pub is_spring_controller: bool,
    pub is_aspnet_controller: bool,
    pub is_laravel_controller: bool,
    pub is_django_view: bool,
    pub is_rails_action: bool,
    pub is_react_component: bool,
    pub is_react_hook: bool,
    pub is_vue_component: bool,
    pub is_svelte_component: bool,
    pub is_flutter_widget: bool,
    pub is_flutter_state: bool,
    pub is_go_init: bool,
    pub is_go_interface: bool,
    pub is_go_goroutine: bool,
    pub is_rust_trait_impl: bool,
    pub is_rust_ffi: bool,

    // ================================================================
    // NEW TYPE FEATURES (5)
    // ================================================================
    pub has_receiver: bool,
    pub has_self: bool,
    pub has_generics: bool,
    pub generic_count: usize,
    pub has_type_annotation: bool,
    pub has_lifetime: bool,

    // ================================================================
    // NEW FILE CONTEXT FEATURES (4)
    // ================================================================
    pub is_in_lib: bool,
    pub is_in_bin: bool,
    pub is_in_proto: bool,
    pub is_in_migrations: bool,
    pub is_in_fixtures: bool,

    // ================================================================
    // NEW DECORATOR FEATURES (15)
    // ================================================================
    pub has_decorator_route: bool,
    pub has_decorator_get: bool,
    pub has_decorator_post: bool,
    pub has_decorator_put: bool,
    pub has_decorator_delete: bool,
    pub has_decorator_patch: bool,
    pub has_decorator_override: bool,
    pub has_decorator_staticmethod: bool,
    pub has_decorator_classmethod: bool,
    pub has_decorator_property: bool,
    pub has_decorator_cached_property: bool,
    pub has_decorator_pytest: bool,
    pub has_decorator_fixture: bool,
    pub has_decorator_parametrize: bool,
    pub has_decorator_test: bool,

    // ================================================================
    // NEW DYNAMIC BEHAVIOR FEATURES (7)
    // ================================================================
    pub has_dynamic_call: bool,
    pub has_ffi: bool,
    pub has_macro: bool,
    pub has_closure: bool,
    pub has_yield: bool,
    pub has_await: bool,
    pub has_thread: bool,

    // ================================================================
    // NEW ERROR HANDLING FEATURES (6)
    // ================================================================
    pub has_try_catch: bool,
    pub has_result_type: bool,
    pub has_throw: bool,
    pub has_panic: bool,
    pub has_question_mark: bool,
    pub has_error_propagation: bool,

    // ================================================================
    // NEW DOCUMENTATION FEATURES (3)
    // ================================================================
    pub has_doc_comment: bool,
    pub doc_comment_length: usize,
    pub has_attr_doc: bool,

    // ================================================================
    // NEW VISIBILITY FEATURES (5)
    // ================================================================
    pub vis_pub_crate: bool,
    pub vis_pub_super: bool,
    pub vis_pub_self: bool,
    pub vis_private: bool,
    pub vis_protected: bool,

    // ================================================================
    // NEW OWNERSHIP FEATURES (4)
    // ================================================================
    pub has_borrow: bool,
    pub has_mut_ref: bool,
    pub has_move: bool,
    pub has_clone: bool,

    // ================================================================
    // NEW PATTERN FEATURES (6)
    // ================================================================
    pub pattern_singleton: bool,
    pub pattern_factory: bool,
    pub pattern_builder: bool,
    pub pattern_observer: bool,
    pub pattern_strategy: bool,
    pub pattern_decorator: bool,

    // ================================================================
    // NEW CONCURRENCY FEATURES (4)
    // ================================================================
    pub has_channel: bool,
    pub has_mutex: bool,
    pub has_atomic: bool,
    pub has_parallel: bool,
}

// src/analysis/training_data.rs

// Add this implementation for FunctionFeatures
impl Default for FunctionFeatures {
    fn default() -> Self {
        Self {
            param_count: 0,
            return_count: 0,
            is_public: false,
            is_async: false,
            name_length: 0,
            starts_with_use: false,
            starts_with_test: false,
            starts_with_bench: false,
            ends_with_test: false,
            contains_trait_impl: false,
            signature_hash: String::new(),
            body_hash: String::new(),
            fan_in: 0,
            fan_out: 0,
            complexity: 0.0,
            call_depth: 0,
            is_cycle: false,
            file_extension: String::new(),
            is_in_test_file: false,
            is_in_benches: false,
            is_in_meta: false,
            is_in_examples: false,
            is_generated: false,
            name_contains_use: false,
            name_contains_test: false,
            name_contains_init: false,
            name_contains_get: false,
            name_contains_set: false,
            name_contains_new: false,
            name_contains_create: false,
            name_contains_build: false,
            name_contains_parse: false,
            name_contains_validate: false,
            name_contains_handle: false,
            name_contains_process: false,
            name_contains_convert: false,
            name_contains_commit: false,
            name_contains_reveal: false,
            name_contains_submit: false,
            name_contains_upload: false,
            name_contains_download: false,
            name_contains_fetch: false,
            name_contains_verify: false,
            name_contains_audit: false,
            type_name: None,
            type_path: None,
            is_method: false,
            is_trait_impl: false,
            trait_name: None,
            is_associated: false,

            // === NEW FIELDS ===
            // Signature
            is_generator: false,
            is_static: false,
            is_abstract: false,
            is_override: false,

            // Complexity
            cognitive_complexity: 0,
            line_count: 0,
            token_count: 0,

            // Name - additional contains
            name_contains_main: false,
            name_contains_start: false,
            name_contains_run: false,
            name_contains_load: false,
            name_contains_save: false,
            name_contains_read: false,
            name_contains_write: false,
            name_contains_open: false,
            name_contains_close: false,
            name_contains_connect: false,
            name_contains_send: false,
            name_contains_receive: false,
            name_contains_delete: false,
            name_contains_update: false,
            name_contains_patch: false,
            name_contains_put: false,
            name_contains_post: false,
            name_contains_list: false,
            name_contains_find: false,
            name_contains_search: false,
            name_contains_filter: false,
            name_contains_map: false,
            name_contains_reduce: false,
            name_contains_clone: false,
            name_contains_copy: false,
            name_contains_move: false,
            name_contains_swap: false,
            name_contains_sort: false,
            name_contains_is: false,
            name_contains_has: false,
            name_contains_can: false,
            name_contains_should: false,
            name_contains_will: false,
            name_contains_do: false,
            name_contains_make: false,
            name_contains_take: false,
            name_contains_give: false,
            name_contains_call: false,
            name_contains_apply: false,
            name_contains_register: false,
            name_contains_unregister: false,
            name_contains_subscribe: false,
            name_contains_unsubscribe: false,

            // Name - starts with
            starts_with_get: false,
            starts_with_set: false,
            starts_with_is: false,
            starts_with_has: false,
            starts_with_can: false,
            starts_with_should: false,
            starts_with_will: false,
            starts_with_on: false,
            starts_with_handle: false,
            starts_with_process: false,
            starts_with_parse: false,
            starts_with_create: false,
            starts_with_build: false,
            starts_with_make: false,
            starts_with_do: false,
            starts_with_apply: false,

            // Name - ends with
            ends_with_handler: false,
            ends_with_processor: false,
            ends_with_service: false,
            ends_with_repository: false,
            ends_with_controller: false,
            ends_with_manager: false,
            ends_with_factory: false,
            ends_with_builder: false,
            ends_with_validator: false,
            ends_with_converter: false,
            ends_with_mapper: false,
            ends_with_filter: false,
            ends_with_loader: false,
            ends_with_saver: false,
            ends_with_creator: false,
            ends_with_updater: false,
            ends_with_deleter: false,
            ends_with_finder: false,
            ends_with_parser: false,
            ends_with_renderer: false,
            ends_with_serializer: false,

            // Language
            language: String::new(),

            // Framework
            is_flask_route: false,
            is_fastapi_route: false,
            is_express_route: false,
            is_nextjs_route: false,
            is_spring_controller: false,
            is_aspnet_controller: false,
            is_laravel_controller: false,
            is_django_view: false,
            is_rails_action: false,
            is_react_component: false,
            is_react_hook: false,
            is_vue_component: false,
            is_svelte_component: false,
            is_flutter_widget: false,
            is_flutter_state: false,
            is_go_init: false,
            is_go_interface: false,
            is_go_goroutine: false,
            is_rust_trait_impl: false,
            is_rust_ffi: false,

            // Type
            has_receiver: false,
            has_self: false,
            has_generics: false,
            generic_count: 0,
            has_type_annotation: false,
            has_lifetime: false,

            // File
            is_in_lib: false,
            is_in_bin: false,
            is_in_proto: false,
            is_in_migrations: false,
            is_in_fixtures: false,

            // Decorators
            has_decorator_route: false,
            has_decorator_get: false,
            has_decorator_post: false,
            has_decorator_put: false,
            has_decorator_delete: false,
            has_decorator_patch: false,
            has_decorator_override: false,
            has_decorator_staticmethod: false,
            has_decorator_classmethod: false,
            has_decorator_property: false,
            has_decorator_cached_property: false,
            has_decorator_pytest: false,
            has_decorator_fixture: false,
            has_decorator_parametrize: false,
            has_decorator_test: false,

            // Dynamic
            has_dynamic_call: false,
            has_ffi: false,
            has_macro: false,
            has_closure: false,
            has_yield: false,
            has_await: false,
            has_thread: false,

            // Error Handling
            has_try_catch: false,
            has_result_type: false,
            has_throw: false,
            has_panic: false,
            has_question_mark: false,
            has_error_propagation: false,

            // Documentation
            has_doc_comment: false,
            doc_comment_length: 0,
            has_attr_doc: false,

            // Visibility
            vis_pub_crate: false,
            vis_pub_super: false,
            vis_pub_self: false,
            vis_private: false,
            vis_protected: false,

            // Ownership
            has_borrow: false,
            has_mut_ref: false,
            has_move: false,
            has_clone: false,

            // Patterns
            pattern_singleton: false,
            pattern_factory: false,
            pattern_builder: false,
            pattern_observer: false,
            pattern_strategy: false,
            pattern_decorator: false,

            // Concurrency
            has_channel: false,
            has_mutex: false,
            has_atomic: false,
            has_parallel: false,
            // Type info (keep existing)
            // type_name, type_path, is_method, is_trait_impl, trait_name, is_associated
            // are already defined above
        }
    }
}

impl FunctionFeatures {
    pub fn from_function(func: &FunctionNode, _call_graph: &CallGraph) -> Self {
        // Start with default values (all fields set to 0/false/None)
        let mut features = Self::default();

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

        use crate::optimize::dedup::core::{compute_exact_hash, compute_signature_hash};
        let sig_hash = compute_signature_hash(func);
        let body_hash = compute_exact_hash(func, None);

        // Set all the fields on the default struct
        features.param_count = func.params.len();
        features.return_count = func.returns.len();
        features.is_public = func.is_public;
        features.is_async = func.is_async;
        features.name_length = func.name.len();
        features.starts_with_use = func.name.starts_with("use");
        features.starts_with_test = func.name.starts_with("test_") || func.name.starts_with("Test");
        features.starts_with_bench =
            func.name.starts_with("bench_") || func.name.starts_with("Benchmark");
        features.ends_with_test = func.name.ends_with("_test");
        features.contains_trait_impl = contains_trait_impl;
        features.signature_hash = sig_hash;
        features.body_hash = body_hash;
        features.fan_in = func.fan_in;
        features.fan_out = func.fan_out;
        features.complexity = func.complexity;
        features.call_depth = call_depth;
        features.is_cycle = func.is_cycle;
        features.file_extension = file_extension;
        features.is_in_test_file = is_in_test_file;
        features.is_in_benches = is_in_benches;
        features.is_in_meta = is_in_meta;
        features.is_in_examples = is_in_examples;
        features.is_generated = is_generated;

        // Name contains patterns
        features.name_contains_use = name_lower.contains("use");
        features.name_contains_test = name_lower.contains("test");
        features.name_contains_init = name_lower.contains("init");
        features.name_contains_get = name_lower.contains("get");
        features.name_contains_set = name_lower.contains("set");
        features.name_contains_new = name_lower.contains("new");
        features.name_contains_create = name_lower.contains("create");
        features.name_contains_build = name_lower.contains("build");
        features.name_contains_parse = name_lower.contains("parse");
        features.name_contains_validate = name_lower.contains("validate");
        features.name_contains_handle = name_lower.contains("handle");
        features.name_contains_process = name_lower.contains("process");
        features.name_contains_convert = name_lower.contains("convert");
        features.name_contains_commit = name_lower.contains("commit");
        features.name_contains_reveal = name_lower.contains("reveal");
        features.name_contains_submit = name_lower.contains("submit");
        features.name_contains_upload = name_lower.contains("upload");
        features.name_contains_download = name_lower.contains("download");
        features.name_contains_fetch = name_lower.contains("fetch");
        features.name_contains_verify = name_lower.contains("verify");
        features.name_contains_audit = name_lower.contains("audit");

        // Type info
        features.type_name = type_info.type_name;
        features.type_path = type_info.type_path;
        features.is_method = type_info.is_method;
        features.is_trait_impl = type_info.is_trait_impl;
        features.trait_name = type_info.trait_name;
        features.is_associated = type_info.is_associated;

        features
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

    pub fn to_feature_vector(&self) -> Vec<f64> {
        let mut builder = FeatureVectorBuilder::new();

        // ================================================================
        // 1. GRAPH FEATURES (4)
        // ================================================================
        builder
            .push_normalized(self.fan_in as f64, 50.0)
            .push_normalized(self.fan_out as f64, 50.0)
            .push_normalized(self.call_depth as f64, 10.0)
            .push_bool(self.is_cycle);

        // ================================================================
        // 2. SIGNATURE FEATURES (8)
        // ================================================================
        builder
            .push_normalized(self.param_count as f64, 10.0)
            .push_normalized(self.return_count as f64, 5.0)
            .push_bool(self.is_public)
            .push_bool(self.is_async)
            .push_bool(self.is_generator)
            .push_bool(self.is_static)
            .push_bool(self.is_abstract)
            .push_bool(self.is_override);

        // ================================================================
        // 3. COMPLEXITY FEATURES (4)
        // ================================================================
        builder
            .push_normalized(self.complexity, 50.0)
            .push_normalized(self.cognitive_complexity as f64, 20.0)
            .push_normalized(self.line_count as f64, 100.0)
            .push_normalized(self.token_count as f64, 500.0);

        // ================================================================
        // 4. NAME FEATURES (40)
        // ================================================================
        // Contains patterns (30+)
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
            .push_bool(self.name_contains_audit)
            // New patterns
            .push_bool(self.name_contains_main)
            .push_bool(self.name_contains_start)
            .push_bool(self.name_contains_run)
            .push_bool(self.name_contains_load)
            .push_bool(self.name_contains_save)
            .push_bool(self.name_contains_read)
            .push_bool(self.name_contains_write)
            .push_bool(self.name_contains_open)
            .push_bool(self.name_contains_close)
            .push_bool(self.name_contains_connect)
            .push_bool(self.name_contains_send)
            .push_bool(self.name_contains_receive)
            .push_bool(self.name_contains_delete)
            .push_bool(self.name_contains_update)
            .push_bool(self.name_contains_patch)
            .push_bool(self.name_contains_put)
            .push_bool(self.name_contains_post)
            .push_bool(self.name_contains_list)
            .push_bool(self.name_contains_find)
            .push_bool(self.name_contains_search)
            .push_bool(self.name_contains_filter)
            .push_bool(self.name_contains_map)
            .push_bool(self.name_contains_reduce)
            .push_bool(self.name_contains_clone)
            .push_bool(self.name_contains_copy)
            .push_bool(self.name_contains_move)
            .push_bool(self.name_contains_swap)
            .push_bool(self.name_contains_sort)
            .push_bool(self.name_contains_is)
            .push_bool(self.name_contains_has)
            .push_bool(self.name_contains_can)
            .push_bool(self.name_contains_should)
            .push_bool(self.name_contains_will)
            .push_bool(self.name_contains_do)
            .push_bool(self.name_contains_make)
            .push_bool(self.name_contains_take)
            .push_bool(self.name_contains_give)
            .push_bool(self.name_contains_call)
            .push_bool(self.name_contains_apply)
            .push_bool(self.name_contains_register)
            .push_bool(self.name_contains_unregister)
            .push_bool(self.name_contains_subscribe)
            .push_bool(self.name_contains_unsubscribe);

        // Starts with patterns (19)
        builder
            .push_bool(self.starts_with_use)
            .push_bool(self.starts_with_test)
            .push_bool(self.starts_with_bench)
            .push_bool(self.starts_with_get)
            .push_bool(self.starts_with_set)
            .push_bool(self.starts_with_is)
            .push_bool(self.starts_with_has)
            .push_bool(self.starts_with_can)
            .push_bool(self.starts_with_should)
            .push_bool(self.starts_with_will)
            .push_bool(self.starts_with_on)
            .push_bool(self.starts_with_handle)
            .push_bool(self.starts_with_process)
            .push_bool(self.starts_with_parse)
            .push_bool(self.starts_with_create)
            .push_bool(self.starts_with_build)
            .push_bool(self.starts_with_make)
            .push_bool(self.starts_with_do)
            .push_bool(self.starts_with_apply);

        // Ends with patterns (22)
        builder
            .push_bool(self.ends_with_test)
            .push_bool(self.ends_with_handler)
            .push_bool(self.ends_with_processor)
            .push_bool(self.ends_with_service)
            .push_bool(self.ends_with_repository)
            .push_bool(self.ends_with_controller)
            .push_bool(self.ends_with_manager)
            .push_bool(self.ends_with_factory)
            .push_bool(self.ends_with_builder)
            .push_bool(self.ends_with_validator)
            .push_bool(self.ends_with_converter)
            .push_bool(self.ends_with_mapper)
            .push_bool(self.ends_with_filter)
            .push_bool(self.ends_with_loader)
            .push_bool(self.ends_with_saver)
            .push_bool(self.ends_with_creator)
            .push_bool(self.ends_with_updater)
            .push_bool(self.ends_with_deleter)
            .push_bool(self.ends_with_finder)
            .push_bool(self.ends_with_parser)
            .push_bool(self.ends_with_renderer)
            .push_bool(self.ends_with_serializer);

        // Name length
        builder.push_normalized(self.name_length as f64, 50.0);

        // ================================================================
        // 5. LANGUAGE FEATURES (10)
        // ================================================================
        builder.push_language(&self.language);

        // ================================================================
        // 6. FRAMEWORK FEATURES (20)
        // ================================================================
        builder
            .push_bool(self.is_flask_route)
            .push_bool(self.is_fastapi_route)
            .push_bool(self.is_express_route)
            .push_bool(self.is_nextjs_route)
            .push_bool(self.is_spring_controller)
            .push_bool(self.is_aspnet_controller)
            .push_bool(self.is_laravel_controller)
            .push_bool(self.is_django_view)
            .push_bool(self.is_rails_action)
            .push_bool(self.is_react_component)
            .push_bool(self.is_react_hook)
            .push_bool(self.is_vue_component)
            .push_bool(self.is_svelte_component)
            .push_bool(self.is_flutter_widget)
            .push_bool(self.is_flutter_state)
            .push_bool(self.is_go_init)
            .push_bool(self.is_go_interface)
            .push_bool(self.is_go_goroutine)
            .push_bool(self.is_rust_trait_impl)
            .push_bool(self.is_rust_ffi);

        // ================================================================
        // 7. TYPE FEATURES (12)
        // ================================================================
        builder
            .push_bool(self.is_method)
            .push_bool(self.is_trait_impl)
            .push_bool(self.is_associated)
            .push_bool(self.has_receiver)
            .push_bool(self.has_self)
            .push_opt(self.type_name.as_ref().map(|s| s.len() as f64 / 20.0), 0.0)
            .push_opt(self.trait_name.as_ref().map(|s| s.len() as f64 / 20.0), 0.0)
            .push_bool(self.type_name == self.trait_name)
            .push_bool(self.has_generics)
            .push_normalized(self.generic_count as f64, 5.0)
            .push_bool(self.has_type_annotation)
            .push_bool(self.has_lifetime);

        // ================================================================
        // 8. FILE CONTEXT FEATURES (10)
        // ================================================================
        builder
            .push_bool(self.is_in_test_file)
            .push_bool(self.is_in_benches)
            .push_bool(self.is_in_meta)
            .push_bool(self.is_in_examples)
            .push_bool(self.is_generated)
            .push_bool(self.is_in_lib)
            .push_bool(self.is_in_bin)
            .push_bool(self.is_in_proto)
            .push_bool(self.is_in_migrations)
            .push_bool(self.is_in_fixtures);

        // ================================================================
        // 9. DECORATOR FEATURES (15)
        // ================================================================
        builder
            .push_bool(self.has_decorator_route)
            .push_bool(self.has_decorator_get)
            .push_bool(self.has_decorator_post)
            .push_bool(self.has_decorator_put)
            .push_bool(self.has_decorator_delete)
            .push_bool(self.has_decorator_patch)
            .push_bool(self.has_decorator_override)
            .push_bool(self.has_decorator_staticmethod)
            .push_bool(self.has_decorator_classmethod)
            .push_bool(self.has_decorator_property)
            .push_bool(self.has_decorator_cached_property)
            .push_bool(self.has_decorator_pytest)
            .push_bool(self.has_decorator_fixture)
            .push_bool(self.has_decorator_parametrize)
            .push_bool(self.has_decorator_test);

        // ================================================================
        // 10. DYNAMIC BEHAVIOR FEATURES (7)
        // ================================================================
        builder
            .push_bool(self.has_dynamic_call)
            .push_bool(self.has_ffi)
            .push_bool(self.has_macro)
            .push_bool(self.has_closure)
            .push_bool(self.has_yield)
            .push_bool(self.has_await)
            .push_bool(self.has_thread);

        // ================================================================
        // 11. ERROR HANDLING FEATURES (6)
        // ================================================================
        builder
            .push_bool(self.has_try_catch)
            .push_bool(self.has_result_type)
            .push_bool(self.has_throw)
            .push_bool(self.has_panic)
            .push_bool(self.has_question_mark)
            .push_bool(self.has_error_propagation);

        // ================================================================
        // 12. DOCUMENTATION FEATURES (3)
        // ================================================================
        builder
            .push_bool(self.has_doc_comment)
            .push_normalized(self.doc_comment_length as f64, 100.0)
            .push_bool(self.has_attr_doc);

        // ================================================================
        // 13. VISIBILITY FEATURES (5)
        // ================================================================
        builder
            .push_bool(self.vis_pub_crate)
            .push_bool(self.vis_pub_super)
            .push_bool(self.vis_pub_self)
            .push_bool(self.vis_private)
            .push_bool(self.vis_protected);

        // ================================================================
        // 14. OWNERSHIP FEATURES (4)
        // ================================================================
        builder
            .push_bool(self.has_borrow)
            .push_bool(self.has_mut_ref)
            .push_bool(self.has_move)
            .push_bool(self.has_clone);

        // ================================================================
        // 15. GENERICS FEATURES (Already added above)
        // ================================================================

        // ================================================================
        // 16. PATTERN FEATURES (6)
        // ================================================================
        builder
            .push_bool(self.pattern_singleton)
            .push_bool(self.pattern_factory)
            .push_bool(self.pattern_builder)
            .push_bool(self.pattern_observer)
            .push_bool(self.pattern_strategy)
            .push_bool(self.pattern_decorator);

        // ================================================================
        // 17. CONCURRENCY FEATURES (4)
        // ================================================================
        builder
            .push_bool(self.has_channel)
            .push_bool(self.has_mutex)
            .push_bool(self.has_atomic)
            .push_bool(self.has_parallel);

        builder.build()
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
            label_source: LabelSource::StaticHeuristic,
            generated_by_model: None,
            verified_by: None,
            created_at: Some(chrono::Utc::now().timestamp()),
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
            label_source: LabelSource::StaticHeuristic,
            generated_by_model: None,
            verified_by: None,
            created_at: Some(chrono::Utc::now().timestamp()),
        }
    }

    pub fn new_verified(
        func: &FunctionNode,
        call_graph: &CallGraph,
        label: TrainingLabel,
        source: LabelSource,
        verified_by: &str,
    ) -> Self {
        let confidence = source.confidence_multiplier();
        Self {
            function_name: func.name.clone(),
            full_path: func.full_path.clone(),
            file: func.file.clone(),
            language: Self::detect_language(&func.file),
            features: FunctionFeatures::from_function(func, call_graph),
            label,
            confidence,
            source: format!("verified_{}", verified_by),
            repository_id: None,
            commit_hash: None,
            dataset_split: None,
            label_reason: Some(format!("verified_by_{}", verified_by)),
            label_version: Some(1),
            label_source: source,
            generated_by_model: None,
            verified_by: Some(verified_by.to_string()),
            created_at: Some(chrono::Utc::now().timestamp()),
        }
    }

    pub fn is_verified(&self) -> bool {
        self.label_source.is_verified()
    }

    pub fn is_heuristic(&self) -> bool {
        self.label_source.is_heuristic()
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
        } else if file.ends_with(".dart") {
            "dart".to_string()
        } else if file.ends_with(".php") {
            "php".to_string()
        } else if file.ends_with(".cs") {
            "csharp".to_string()
        } else if file.ends_with(".cpp")
            || file.ends_with(".cc")
            || file.ends_with(".cxx")
            || file.ends_with(".hpp")
            || file.ends_with(".h")
        {
            "cpp".to_string()
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
                label,
                confidence,
                source: label_reason.clone(),
                repository_id: None,
                commit_hash: None,
                dataset_split: None,
                label_reason: Some(label_reason),
                label_version: Some(1),
                label_source: LabelSource::StaticHeuristic,
                generated_by_model: None,
                verified_by: None,
                created_at: Some(chrono::Utc::now().timestamp()),
            };

            self.examples.push(example.clone());
            self.update_stats(&example);
        }
    }

    pub fn add_high_confidence_example(
        &mut self,
        func: &FunctionNode,
        call_graph: &CallGraph,
        label: TrainingLabel,
        confidence: f64,
        source_label: &str,
    ) {
        let example = TrainingExample {
            function_name: func.name.clone(),
            full_path: func.full_path.clone(),
            file: func.file.clone(),
            language: TrainingExample::detect_language(&func.file),
            features: FunctionFeatures::from_function(func, call_graph),
            label,
            confidence,
            source: source_label.to_string(),
            repository_id: None,
            commit_hash: None,
            dataset_split: None,
            label_reason: Some(source_label.to_string()),
            label_version: Some(1),
            label_source: LabelSource::StaticHeuristic,
            generated_by_model: None,
            verified_by: None,
            created_at: Some(chrono::Utc::now().timestamp()),
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
