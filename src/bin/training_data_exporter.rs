// src/bin/training_data_exporter.rs

use code_intelligence::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
use code_intelligence::analysis::training_data::{TrainingDataCollector, TrainingLabel};
use code_intelligence::graph::GraphMetrics;
use code_intelligence::Pipeline;
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

    // Step 1: Detect ROOTS using unified RootDetector

    let config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &config);

    let roots = root_set.all();
    println!("   Found {} roots:", roots.len());
    for (category, count) in root_set.counts() {
        println!("      {}: {}", category, count);
    }

    // Step 2: Compute REACHABILITY using unified analyzer

    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    println!(
        "   {} functions reachable from roots",
        reachability.reachable_count()
    );
    println!(
        "   {} functions unreachable from roots",
        reachability.unreachable_count()
    );

    // Step 3: Label functions with HIGH CONFIDENCE

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

        // HIGH CONFIDENCE: ALIVE

        // Roots are definitely alive (entry points, exports, tests, etc.)
        if roots.contains(full_path) {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.99,
                "root",
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

        // Test functions are alive (even if not in root_set due to config)
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

        // HIGH CONFIDENCE: DEAD

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
            && !reachability.is_reachable(full_path)
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

        // UNKNOWN: Everything else

        unknown_count += 1;
    }

    // Print Statistics

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

    // Save to File

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
