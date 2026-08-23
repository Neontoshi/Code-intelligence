// benches/large_repo_benchmark.rs

use code_intelligence::{
    analysis::{
        dead_code::DeadCodeDetector,
        roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
        verdict_source::state::{VerdictConfig, VerdictEngine},
    },
    graph::GraphMetrics,
    Pipeline,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;

/// Simulate a large repository with generated code
fn generate_large_repo(size: usize) -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Generate multiple files with many functions
    for file_idx in 0..(size / 20).max(1) {
        let mut code = String::new();

        // Add imports
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::sync::Arc;\n\n");

        // Generate 20 functions per file
        for func_idx in 0..20 {
            let is_public = func_idx % 3 == 0;
            let is_async = func_idx % 4 == 0;
            let has_params = func_idx % 2 == 0;
            let has_return = func_idx % 3 != 1;
            let has_calls = func_idx % 3 != 2;

            code.push_str(if is_public { "pub " } else { "" });
            code.push_str(if is_async { "async " } else { "" });
            code.push_str(&format!("fn func_{}_{}(", file_idx, func_idx));

            let mut params = Vec::new();
            if has_params {
                params.push("x: i32".to_string());
                if func_idx % 3 == 0 {
                    params.push("y: i32".to_string());
                }
                if func_idx % 5 == 0 {
                    params.push("z: String".to_string());
                }
            }
            code.push_str(&params.join(", "));
            code.push_str(")");

            if has_return {
                code.push_str(" -> i32");
            }
            code.push_str(" {\n");

            // Body
            if has_calls && func_idx > 0 {
                let callee_idx = func_idx - 1;
                if func_idx % 2 == 0 {
                    code.push_str(&format!(
                        "    let result = func_{}_{}(x);\n",
                        file_idx, callee_idx
                    ));
                } else {
                    code.push_str(&format!(
                        "    let result = helper_{}_{}(x);\n",
                        file_idx, callee_idx
                    ));
                }
            }

            // Random complexity
            if func_idx % 5 == 0 {
                code.push_str("    if x > 10 {\n");
                code.push_str("        return x * 2;\n");
                code.push_str("    } else if x > 5 {\n");
                code.push_str("        return x + 10;\n");
                code.push_str("    } else {\n");
                code.push_str("        return x;\n");
                code.push_str("    }\n");
            } else if has_return {
                code.push_str("    x + 1\n");
            } else {
                code.push_str("    println!(\"Hello\");\n");
            }
            code.push_str("}\n\n");

            // Generate some helper functions
            if func_idx % 3 == 0 {
                code.push_str(&format!(
                    "fn helper_{}_{}(x: i32) -> i32 {{\n",
                    file_idx, func_idx
                ));
                code.push_str("    x * 2\n");
                code.push_str("}\n\n");
            }
        }

        // Generate structs and types
        for type_idx in 0..3 {
            code.push_str(&format!("pub struct Type_{} {{\n", type_idx));
            for field_idx in 0..3 {
                let field_type = if field_idx % 2 == 0 { "i32" } else { "String" };
                code.push_str(&format!("    field_{}: {},\n", field_idx, field_type));
            }
            code.push_str("}\n\n");

            code.push_str(&format!("impl Type_{} {{\n", type_idx));
            code.push_str(&format!("    pub fn new() -> Self {{\n"));
            code.push_str(&format!(
                "        Self {{ field_0: 0, field_1: \"\".to_string(), field_2: 0 }}\n"
            ));
            code.push_str("    }\n");
            code.push_str("}\n\n");
        }

        let file_name = format!("file_{:04}.rs", file_idx);
        std::fs::write(path.join(&file_name), code).unwrap();
    }

    // Add a main.rs entry point
    let main_code = r#"
pub fn main() {
    let result = func_0_0(42);
    println!("Result: {}", result);
}
"#;
    std::fs::write(path.join("main.rs"), main_code).unwrap();

    // Add Cargo.toml
    let cargo_toml = r#"[package]
name = "large_repo"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(path.join("Cargo.toml"), cargo_toml).unwrap();

    temp_dir
}

/// Benchmark: Parse a large repository
fn bench_parse_large_repo(c: &mut Criterion) {
    let temp_dir = generate_large_repo(1000);
    let path = temp_dir.path();

    c.bench_function("parse_1000_functions", |b| {
        b.iter(|| {
            let parser = code_intelligence::parser::tree_sitter::TreeSitterParser::new();
            let files: Vec<_> = std::fs::read_dir(path)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
                .map(|e| parser.parse_file(&e.path()).unwrap())
                .collect();
            black_box(files.len())
        })
    });
}

/// Benchmark: Build call graph for large repository
fn bench_build_call_graph(c: &mut Criterion) {
    let temp_dir = generate_large_repo(1000);
    let path = temp_dir.path();

    let parser = code_intelligence::parser::tree_sitter::TreeSitterParser::new();
    let files: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        .map(|e| parser.parse_file(&e.path()).unwrap())
        .collect();

    c.bench_function("build_graph_1000_functions", |b| {
        b.iter(|| {
            let graph = code_intelligence::engine::call_graph_builder::CallGraphBuilder::build(
                black_box(&files),
            );
            black_box(graph.node_count())
        })
    });
}

/// Benchmark: Full analysis pipeline
fn bench_full_analysis(c: &mut Criterion) {
    let temp_dir = generate_large_repo(500);
    let path = temp_dir.path().to_path_buf();

    c.bench_function("full_analysis_500_functions", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut pipeline = Pipeline::new();
            let analysis = rt
                .block_on(pipeline.process_project(black_box(&path)))
                .unwrap();
            black_box(analysis.call_graph.node_count())
        })
    });
}

/// Benchmark: Verdict engine performance
fn bench_verdict_engine(c: &mut Criterion) {
    let temp_dir = generate_large_repo(500);
    let path = temp_dir.path();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = Pipeline::new();
    let analysis = rt.block_on(pipeline.process_project(path)).unwrap();

    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);
    let verdict_engine = VerdictEngine::new(VerdictConfig::default());

    c.bench_function("verdict_engine_500_functions", |b| {
        b.iter(|| {
            let verdicts = verdict_engine
                .evaluate_all(black_box(&analysis.call_graph), black_box(&reachability));
            black_box(verdicts.len())
        })
    });
}

/// Benchmark: Dead code detection
fn bench_dead_code_detection(c: &mut Criterion) {
    let temp_dir = generate_large_repo(500);
    let path = temp_dir.path();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = Pipeline::new();
    let analysis = rt.block_on(pipeline.process_project(path)).unwrap();

    c.bench_function("dead_code_detection_500_functions", |b| {
        b.iter(|| {
            let stats = DeadCodeDetector::get_dead_stats(
                black_box(&analysis.call_graph),
                black_box(&analysis.files),
            );
            black_box(stats.dead)
        })
    });
}

/// Benchmark: Memory usage during analysis
fn bench_memory_usage(c: &mut Criterion) {
    let temp_dir = generate_large_repo(1000);
    let path = temp_dir.path();

    c.bench_function("memory_usage_1000_functions", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut pipeline = Pipeline::new();
            let start = Instant::now();
            let analysis = rt
                .block_on(pipeline.process_project(black_box(&path)))
                .unwrap();
            let elapsed = start.elapsed();

            // Rough memory estimate
            let node_count = analysis.call_graph.node_count();
            let edge_count = analysis.call_graph.edge_count();

            // Print metrics
            println!("\n📊 Memory Benchmark Results:");
            println!("   Functions: {}", node_count);
            println!("   Edges: {}", edge_count);
            println!("   Files: {}", analysis.files.len());
            println!("   Time: {:.2}s", elapsed.as_secs_f64());

            black_box((node_count, edge_count, elapsed))
        })
    });
}

/// Benchmark: Scalability test with different sizes
fn bench_scalability(_c: &mut Criterion) {
    let sizes = [100, 250, 500, 1000];
    let mut results = Vec::new();

    for &size in &sizes {
        let temp_dir = generate_large_repo(size);
        let path = temp_dir.path();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let start = Instant::now();
        let mut pipeline = Pipeline::new();
        let analysis = rt.block_on(pipeline.process_project(path)).unwrap();
        let elapsed = start.elapsed();

        results.push((size, analysis.call_graph.node_count(), elapsed));

        println!("\n📊 Scalability Benchmark ({} functions):", size);
        println!("   Actual functions: {}", analysis.call_graph.node_count());
        println!("   Time: {:.2}s", elapsed.as_secs_f64());
        println!("   Edge count: {}", analysis.call_graph.edge_count());
    }

    // Print summary
    println!("\n📊 Scalability Summary:");
    println!("   Functions | Time (s) | Functions/s");
    println!("   ----------|----------|------------");
    for (_size, actual, elapsed) in results {
        let rate = actual as f64 / elapsed.as_secs_f64();
        println!(
            "   {:>9} | {:>8.2} | {:>10.0}",
            actual,
            elapsed.as_secs_f64(),
            rate
        );
    }
}

/// Benchmark: Cache performance
fn bench_cache_performance(c: &mut Criterion) {
    let temp_dir = generate_large_repo(500);
    let path = temp_dir.path().to_path_buf();
    let cache_dir = path.join(".cache");

    c.bench_function("cache_hit_500_functions", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut pipeline = Pipeline::new().with_cache_dir(cache_dir.clone());
            let start = Instant::now();
            let analysis = rt
                .block_on(pipeline.process_project(black_box(&path)))
                .unwrap();
            let elapsed = start.elapsed();

            // First run populates cache, subsequent runs use it
            // The benchmark will benefit from cache on repeated runs
            black_box((analysis.call_graph.node_count(), elapsed))
        })
    });
}

criterion_group!(
    benches,
    bench_parse_large_repo,
    bench_build_call_graph,
    bench_full_analysis,
    bench_verdict_engine,
    bench_dead_code_detection,
    bench_memory_usage,
    bench_scalability,
    bench_cache_performance,
);
criterion_main!(benches);
