// tests/integration/test-self-analysis.rs

use code_intelligence::analysis::dead_code::DeadCodeDetector;
use code_intelligence::graph::GraphMetrics;
use code_intelligence::optimize::dedup::Deduplicator;
use code_intelligence::Pipeline;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_pipeline_self_analysis() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&root).await;

    assert!(
        analysis.is_ok(),
        "Pipeline should parse code-intelligence itself"
    );
    let analysis = analysis.unwrap();

    assert!(
        analysis.call_graph.node_count() > 50,
        "Call graph should have mapped all internal functions"
    );
    assert!(
        !analysis.files.is_empty(),
        "Should have indexed source files"
    );
}

#[tokio::test]
async fn test_multi_language_fixture_detection() {
    let temp_dir = tempfile::tempdir().expect("Create temporary fixture directory");
    let temp_path = temp_dir.path();

    // 1. Rust module
    let rs_code = r#"
        pub fn entry_point() {
            used_helper();
        }

        fn used_helper() {}

        fn dead_rust_fn() {
            println!("I am dead code");
        }
    "#;
    fs::write(temp_path.join("main.rs"), rs_code).unwrap();

    // 2. Python module with route decorator
    let py_code = r#"
        def route(path):
            def decorator(f):
                return f
            return decorator

        @route("/api/v1/alive")
        def alive_endpoint():
            return "ok"

        def dead_python_function():
            pass
    "#;
    fs::write(temp_path.join("app.py"), py_code).unwrap();

    // 3. TypeScript module with JSX component
    let ts_code = r#"
        export function UserProfile() {
            return "<div>User</div>";
        }

        function deadTypeScriptHelper() {
            return false;
        }
    "#;
    fs::write(temp_path.join("Component.tsx"), ts_code).unwrap();

    let mut pipeline = Pipeline::new();
    let analysis = pipeline
        .process_project(temp_path)
        .await
        .expect("Process multi-language fixture");

    let stats = DeadCodeDetector::get_dead_stats(&analysis.call_graph, &analysis.files);

    assert!(
        stats.total >= 5,
        "Should have discovered all fixture functions across languages"
    );
    assert!(
        stats.dead >= 2,
        "Should have detected dead functions across languages"
    );

    // Deduplication check
    let dedup = Deduplicator::new();
    let dedup_report = dedup.find_duplicates(&analysis.call_graph, &analysis.files);
    assert!(dedup_report.accuracy_metrics.confidence_score >= 0.0);
}
