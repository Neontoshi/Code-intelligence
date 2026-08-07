// src/bin/training_data_exporter.rs

use code_intelligence::analysis::training_data::{TrainingDataCollector, TrainingLabel};
use code_intelligence::graph::GraphMetrics; // ⭐ ADD THIS
use code_intelligence::Pipeline;
use std::path::PathBuf;

// Remove the unused WHITELIST import

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

    // High-confidence label generation
    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];

        // Skip test functions — they have special patterns
        let is_test = func.name.starts_with("test_")
            || func.name.starts_with("Test")
            || func.name.starts_with("bench_")
            || func.name.starts_with("Benchmark")
            || func.file.contains("/tests/")
            || func.file.ends_with("_test.rs")
            || func.file.ends_with("_test.go");

        if is_test {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.95,
                "test",
            );
            continue;
        }

        // Entry points are ALIVE
        let is_entry = func.name == "main"
            || func.name == "async_main"
            || func.name == "run"
            || func.name == "start";

        if is_entry {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.99,
                "entry_point",
            );
            continue;
        }

        // Exported functions that are USED are ALIVE
        let is_exported =
            func.is_public || func.file.contains("lib.rs") || func.file.contains("mod.rs");
        let has_callers = func.fan_in > 0;

        if is_exported && has_callers {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.90,
                "exported_and_used",
            );
            continue;
        }

        // Functions that are called AND exported are definitely ALIVE
        if func.fan_in > 0 {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.85,
                "has_callers",
            );
            continue;
        }

        // Functions that are TRULY dead:
        let is_truly_dead = func.fan_in == 0
            && !func.is_public
            && !is_test
            && !func.file.contains("/.meta/")
            && !func.file.contains(".gen.go")
            && !func.file.contains("_gen.go")
            && func.trait_impl.is_none()
            && !func.file.contains("/dist/")
            && !func.file.contains("node_modules/")
            && !func.file.ends_with(".min.js")
            && !func.file.contains("/benches/");

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
                    0.80,
                    "truly_dead",
                );
            }
        }

        // Everything else becomes UNKNOWN
    }

    // Print stats
    println!("\n📊 Training Data Stats:");
    // ⭐ FIX: Use the GraphMetrics trait's node_count method
    println!("   Total functions: {}", analysis.call_graph.node_count());
    println!("   Labeled Alive: {}", collector.stats.alive_count);
    println!("   Labeled Dead: {}", collector.stats.dead_count);
    println!("   Unlabeled (Unknown): {}", collector.stats.unknown_count);
    println!("\n   By Language:");
    for (lang, count) in &collector.stats.by_language {
        println!("      {}: {}", lang, count);
    }

    // Save to file
    let json = collector.to_json()?;
    std::fs::write(&output_file, json)?;
    println!("\n✅ Training data saved to: {:?}", output_file);

    // Also save as JSONL for ML training
    let jsonl_path = output_file.with_extension("jsonl");
    std::fs::write(&jsonl_path, collector.to_jsonl())?;
    println!("✅ JSONL format saved to: {:?}", jsonl_path);

    // Show sample
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
        println!("      - In test file: {}", first.features.is_in_test_file);
    }

    Ok(())
}
