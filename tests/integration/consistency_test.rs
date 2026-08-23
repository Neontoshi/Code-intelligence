// tests/integration/consistency_test.rs

use code_intelligence::analysis::service::{AnalysisService, AnalysisServiceConfig};
use code_intelligence::graph::GraphMetrics;
use code_intelligence::Pipeline;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Test that CLI and Dashboard analysis produce the same dead function count
#[test]
fn test_cli_dashboard_consistency_dead_count() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a test project with known dead code
    let code = r#"
pub fn main() {
    let result = helper(42);
    println!("{}", result);
}

fn helper(x: i32) -> i32 {
    x * 2
}

fn unused_function() -> i32 {
    0
}

fn another_unused() -> String {
    "dead".to_string()
}

// This is a test function - should be alive
#[test]
fn test_helper() {
    assert_eq!(helper(2), 4);
}

// Trait implementation - should be alive
pub trait Handler {
    fn handle(&self) -> String;
}

pub struct DefaultHandler;

impl Handler for DefaultHandler {
    fn handle(&self) -> String {
        "Handled".to_string()
    }
}
"#;

    let file_path = temp_path.join("test.rs");
    std::fs::write(&file_path, code).unwrap();

    // Create Cargo.toml for the test project
    let cargo_toml = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(temp_path.join("Cargo.toml"), cargo_toml).unwrap();

    // Run CLI analysis
    let cli_analysis = run_cli_analysis(temp_path);

    // Run Dashboard analysis (via service)
    let dashboard_analysis = run_dashboard_analysis(&temp_path.to_path_buf());

    // Compare results
    assert_eq!(
        cli_analysis.dead_count, dashboard_analysis.dead_count,
        "CLI dead count ({}) != Dashboard dead count ({})",
        cli_analysis.dead_count, dashboard_analysis.dead_count
    );

    assert_eq!(
        cli_analysis.total_functions, dashboard_analysis.total_functions,
        "CLI total functions ({}) != Dashboard total functions ({})",
        cli_analysis.total_functions, dashboard_analysis.total_functions
    );

    // Compare dead function names (should be identical)
    let mut cli_dead_names: Vec<String> = cli_analysis.dead_function_names;
    let mut dashboard_dead_names: Vec<String> = dashboard_analysis.dead_function_names;
    cli_dead_names.sort();
    dashboard_dead_names.sort();

    assert_eq!(
        cli_dead_names, dashboard_dead_names,
        "CLI dead functions {:?} != Dashboard dead functions {:?}",
        cli_dead_names, dashboard_dead_names
    );

    println!("✅ CLI and Dashboard produce identical results!");
    println!("   Dead count: {}", cli_analysis.dead_count);
    println!("   Total functions: {}", cli_analysis.total_functions);
    println!("   Dead functions: {:?}", cli_dead_names);
}

/// Test that the shared service produces the same results as the CLI
#[test]
fn test_service_cli_consistency() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let code = r#"
pub fn main() {
    used_function();
}

fn used_function() -> i32 {
    42
}

fn dead_function() -> i32 {
    0
}
"#;

    let file_path = temp_path.join("test.rs");
    std::fs::write(&file_path, code).unwrap();

    let cargo_toml = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(temp_path.join("Cargo.toml"), cargo_toml).unwrap();

    // Run CLI
    let cli_analysis = run_cli_analysis(temp_path);

    // Run Service
    let service_analysis = run_service_analysis(&temp_path.to_path_buf());

    assert_eq!(
        cli_analysis.dead_count, service_analysis.dead_count,
        "CLI dead count ({}) != Service dead count ({})",
        cli_analysis.dead_count, service_analysis.dead_count
    );
}

/// Test that the service properly loads and uses an ML model
#[test]
fn test_service_with_model() {
    // This test requires a model file
    // Skip if no model is available
    let model_path = PathBuf::from("models/dead_code_model_v4_balanced_calibrated.bin");
    if !model_path.exists() {
        println!(
            "⚠️ Skipping model test - model file not found: {:?}",
            model_path
        );
        return;
    }

    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let code = r#"
pub fn main() {
    helper();
}

fn helper() -> i32 {
    42
}

fn dead_func() -> i32 {
    0
}
"#;

    let file_path = temp_path.join("test.rs");
    std::fs::write(&file_path, code).unwrap();

    let cargo_toml = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(temp_path.join("Cargo.toml"), cargo_toml).unwrap();

    // Run service with model
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let config = AnalysisServiceConfig {
            model_path: Some(model_path),
            threshold: None,
            verbose: true,
            debug: false,
            cache: false,
            cache_dir: None,
            llm: false,
            git: false,
        };
        let mut service = AnalysisService::new(config);
        service.analyze(&temp_path.to_path_buf()).await
    });

    assert!(
        result.is_ok(),
        "Service with model failed: {:?}",
        result.err()
    );

    let analysis = result.unwrap();
    assert!(
        analysis.dead_verdicts.len() >= 1,
        "Expected at least 1 dead function"
    );

    println!(
        "✅ Service with model produced {} dead verdicts",
        analysis.dead_verdicts.len()
    );
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Run CLI analysis and return summary
fn run_cli_analysis(path: &Path) -> AnalysisSummary {
    use code_intelligence::analysis::dynamic_refs::DynamicRefDetector;
    use code_intelligence::analysis::roots::{
        ReachabilityAnalyzer, RootDetectionConfig, RootDetector,
    };
    use code_intelligence::analysis::verdict_source::state::{VerdictConfig, VerdictEngine};

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = Pipeline::new();
    let analysis = rt.block_on(pipeline.process_project(path)).unwrap();

    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    let dynamic_detector = DynamicRefDetector::new();
    let dynamic_refs = dynamic_detector.detect_all(&analysis.call_graph, &analysis.files);

    let verdict_engine =
        VerdictEngine::new(VerdictConfig::default()).with_dynamic_refs(dynamic_refs);

    let verdicts = verdict_engine.evaluate_all(&analysis.call_graph, &reachability);
    let dead_verdicts = verdict_engine.filter_dead(&verdicts);

    let dead_names: Vec<String> = dead_verdicts
        .iter()
        .map(|v| v.function_name.clone())
        .collect();

    AnalysisSummary {
        total_functions: analysis.call_graph.node_count(),
        dead_count: dead_verdicts.len(),
        dead_function_names: dead_names,
    }
}

/// Run Dashboard analysis (via service) and return summary
fn run_dashboard_analysis(path: &PathBuf) -> AnalysisSummary {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let config = AnalysisServiceConfig {
            model_path: None,
            threshold: None,
            verbose: false,
            debug: false,
            cache: false,
            cache_dir: None,
            llm: false,
            git: false,
        };
        let mut service = AnalysisService::new(config);
        service.analyze(path).await
    });

    let analysis = result.unwrap();

    let dead_names: Vec<String> = analysis
        .dead_verdicts
        .iter()
        .map(|v| v.function_name.clone())
        .collect();

    AnalysisSummary {
        total_functions: analysis.call_graph.node_count(),
        dead_count: analysis.dead_verdicts.len(),
        dead_function_names: dead_names,
    }
}

/// Run Service analysis and return summary
fn run_service_analysis(path: &PathBuf) -> AnalysisSummary {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let config = AnalysisServiceConfig {
            model_path: None,
            threshold: None,
            verbose: false,
            debug: false,
            cache: false,
            cache_dir: None,
            llm: false,
            git: false,
        };
        let mut service = AnalysisService::new(config);
        service.analyze(path).await
    });

    let analysis = result.unwrap();

    let dead_names: Vec<String> = analysis
        .dead_verdicts
        .iter()
        .map(|v| v.function_name.clone())
        .collect();

    AnalysisSummary {
        total_functions: analysis.call_graph.node_count(),
        dead_count: analysis.dead_verdicts.len(),
        dead_function_names: dead_names,
    }
}

/// Summary of analysis results for comparison
#[derive(Debug, Clone)]
struct AnalysisSummary {
    total_functions: usize,
    dead_count: usize,
    dead_function_names: Vec<String>,
}
