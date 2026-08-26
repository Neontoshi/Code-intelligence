// src/analysis/features.rs

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::core::{compute_ast_hash, compute_exact_hash, compute_signature_hash};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

/// All features extracted from a function - computed once, used everywhere
#[derive(Debug, Clone)]
pub struct FunctionFeatures {
    pub full_path: String,
    pub name: String,
    pub file: String,
    pub line: usize,
    pub signature_hash: String,
    pub ast_hash: String,
    pub body_hash: String,
    pub complexity: f64,
    pub cyclomatic_complexity: f64,
    pub nesting_depth: usize,
    pub line_count: usize,
    pub token_count: usize,
    pub param_count: usize,
    pub return_count: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub call_count: usize,
    pub caller_count: usize,
    pub fan_in: usize,
    pub fan_out: usize,
    pub language: String,
    pub layer: String,
    pub feature_vector: Vec<f64>,
    pub normalized_tokens: Vec<String>,
    pub body: Option<String>,
    pub doc_comment: Option<String>,
    pub is_method: bool,
    pub container: Option<String>,
    pub trait_impl: Option<String>,
    pub is_trait_method: bool,
    pub is_trait_default: bool,
    pub is_test: bool,
    pub is_override: bool,
}

impl FunctionFeatures {
    /// Create features from a function node and source
    pub fn from_function(func: &FunctionNode, source: Option<&str>, language: &str) -> Self {
        let body = source.map(|s| s.to_string());

        // 1. COMPUTE HASHES
        let signature_hash = compute_signature_hash(func);
        let ast_hash = if let Some(src) = source {
            compute_ast_hash(func, src)
        } else {
            String::new()
        };
        let body_hash = if let Some(src) = source {
            compute_exact_hash(func, Some(src))
        } else {
            compute_exact_hash(func, None)
        };

        // 2. COMPUTE COMPLEXITY
        let (complexity, cyclomatic, nesting) = if let Some(src) = source {
            Self::compute_complexity_metrics(src)
        } else {
            (1.0, 1.0, 0)
        };

        // 3. COMPUTE TOKENS
        let normalized_tokens = if let Some(src) = source {
            Self::normalize_tokens(src)
        } else {
            Vec::new()
        };

        // 4. COMPUTE BASIC METRICS
        let line_count = source.map(|s| s.lines().count()).unwrap_or(1);
        let token_count = source.map(|s| s.split_whitespace().count()).unwrap_or(0);

        // 5. COMPUTE TYPE-RELATED FIELDS
        let is_method = func
            .params
            .iter()
            .any(|p| p == "self" || p == "&self" || p == "&mut self");

        let container = None;

        let trait_impl = func.trait_impl.clone();
        let is_override = func.is_override();

        // 6. CREATE THE FEATURES STRUCT (WITHOUT FEATURE_VECTOR YET)
        let mut features = Self {
            full_path: func.full_path.clone(),
            name: func.name.clone(),
            file: func.file.clone(),
            line: func.line,

            signature_hash: signature_hash.clone(),
            ast_hash: ast_hash.clone(),
            body_hash: body_hash.clone(),

            complexity,
            cyclomatic_complexity: cyclomatic,
            nesting_depth: nesting,
            line_count,
            token_count,

            param_count: func.params.len(),
            return_count: func.returns.len(),
            is_public: func.is_public,
            is_async: func.is_async,

            call_count: func.fan_out,
            caller_count: func.fan_in,
            fan_in: func.fan_in,
            fan_out: func.fan_out,

            language: language.to_string(),
            layer: func.layer.clone(),

            // ⭐ These will be set after building the feature vector
            feature_vector: Vec::new(),
            normalized_tokens: normalized_tokens.clone(),

            body,
            doc_comment: func.doc_comment.clone(),

            // ⭐ Type-related fields
            is_method,
            container,
            trait_impl: trait_impl.clone(),
            is_trait_method: func.is_trait_method,
            is_trait_default: func.is_trait_default,
            is_test: func.is_test,
            is_override,
        };

        // 7. BUILD THE FEATURE VECTOR (USING THE CREATED FEATURES)
        let feature_vector = Self::build_feature_vector(
            &signature_hash,
            &ast_hash,
            func,
            complexity,
            &features.normalized_tokens,
            language,
            source,
            &features,
        );

        // 8. SET THE FEATURE VECTOR AND RETURN
        features.feature_vector = feature_vector;

        features
    }

    // Find this function and add bounds checking:

    fn compute_complexity_metrics(source: &str) -> (f64, f64, usize) {
        let mut complexity: f64 = 1.0;
        let mut cyclomatic: f64 = 1.0;
        let mut max_nesting: usize = 0;
        let mut current_nesting: usize = 0;

        let control_flow_patterns = [
            ("if", 0.5),
            ("else if", 0.3),
            ("for", 0.5),
            ("while", 0.5),
            ("loop", 0.3),
            ("match", 0.5),
            ("switch", 0.5),
            ("case", 0.2),
            ("&&", 0.2),
            ("||", 0.2),
            ("?", 0.3),
            ("catch", 0.3),
            ("try", 0.2),
            ("unwrap", 0.2),
            ("expect", 0.2),
        ];

        // Add bounds checking for empty source
        if source.is_empty() {
            return (1.0, 1.0, 0);
        }

        for line in source.lines() {
            let trimmed = line.trim();

            // Track nesting
            if trimmed.contains('{') {
                current_nesting += 1;
                max_nesting = max_nesting.max(current_nesting);
            }
            if trimmed.contains('}') {
                current_nesting = current_nesting.saturating_sub(1);
            }

            // Control flow complexity
            for (pattern, weight) in &control_flow_patterns {
                if trimmed.contains(pattern) {
                    complexity += weight;
                    cyclomatic += 1.0;
                }
            }
        }

        // Nesting penalty
        complexity += max_nesting as f64 * 0.2;

        // Cap at reasonable maximum
        let complexity = complexity.min(50.0);
        let cyclomatic = cyclomatic.min(50.0);

        (complexity, cyclomatic, max_nesting)
    }

    fn normalize_tokens(source: &str) -> Vec<String> {
        use regex::Regex;

        let mut tokens = Vec::new();

        // Replace identifiers with placeholders
        let id_regex = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
        let mut var_counter = 0;
        let mut var_map = HashMap::new();

        for word in source.split_whitespace() {
            let word = word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');

            if word.is_empty() {
                continue;
            }

            // Check if it's an identifier
            if id_regex.is_match(word) {
                let skip_words = [
                    "if", "else", "for", "while", "match", "fn", "pub", "async", "await", "return",
                    "let", "mut", "struct", "enum", "trait", "impl", "use", "mod", "true", "false",
                    "null", "None", "Some", "Ok", "Err", "Result", "Option", "Vec", "String",
                    "Box", "Arc", "Rc", "self", "Self", "super", "crate",
                ];
                if !skip_words.contains(&word) {
                    let var_id = var_map.entry(word.to_string()).or_insert_with(|| {
                        var_counter += 1;
                        var_counter
                    });
                    tokens.push(format!("VAR{}", var_id));
                    continue;
                }
            }
            tokens.push(word.to_string());
        }

        tokens
    }

    fn build_feature_vector(
        _signature_hash: &str,
        _ast_hash: &str,
        func: &FunctionNode,
        complexity: f64,
        _tokens: &[String],
        language: &str,
        source: Option<&str>,
        features: &FunctionFeatures,
    ) -> Vec<f64> {
        use crate::ml::feature_schema::FeatureVectorBuilder;

        let mut builder = FeatureVectorBuilder::new();

        // 1. GRAPH FEATURES (4)
        builder
            .push_normalized(func.fan_in as f64, 50.0)
            .push_normalized(func.fan_out as f64, 50.0)
            .push_normalized(func.depth as f64, 10.0)
            .push_bool(func.is_cycle);

        // 2. SIGNATURE FEATURES (8)
        builder
            .push_normalized(func.params.len() as f64, 10.0)
            .push_normalized(func.returns.len() as f64, 5.0)
            .push_bool(func.is_public)
            .push_bool(func.is_async)
            .push_bool(Self::is_generator(source))
            .push_bool(Self::is_static_method(func))
            .push_bool(Self::is_abstract_method(func))
            .push_bool(Self::is_override_method(func));

        // 3. COMPLEXITY FEATURES (4)
        let cognitive = Self::calculate_cognitive_complexity(source);
        let line_count = source.map(|s| s.lines().count()).unwrap_or(0);
        let token_count = source.map(|s| s.split_whitespace().count()).unwrap_or(0);

        builder
            .push_normalized(complexity, 50.0)
            .push_normalized(cognitive as f64, 20.0)
            .push_normalized(line_count as f64, 100.0)
            .push_normalized(token_count as f64, 500.0);

        // 4. NAME FEATURES (110)
        let name_lower = func.name.to_lowercase();

        // Contains patterns
        let contains_patterns = vec![
            "use",
            "test",
            "init",
            "get",
            "set",
            "new",
            "create",
            "build",
            "parse",
            "validate",
            "handle",
            "process",
            "convert",
            "commit",
            "reveal",
            "submit",
            "upload",
            "download",
            "fetch",
            "verify",
            "audit",
            "main",
            "start",
            "run",
            "load",
            "save",
            "read",
            "write",
            "open",
            "close",
            "connect",
            "disconnect",
            "send",
            "receive",
            "delete",
            "update",
            "patch",
            "put",
            "post",
            "list",
            "find",
            "search",
            "filter",
            "map",
            "reduce",
            "clone",
            "copy",
            "move",
            "swap",
            "sort",
            "is",
            "has",
            "can",
            "should",
            "will",
            "do",
            "make",
            "take",
            "give",
            "call",
            "apply",
            "register",
            "unregister",
            "subscribe",
            "unsubscribe",
        ];

        for pattern in contains_patterns {
            builder.push_bool(name_lower.contains(pattern));
        }

        // Starts with patterns
        let start_patterns = vec![
            "use", "test", "bench", "get", "set", "is", "has", "can", "should", "will", "on",
            "handle", "process", "parse", "create", "build", "make", "do", "apply",
        ];

        for pattern in start_patterns {
            builder.push_bool(func.name.starts_with(pattern));
        }

        // Ends with patterns
        let end_patterns = vec![
            "test",
            "handler",
            "processor",
            "service",
            "repository",
            "controller",
            "manager",
            "factory",
            "builder",
            "validator",
            "converter",
            "mapper",
            "filter",
            "loader",
            "saver",
            "creator",
            "updater",
            "deleter",
            "finder",
            "parser",
            "renderer",
            "serializer",
        ];

        for pattern in end_patterns {
            builder.push_bool(func.name.ends_with(pattern));
        }

        // Name length
        builder.push_normalized(func.name.len() as f64, 50.0);

        // 5. LANGUAGE FEATURES (10)
        builder.push_language(language);

        // 6. FRAMEWORK FEATURES (23)
        builder
            .push_bool(Self::is_flask_route(func, source))
            .push_bool(Self::is_fastapi_route(func, source))
            .push_bool(Self::is_express_route(func, source))
            .push_bool(Self::is_nextjs_route(func, source))
            .push_bool(Self::is_spring_controller(func, source))
            .push_bool(Self::is_aspnet_controller(func, source))
            .push_bool(Self::is_laravel_controller(func, source))
            .push_bool(Self::is_django_view(func, source))
            .push_bool(Self::is_rails_action(func, source))
            .push_bool(Self::is_react_component(func))
            .push_bool(Self::is_react_hook(func))
            .push_bool(Self::is_vue_component(func, source))
            .push_bool(Self::is_svelte_component(func, source))
            .push_bool(Self::is_flutter_widget(func))
            .push_bool(Self::is_flutter_state(func))
            .push_bool(Self::is_go_init(func))
            .push_bool(Self::is_go_interface(func))
            .push_bool(Self::is_go_goroutine(func, source))
            .push_bool(Self::is_rust_trait_impl(func))
            .push_bool(Self::is_rust_ffi(func, source));

        // 7. TYPE FEATURES (12)
        builder
            .push_bool(features.is_method)
            .push_bool(features.container.is_some())
            .push_bool(func.trait_impl.is_some())
            .push_bool(Self::has_receiver(func))
            .push_bool(Self::has_self(func))
            .push_opt(
                features.container.as_ref().map(|c| c.len() as f64 / 20.0),
                0.0,
            )
            .push_opt(func.trait_impl.as_ref().map(|t| t.len() as f64 / 20.0), 0.0)
            .push_bool(Self::type_trait_match(features))
            .push_bool(Self::has_generics(func, source))
            .push_normalized(Self::generic_count(func, source) as f64, 5.0)
            .push_bool(Self::has_type_annotation(func, source))
            .push_bool(Self::has_lifetime(func, source));

        // 8. FILE CONTEXT FEATURES (10)
        let file_path = &func.file;
        builder
            .push_bool(Self::is_in_test_file(file_path))
            .push_bool(Self::is_in_benches(file_path))
            .push_bool(Self::is_in_meta(file_path))
            .push_bool(Self::is_in_examples(file_path))
            .push_bool(Self::is_generated(file_path))
            .push_bool(Self::is_in_lib(file_path))
            .push_bool(Self::is_in_bin(file_path))
            .push_bool(Self::is_in_proto(file_path))
            .push_bool(Self::is_in_migrations(file_path))
            .push_bool(Self::is_in_fixtures(file_path));

        // 9. DECORATOR FEATURES (19)
        let decorator_patterns = vec![
            "route",
            "get",
            "post",
            "put",
            "delete",
            "patch",
            "override",
            "staticmethod",
            "classmethod",
            "property",
            "cached_property",
            "pytest",
            "fixture",
            "parametrize",
            "test",
        ];

        for pattern in decorator_patterns {
            builder.push_bool(Self::has_decorator(func, pattern));
        }

        // 10. DYNAMIC BEHAVIOR FEATURES (7)
        builder
            .push_bool(Self::has_dynamic_call(source))
            .push_bool(Self::has_ffi(func, source))
            .push_bool(Self::has_macro(func, source))
            .push_bool(Self::has_closure(source))
            .push_bool(Self::has_yield(source))
            .push_bool(Self::has_await(source))
            .push_bool(Self::has_thread(source));

        // 11. ERROR HANDLING FEATURES (6)
        builder
            .push_bool(Self::has_try_catch(source))
            .push_bool(Self::has_result_type(func))
            .push_bool(Self::has_throw(source))
            .push_bool(Self::has_panic(source))
            .push_bool(Self::has_question_mark(source))
            .push_bool(Self::has_error_propagation(source));

        // 12. DOCUMENTATION FEATURES (3)
        let doc_len = func.doc_comment.as_ref().map(|d| d.len()).unwrap_or(0);
        builder
            .push_bool(func.doc_comment.is_some())
            .push_normalized(doc_len as f64, 100.0)
            .push_bool(Self::has_attr_doc(func, source));

        // 13. VISIBILITY FEATURES (5)
        builder
            .push_bool(Self::is_pub_crate(func, source))
            .push_bool(Self::is_pub_super(func, source))
            .push_bool(Self::is_pub_self(func, source))
            .push_bool(Self::is_private(func))
            .push_bool(Self::is_protected(func, source));

        // 14. OWNERSHIP FEATURES (4)
        builder
            .push_bool(Self::has_borrow(source))
            .push_bool(Self::has_mut_ref(source))
            .push_bool(Self::has_move(source))
            .push_bool(Self::has_clone(source));

        // 15. GENERICS FEATURES (Already added above)

        // 16. PATTERN FEATURES (6)
        builder
            .push_bool(Self::is_singleton_pattern(func, source))
            .push_bool(Self::is_factory_pattern(func, source))
            .push_bool(Self::is_builder_pattern(func, source))
            .push_bool(Self::is_observer_pattern(func, source))
            .push_bool(Self::is_strategy_pattern(func, source))
            .push_bool(Self::is_decorator_pattern(func, source));

        // 17. CONCURRENCY FEATURES (4)
        builder
            .push_bool(Self::has_channel(source))
            .push_bool(Self::has_mutex(source))
            .push_bool(Self::has_atomic(source))
            .push_bool(Self::has_parallel(source));

        // Total: 224 features!
        builder.build()
    }

    // ================================================================
    // HELPER METHODS FOR FEATURE EXTRACTION
    // ================================================================

    fn is_generator(source: Option<&str>) -> bool {
        source.map(|s| s.contains("yield")).unwrap_or(false)
    }

    fn is_static_method(func: &FunctionNode) -> bool {
        func.decorators
            .iter()
            .any(|d| d.contains("staticmethod") || d.contains("classmethod"))
            || func.name.contains("static")
    }

    fn is_abstract_method(func: &FunctionNode) -> bool {
        func.decorators
            .iter()
            .any(|d| d.contains("abstract") || d.contains("virtual"))
            || func.is_trait_method
    }

    fn is_override_method(func: &FunctionNode) -> bool {
        func.decorators.iter().any(|d| d.contains("override")) || func.is_override()
    }

    fn calculate_cognitive_complexity(source: Option<&str>) -> usize {
        if let Some(src) = source {
            let mut depth: usize = 0;
            let mut max_depth = 0;
            for line in src.lines() {
                if line.contains('{') {
                    depth += 1;
                }
                if line.contains('}') {
                    depth = depth.saturating_sub(1);
                }
                max_depth = max_depth.max(depth);
            }
            max_depth
        } else {
            0
        }
    }

    fn has_receiver(func: &FunctionNode) -> bool {
        func.params
            .iter()
            .any(|p| p == "self" || p == "&self" || p == "&mut self")
            || func.file.ends_with(".go")
                && func
                    .params
                    .first()
                    .map(|p| p.contains('*'))
                    .unwrap_or(false)
    }

    fn has_self(func: &FunctionNode) -> bool {
        func.params
            .iter()
            .any(|p| p == "self" || p == "&self" || p == "&mut self")
            || func.name.contains("self")
    }

    fn type_trait_match(features: &FunctionFeatures) -> bool {
        if let (Some(container), Some(trait_impl)) = (&features.container, &features.trait_impl) {
            container == trait_impl
        } else {
            false
        }
    }

    fn has_generics(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| s.contains('<') && s.contains('>'))
            .unwrap_or(false)
    }

    fn generic_count(_func: &FunctionNode, source: Option<&str>) -> usize {
        if let Some(src) = source {
            src.matches('<').count()
        } else {
            0
        }
    }

    fn has_type_annotation(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| s.contains(':') && !s.contains("::"))
            .unwrap_or(false)
    }

    fn has_lifetime(_func: &FunctionNode, source: Option<&str>) -> bool {
        source.map(|s| s.contains('\'')).unwrap_or(false)
    }

    // File context helpers
    fn is_in_test_file(file: &str) -> bool {
        file.contains("/test/")
            || file.contains("/tests/")
            || file.ends_with("_test.rs")
            || file.ends_with("_test.go")
            || file.ends_with("_test.py")
            || file.ends_with(".test.ts")
            || file.ends_with(".test.js")
            || file.ends_with("_test.dart")
            || file.ends_with("Test.java")
    }

    fn is_in_benches(file: &str) -> bool {
        file.contains("/benches/") || file.ends_with("_bench.rs") || file.ends_with("_bench.go")
    }

    fn is_in_meta(file: &str) -> bool {
        file.contains("/.meta/")
    }

    fn is_in_examples(file: &str) -> bool {
        file.contains("/examples/") || file.contains("/example/")
    }

    fn is_generated(file: &str) -> bool {
        file.contains(".gen.")
            || file.contains("_gen.")
            || file.contains(".generated.")
            || file.contains(".pb.go")
            || file.contains("_pb2.py")
            || file.ends_with(".g.dart")
            || file.ends_with(".freezed.dart")
    }

    fn is_in_lib(file: &str) -> bool {
        file.contains("/lib/") || file.ends_with("lib.rs") || file.contains("/src/lib")
    }

    fn is_in_bin(file: &str) -> bool {
        file.contains("/bin/")
            || file.contains("/cmd/")
            || file.ends_with("main.rs")
            || file.contains("/src/bin/")
    }

    fn is_in_proto(file: &str) -> bool {
        file.contains("/proto/")
            || file.contains("/protobuf/")
            || file.ends_with(".proto")
            || file.ends_with(".pb.go")
    }

    fn is_in_migrations(file: &str) -> bool {
        file.contains("/migrations/") || file.contains("/db/migrate/")
    }

    fn is_in_fixtures(file: &str) -> bool {
        file.contains("/fixtures/") || file.contains("/testdata/")
    }

    // Framework detection helpers
    fn is_flask_route(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("@app.route") || s.contains("@router.route"))
            .unwrap_or(false)
    }

    fn is_fastapi_route(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("@app.get")
                    || s.contains("@app.post")
                    || s.contains("@app.put")
                    || s.contains("@router.get")
                    || s.contains("@router.post")
            })
            .unwrap_or(false)
    }

    fn is_express_route(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("app.get")
                    || s.contains("app.post")
                    || s.contains("app.put")
                    || s.contains("router.get")
                    || s.contains("router.post")
            })
            .unwrap_or(false)
    }

    fn is_nextjs_route(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("export default") && (s.contains("Page") || s.contains("Component"))
            })
            .unwrap_or(false)
    }

    fn is_spring_controller(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("@RestController")
                    || s.contains("@Controller")
                    || s.contains("@GetMapping")
                    || s.contains("@PostMapping")
            })
            .unwrap_or(false)
    }

    fn is_aspnet_controller(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("[ApiController]") || s.contains("[HttpGet") || s.contains("[HttpPost")
            })
            .unwrap_or(false)
    }

    fn is_laravel_controller(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("use Illuminate\\Http\\Request")
                    && s.contains("class")
                    && s.contains("Controller")
            })
            .unwrap_or(false)
    }

    fn is_django_view(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("def ") && (s.contains("request") || s.contains("HttpResponse")))
            .unwrap_or(false)
    }

    fn is_rails_action(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("def ") && s.contains("render") && !s.contains("private"))
            .unwrap_or(false)
    }

    fn is_react_component(func: &FunctionNode) -> bool {
        func.name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
            && (func.file.ends_with(".tsx") || func.file.ends_with(".jsx"))
    }

    fn is_react_hook(func: &FunctionNode) -> bool {
        func.name.starts_with("use")
            && func
                .name
                .chars()
                .nth(3)
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
    }

    fn is_vue_component(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Vue uses SFC files, not functions
    }

    fn is_svelte_component(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Svelte uses SFC files
    }

    fn is_flutter_widget(func: &FunctionNode) -> bool {
        func.name == "build"
            && func.file.ends_with(".dart")
            && func
                .trait_impl
                .as_ref()
                .map(|c| c.contains("Widget"))
                .unwrap_or(false)
    }

    fn is_flutter_state(func: &FunctionNode) -> bool {
        matches!(
            func.name.as_str(),
            "initState" | "dispose" | "didUpdateWidget" | "didChangeDependencies"
        ) && func.file.ends_with(".dart")
    }

    fn is_go_init(func: &FunctionNode) -> bool {
        func.name == "init" && func.file.ends_with(".go")
    }

    fn is_go_interface(func: &FunctionNode) -> bool {
        func.trait_impl.is_some() && func.file.ends_with(".go")
    }

    fn is_go_goroutine(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("go ") && s.contains("func("))
            .unwrap_or(false)
    }

    fn is_rust_trait_impl(func: &FunctionNode) -> bool {
        func.trait_impl.is_some() && func.file.ends_with(".rs")
    }

    fn is_rust_ffi(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("extern \"C\"") || s.contains("#[no_mangle]"))
            .unwrap_or(false)
    }

    // Decorator helpers
    fn has_decorator(func: &FunctionNode, pattern: &str) -> bool {
        func.decorators
            .iter()
            .any(|d| d.to_lowercase().contains(pattern))
    }

    // Dynamic behavior helpers
    fn has_dynamic_call(source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("getattr")
                    || s.contains("setattr")
                    || s.contains("hasattr")
                    || s.contains("importlib")
                    || s.contains("__import__")
                    || s.contains("reflect")
                    || s.contains("MethodByName")
                    || s.contains("call_user_func")
            })
            .unwrap_or(false)
    }

    fn has_ffi(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("extern \"C\"")
                    || s.contains("#[no_mangle]")
                    || s.contains("extern") && s.contains("C")
                    || s.contains("ffi")
            })
            .unwrap_or(false)
    }

    fn has_macro(_func: &FunctionNode, source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("macro_rules") || s.contains("#[derive") || s.contains("proc_macro")
            })
            .unwrap_or(false)
    }

    fn has_closure(source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("||")
                    || s.contains("lambda")
                    || s.contains("=>")
                    || s.contains("function(") && s.contains(") =>")
            })
            .unwrap_or(false)
    }

    fn has_yield(source: Option<&str>) -> bool {
        source.map(|s| s.contains("yield")).unwrap_or(false)
    }

    fn has_await(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("await") || s.contains(".await"))
            .unwrap_or(false)
    }

    fn has_thread(source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("tokio::spawn")
                    || s.contains("thread::spawn")
                    || s.contains("go ")
                    || s.contains("async_std::spawn")
            })
            .unwrap_or(false)
    }

    // Error handling helpers
    fn has_try_catch(source: Option<&str>) -> bool {
        source
            .map(|s| {
                s.contains("try") && s.contains("catch")
                    || s.contains("try:") && s.contains("except")
                    || s.contains("try {") && s.contains("catch (")
            })
            .unwrap_or(false)
    }

    fn has_result_type(func: &FunctionNode) -> bool {
        func.returns.iter().any(|r| {
            r.contains("Result")
                || r.contains("Option")
                || r.contains("Either")
                || r.contains("?")
                || r.contains("anyhow::Result")
        })
    }

    fn has_throw(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("throw") || s.contains("raise"))
            .unwrap_or(false)
    }

    fn has_panic(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("unwrap") || s.contains("expect") || s.contains("panic"))
            .unwrap_or(false)
    }

    fn has_question_mark(source: Option<&str>) -> bool {
        source.map(|s| s.contains("?")).unwrap_or(false)
    }

    fn has_error_propagation(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("?") || s.contains("try!") || s.contains("catch"))
            .unwrap_or(false)
    }

    // Documentation helpers
    fn has_attr_doc(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Custom attribute docs
    }

    // Visibility helpers
    fn is_pub_crate(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Rust-specific: pub(crate)
    }

    fn is_pub_super(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Rust-specific: pub(super)
    }

    fn is_pub_self(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Rust-specific: pub(self)
    }

    fn is_private(func: &FunctionNode) -> bool {
        !func.is_public
    }

    fn is_protected(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Language-specific
    }

    // Ownership helpers
    fn has_borrow(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("&") && !s.contains("&mut"))
            .unwrap_or(false)
    }

    fn has_mut_ref(source: Option<&str>) -> bool {
        source.map(|s| s.contains("&mut")).unwrap_or(false)
    }

    fn has_move(source: Option<&str>) -> bool {
        source.map(|s| s.contains("move")).unwrap_or(false)
    }

    fn has_clone(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("clone") || s.contains("Clone"))
            .unwrap_or(false)
    }

    // Concurrency helpers
    fn has_channel(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("channel") || s.contains("mpsc") || s.contains("watch"))
            .unwrap_or(false)
    }

    fn has_mutex(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("Mutex") || s.contains("RwLock"))
            .unwrap_or(false)
    }

    fn has_atomic(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("Atomic") || s.contains("Ordering"))
            .unwrap_or(false)
    }

    fn has_parallel(source: Option<&str>) -> bool {
        source
            .map(|s| s.contains("rayon") || s.contains("par_iter") || s.contains("parallel"))
            .unwrap_or(false)
    }

    // Pattern detection helpers
    fn is_singleton_pattern(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Complex pattern detection
    }

    fn is_factory_pattern(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Complex pattern detection
    }

    fn is_builder_pattern(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Complex pattern detection
    }

    fn is_observer_pattern(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Complex pattern detection
    }

    fn is_strategy_pattern(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Complex pattern detection
    }

    fn is_decorator_pattern(_func: &FunctionNode, _source: Option<&str>) -> bool {
        false // Complex pattern detection
    }
}

pub fn cosine_similarity(a: &FunctionFeatures, b: &FunctionFeatures) -> f64 {
    let a_vec = &a.feature_vector;
    let b_vec = &b.feature_vector;

    if a_vec.is_empty() || b_vec.is_empty() || a_vec.len() != b_vec.len() {
        return 0.0;
    }

    let dot: f64 = a_vec.iter().zip(b_vec).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a_vec.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let norm_b: f64 = b_vec.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

pub fn token_overlap(a: &FunctionFeatures, b: &FunctionFeatures) -> f64 {
    let a_set: std::collections::HashSet<_> = a.normalized_tokens.iter().collect();
    let b_set: std::collections::HashSet<_> = b.normalized_tokens.iter().collect();

    let intersection = a_set.intersection(&b_set).count();
    let union = a_set.len() + b_set.len() - intersection;

    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}
// Feature Extractor

#[derive(Debug)]
pub struct FeatureExtractor {
    features: HashMap<String, FunctionFeatures>,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }
    pub fn insert(&mut self, full_path: String, feature: FunctionFeatures) {
        self.features.insert(full_path, feature);
    }

    /// Extract features for all functions in a codebase
    pub fn extract_all(
        &mut self,
        functions: &[FunctionNode],
        files: &[ParsedFile],
    ) -> &HashMap<String, FunctionFeatures> {
        // Build source map for quick lookup
        let source_map: HashMap<String, &str> = files
            .iter()
            .flat_map(|f| {
                f.functions.iter().map(move |fi| {
                    let full_path = format!("{}::{}", f.path, fi.name);
                    let range = &fi.body_range;
                    let source = &f.source[range.0..range.1];
                    (full_path, source)
                })
            })
            .collect();

        // Build language map
        let lang_map: HashMap<String, &str> = files
            .iter()
            .flat_map(|f| {
                f.functions.iter().map(move |fi| {
                    let full_path = format!("{}::{}", f.path, fi.name);
                    (full_path, f.language.as_str())
                })
            })
            .collect();

        for func in functions {
            let full_path = &func.full_path;
            let source = source_map.get(full_path).copied();
            let language = lang_map.get(full_path).copied().unwrap_or("unknown");

            let features = FunctionFeatures::from_function(func, source, language);
            self.features.insert(full_path.clone(), features);
        }

        &self.features
    }

    /// Get features for a specific function
    pub fn get(&self, full_path: &str) -> Option<&FunctionFeatures> {
        self.features.get(full_path)
    }

    /// Get all features
    pub fn all(&self) -> &HashMap<String, FunctionFeatures> {
        &self.features
    }

    /// Get feature vector for ML
    pub fn get_feature_vector(&self, full_path: &str) -> Option<Vec<f64>> {
        self.features
            .get(full_path)
            .map(|f| f.feature_vector.clone())
    }

    /// Get normalized tokens for a function
    pub fn get_tokens(&self, full_path: &str) -> Option<Vec<String>> {
        self.features
            .get(full_path)
            .map(|f| f.normalized_tokens.clone())
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}
