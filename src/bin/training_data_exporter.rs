// src/bin/training_data_exporter.rs

use code_intelligence::analysis::dead_code::WHITELIST; // Removed DeadCodeAnalyzer
use code_intelligence::analysis::training_data::TrainingDataCollector;
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

    // Collect with whitelist logic
    collector.collect_from_analysis(
        &analysis.call_graph,
        |func| WHITELIST.is_whitelisted(&func.name) || WHITELIST.is_whitelisted_path(&func.file),
        |func| func.fan_in == 0 && !func.is_public && !WHITELIST.is_whitelisted(&func.name),
    );

    // Print stats
    println!("\n📊 Training Data Stats:");
    println!("   Total functions: {}", collector.stats.total_functions);
    println!("   Alive: {}", collector.stats.alive_count);
    println!("   Dead: {}", collector.stats.dead_count);
    println!("   Unknown: {}", collector.stats.unknown_count);
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
