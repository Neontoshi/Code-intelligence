// tests/unit/feature_tests.rs

//! Unit tests for feature extraction

use code_intelligence::analysis::features::{
    cosine_similarity, token_overlap, FeatureExtractor, FunctionFeatures,
};
use code_intelligence::graph::call_graph::{CallGraph, FunctionNode};
use code_intelligence::parser::tree_sitter::{FunctionInfo, FunctionRole, ParsedFile};

#[test]
fn test_feature_extraction_basic() {
    let func = create_test_function("test_func", true, false);
    let _call_graph = CallGraph::new();
    let source = "fn test_func() -> i32 { 42 }";

    let features = FunctionFeatures::from_function(&func, Some(source), "rust");

    assert_eq!(features.name, "test_func");
    assert_eq!(features.language, "rust");
    assert_eq!(features.param_count, 0);
    assert_eq!(features.return_count, 0);
    assert!(features.is_public);
    assert!(!features.is_async);
}

#[test]
fn test_feature_extraction_with_params() {
    let mut func = create_test_function("add", false, false);
    func.params = vec!["a".to_string(), "b".to_string()];
    func.returns = vec!["i32".to_string()];

    let _call_graph = CallGraph::new();
    let source = "fn add(a: i32, b: i32) -> i32 { a + b }";

    let features = FunctionFeatures::from_function(&func, Some(source), "rust");

    assert_eq!(features.param_count, 2);
    assert_eq!(features.return_count, 1);
    assert_eq!(features.line_count, 1);
    assert!(features.complexity > 0.0);
}

#[test]
fn test_feature_extractor_collect() {
    let mut extractor = FeatureExtractor::new();

    let funcs = vec![
        create_test_function("func1", true, false),
        create_test_function("func2", false, true),
    ];

    let files = vec![create_test_file()];

    let features = extractor.extract_all(&funcs, &files);

    assert_eq!(features.len(), 2);
    assert!(features.contains_key("test.rs::func1"));
    assert!(features.contains_key("test.rs::func2"));
}

#[test]
fn test_feature_cosine_similarity() {
    let func1 = create_test_function("test1", true, false);
    let func2 = create_test_function("test2", true, false);
    let _call_graph = CallGraph::new();

    let features1 = FunctionFeatures::from_function(&func1, Some("fn test1() {}"), "rust");
    let features2 = FunctionFeatures::from_function(&func2, Some("fn test2() {}"), "rust");

    // ✅ Use the FREE function, not a method!
    let similarity = cosine_similarity(&features1, &features2);
    assert!(similarity >= 0.0 && similarity <= 1.0);
}

#[test]
fn test_feature_token_overlap() {
    let func1 = create_test_function("test1", true, false);
    let func2 = create_test_function("test2", true, false);
    let _call_graph = CallGraph::new();

    let features1 =
        FunctionFeatures::from_function(&func1, Some("fn test1() { let x = 42; }"), "rust");
    let features2 =
        FunctionFeatures::from_function(&func2, Some("fn test2() { let y = 42; }"), "rust");

    // ✅ Use the FREE function, not a method!
    let overlap = token_overlap(&features1, &features2);
    assert!(overlap >= 0.0 && overlap <= 1.0);
}

// ================================================================
// Helper functions
// ================================================================

fn create_test_function(name: &str, is_public: bool, is_async: bool) -> FunctionNode {
    FunctionNode {
        name: name.to_string(),
        full_path: format!("test.rs::{}", name),
        file: "test.rs".to_string(),
        line: 1,
        body_start_line: 1,
        body_end_line: 10,
        is_public,
        is_async,
        params: vec![],
        returns: vec![],
        complexity: 1.0,
        importance_score: 0.0,
        doc_comment: None,
        writes_to: vec![],
        reads_from: vec![],
        errors: vec![],
        fan_in: 0,
        fan_out: 0,
        is_cycle: false,
        depth: 0,
        layer: "core".to_string(),
        trait_impl: None,
        decorators: Vec::new(),
        is_test: false,
        is_trait_method: false,
        is_trait_default: false,
    }
}

fn create_test_file() -> ParsedFile {
    let source = "fn func1() {\n    // body 1\n}\n\nfn func2() {\n    // body 2\n}\n".to_string();

    ParsedFile {
        path: "test.rs".to_string(),
        language: "rust".to_string(),
        functions: vec![
            FunctionInfo {
                name: "func1".to_string(),
                line: 1,
                is_public: true,
                is_async: false,
                params: vec![],
                return_type: None,
                doc_comment: None,
                calls: vec![],
                body_range: (0, 25),
                body_start_line: 1,
                body_end_line: 3,
                complexity: 0,
                container: None,
                role: FunctionRole::Unknown,
                purpose: "test".to_string(),
                trait_impl: None,
                decorators: vec![],
                is_test: false,
                is_trait_method: false,
                is_trait_default: false,
                variables: vec![],
            },
            FunctionInfo {
                name: "func2".to_string(),
                line: 5,
                is_public: false,
                is_async: true,
                params: vec![],
                return_type: None,
                doc_comment: None,
                calls: vec![],
                body_range: (27, 52),
                body_start_line: 5,
                body_end_line: 7,
                complexity: 0,
                container: None,
                role: FunctionRole::Unknown,
                purpose: "test".to_string(),
                trait_impl: None,
                decorators: vec![],
                is_test: false,
                is_trait_method: false,
                is_trait_default: false,
                variables: vec![],
            },
        ],
        imports: vec![],
        types: vec![],
        source,
    }
}
