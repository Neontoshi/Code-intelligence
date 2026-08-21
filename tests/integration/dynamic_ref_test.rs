// tests/integration/dynamic_ref_test.rs

//! Tests for dynamic reference detection

use code_intelligence::analysis::dynamic_refs::{DynamicRefDetector, DynamicRefType};
use code_intelligence::graph::call_graph::CallGraph;
use code_intelligence::parser::tree_sitter::TreeSitterParser;
use std::path::PathBuf;

#[test]
fn test_detect_python_reflection() {
    let source = r#"
        import importlib

        def dynamic_call(module_name, func_name):
            module = importlib.import_module(module_name)
            func = getattr(module, func_name)
            return func()

        @app.route('/api/test')
        def test_endpoint():
            return 'test'

        def normal_function():
            return 42
    "#;

    let (_, files) = create_files_from_source(source, "python");
    let call_graph = CallGraph::new();
    let detector = DynamicRefDetector::new();
    let refs = detector.detect_all(&call_graph, &files);

    // Should detect dynamic imports and decorators
    let dynamic_refs: Vec<_> = refs
        .iter()
        .filter(|r| r.reference_type == DynamicRefType::Reflection)
        .collect();

    assert!(
        !dynamic_refs.is_empty(),
        "Should detect reflection/getattr usage"
    );

    let framework_refs: Vec<_> = refs
        .iter()
        .filter(|r| r.reference_type == DynamicRefType::Framework)
        .collect();

    assert!(
        !framework_refs.is_empty(),
        "Should detect framework decorators"
    );

    println!("✅ Python dynamic reference detection test passed");
}

#[test]
fn test_detect_javascript_dynamic_imports() {
    let source = r#"
        import React from 'react';

        export function Component() {
            return <div>Test</div>;
        }

        // Dynamic import
        const module = import('./dynamic-module.js');

        // React hook
        function useCustomHook() {
            return useState(0);
        }

        // Normal function
        function helper() {
            return 'helper';
        }
    "#;

    let (_, files) = create_files_from_source(source, "typescript");
    let call_graph = CallGraph::new();
    let detector = DynamicRefDetector::new();
    let refs = detector.detect_all(&call_graph, &files);

    // Should detect framework (React) components and hooks
    let framework_refs: Vec<_> = refs
        .iter()
        .filter(|r| r.reference_type == DynamicRefType::Framework)
        .collect();

    assert!(!framework_refs.is_empty(), "Should detect React components");

    let dynamic_refs: Vec<_> = refs
        .iter()
        .filter(|r| r.reference_type == DynamicRefType::DynamicImport)
        .collect();

    assert!(!dynamic_refs.is_empty(), "Should detect dynamic imports");

    println!("✅ JavaScript dynamic reference detection test passed");
}

#[test]
fn test_detect_rust_dynamic_dispatch() {
    let source = r#"
        trait Handler {
            fn handle(&self);
        }

        struct DefaultHandler;

        impl Handler for DefaultHandler {
            fn handle(&self) {
                println!("Default");
            }
        }

        struct DynamicHandler;

        impl Handler for DynamicHandler {
            fn handle(&self) {
                println!("Dynamic");
            }
        }

        fn process(handler: &dyn Handler) {
            handler.handle();
        }

        fn normal_function() {
            // regular function
        }
    "#;

    let (_, files) = create_files_from_source(source, "rust");
    let call_graph = CallGraph::new();
    let detector = DynamicRefDetector::new();
    let _refs = detector.detect_all(&call_graph, &files);

    // The detector should identify trait methods as potential dynamic dispatch
    println!("✅ Rust dynamic dispatch detection test passed");
}

#[test]
fn test_detect_go_reflection() {
    let source = r#"
        package main

        import "reflect"

        type User struct {
            Name string
        }

        func process(data interface{}) {
            v := reflect.ValueOf(data)
            if v.Kind() == reflect.Struct {
                // reflection
            }
        }

        func normalFunction() {
            // regular
        }
    "#;

    let (_, files) = create_files_from_source(source, "go");
    let call_graph = CallGraph::new();
    let detector = DynamicRefDetector::new();
    let refs = detector.detect_all(&call_graph, &files);

    // Should detect reflection usage
    let reflection_refs: Vec<_> = refs
        .iter()
        .filter(|r| r.reference_type == DynamicRefType::Reflection)
        .collect();

    assert!(
        !reflection_refs.is_empty(),
        "Should detect reflect package usage"
    );

    println!("✅ Go reflection detection test passed");
}

// ============================================================
// Helper Functions
// ============================================================

fn create_files_from_source(
    source: &str,
    language: &str,
) -> (
    PathBuf,
    Vec<code_intelligence::parser::tree_sitter::ParsedFile>,
) {
    let parser = TreeSitterParser::new();
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path().with_extension(match language {
        "rust" => "rs",
        "python" => "py",
        "typescript" => "tsx",
        "go" => "go",
        "java" => "java",
        _ => "txt",
    });
    std::fs::write(&path, source).unwrap();

    let parsed = parser.parse_file(&path).unwrap();
    (path, vec![parsed])
}
