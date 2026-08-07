// src/bin/training_data_exporter.rs

//! Export high-confidence training data for ML-based dead code detection
//!
//! This uses evidence-based labeling instead of heuristics:
//! - ALIVE: Functions with callers, exports, entry points, or test functions
//! - DEAD: Functions with NO callers, NO exports, NOT reachable from roots
//! - UNKNOWN: Everything else (we don't label what we're not sure about)

use code_intelligence::analysis::training_data::{TrainingDataCollector, TrainingLabel};
use code_intelligence::graph::GraphMetrics;
use code_intelligence::Pipeline;
use std::collections::HashSet;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };

    let output_file = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("training_data.json")
    };

    println!("📊 Collecting training data from: {:?}", path);
    println!("📁 Output file: {:?}", output_file);

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&path).await?;

    let mut collector = TrainingDataCollector::new();

    // ================================================================
    // Step 1: Identify ENTRY POINTS (definitely alive)
    // ================================================================

    let mut entry_points = HashSet::new();

    // Application entry points
    let app_entry_names = vec!["main", "async_main", "run", "start", "init", "setup"];

    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];
        if app_entry_names.contains(&func.name.as_str()) {
            entry_points.insert(func.full_path.clone());
        }
    }

    // Public functions with no callers are also entry points (library API)
    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];
        if func.is_public && func.fan_in == 0 {
            entry_points.insert(func.full_path.clone());
        }
    }

    println!("   Found {} entry points", entry_points.len());

    // ================================================================
    // Step 2: Identify ROOTS (entry points + test entry points)
    // ================================================================

    let mut roots = HashSet::new();

    // Add entry points
    for entry in &entry_points {
        roots.insert(entry.clone());
    }

    // Test entry points (test functions)
    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];
        let is_test = func.name.starts_with("test_")
            || func.name.starts_with("Test")
            || func.name.starts_with("bench_")
            || func.name.starts_with("Benchmark")
            || func.file.contains("/tests/")
            || func.file.ends_with("_test.rs")
            || func.file.ends_with("_test.go");
        if is_test {
            roots.insert(func.full_path.clone());
        }
    }

    println!(
        "   Found {} roots (entry points + test functions)",
        roots.len()
    );

    // ================================================================
    // Step 3: Compute REACHABILITY from roots
    // ================================================================

    let mut reachable = HashSet::new();
    let mut to_visit: Vec<String> = roots.iter().cloned().collect();

    while let Some(current) = to_visit.pop() {
        if reachable.contains(&current) {
            continue;
        }
        reachable.insert(current.clone());

        // Find the node index for this function
        for idx in analysis.call_graph.node_indices() {
            let func = &analysis.call_graph[idx];
            if func.full_path == current {
                // Add all callees
                for callee in analysis.call_graph.get_callees(idx) {
                    if !reachable.contains(&callee.full_path) {
                        to_visit.push(callee.full_path.clone());
                    }
                }
                break;
            }
        }
    }

    println!("   {} functions reachable from roots", reachable.len());
    println!(
        "   {} functions unreachable from roots",
        analysis.call_graph.node_count() - reachable.len()
    );

    // ================================================================
    // Step 4: Label functions with HIGH CONFIDENCE
    // ================================================================

    let mut alive_count = 0;
    let mut dead_count = 0;
    let mut unknown_count = 0;

    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];
        let full_path = &func.full_path;

        // Skip test functions — they have special patterns
        let is_test = func.name.starts_with("test_")
            || func.name.starts_with("Test")
            || func.name.starts_with("bench_")
            || func.name.starts_with("Benchmark")
            || func.file.contains("/tests/")
            || func.file.ends_with("_test.rs")
            || func.file.ends_with("_test.go");

        // ============================================================
        // HIGH CONFIDENCE: ALIVE
        // ============================================================

        // Entry points are definitely alive
        if entry_points.contains(full_path) {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.99,
                "entry_point",
            );
            alive_count += 1;
            continue;
        }

        // Functions with callers are alive (unless it's a cycle)
        if func.fan_in > 0 && !func.is_cycle {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.90,
                "has_callers",
            );
            alive_count += 1;
            continue;
        }

        // Test functions are alive
        if is_test {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.95,
                "test_function",
            );
            alive_count += 1;
            continue;
        }

        // Exported functions from libraries are alive
        if func.is_public && (func.file.contains("lib.rs") || func.file.contains("mod.rs")) {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.85,
                "library_export",
            );
            alive_count += 1;
            continue;
        }

        // ============================================================
        // HIGH CONFIDENCE: DEAD
        // ============================================================

        // A function is dead if:
        // 1. No callers
        // 2. Not public (can't be used externally)
        // 3. Not reachable from roots
        // 4. Not a trait implementation
        // 5. Not generated code
        // 6. Not in tests/benches
        // 7. Not a React component/hook

        let is_truly_dead = func.fan_in == 0
            && !func.is_public
            && !reachable.contains(full_path)
            && func.trait_impl.is_none()
            && !func.file.contains("/.meta/")
            && !func.file.contains(".gen.go")
            && !func.file.contains("_gen.go")
            && !func.file.contains("/dist/")
            && !func.file.contains("node_modules/")
            && !func.file.ends_with(".min.js")
            && !func.file.contains("/benches/")
            && !is_test;

        if is_truly_dead {
            // Check that it's not a React component
            let is_react = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
            let is_component = func
                .name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);
            let is_hook = func.name.starts_with("use");

            if !is_react || (!is_component && !is_hook) {
                collector.add_high_confidence_example(
                    func,
                    &analysis.call_graph,
                    TrainingLabel::Dead,
                    0.85,
                    "truly_dead",
                );
                dead_count += 1;
                continue;
            }
        }

        // ============================================================
        // UNKNOWN: Everything else
        // ============================================================

        unknown_count += 1;
    }

    // ================================================================
    // Print Statistics
    // ================================================================

    println!("\n📊 Training Data Stats:");
    println!("   Total functions: {}", analysis.call_graph.node_count());
    println!("   Labeled Alive: {}", alive_count);
    println!("   Labeled Dead: {}", dead_count);
    println!("   Unlabeled (Unknown): {}", unknown_count);

    // Show label distribution by category
    println!("\n   Label Sources:");
    let mut source_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for example in &collector.examples {
        *source_counts.entry(example.source.clone()).or_insert(0) += 1;
    }
    for (source, count) in &source_counts {
        println!("      {}: {}", source, count);
    }

    // Show languages
    println!("\n   By Language:");
    for (lang, count) in &collector.stats.by_language {
        println!("      {}: {}", lang, count);
    }

    // ================================================================
    // Save to File
    // ================================================================

    let json = collector.to_json()?;
    std::fs::write(&output_file, json)?;
    println!("\n✅ Training data saved to: {:?}", output_file);

    let jsonl_path = output_file.with_extension("jsonl");
    std::fs::write(&jsonl_path, collector.to_jsonl())?;
    println!("✅ JSONL format saved to: {:?}", jsonl_path);

    // Show a sample
    if let Some(first) = collector.examples.first() {
        println!("\n📝 Sample Example:");
        println!("   Function: {}", first.function_name);
        println!("   Label: {:?}", first.label);
        println!("   Confidence: {:.2}", first.confidence);
        println!("   Source: {}", first.source);
        println!("   Language: {}", first.language);
        println!("   Features:");
        println!("      - Params: {}", first.features.param_count);
        println!("      - Public: {}", first.features.is_public);
        println!("      - Fan-in: {}", first.features.fan_in);
        println!("      - Complexity: {}", first.features.complexity);
    }

    Ok(())
}
