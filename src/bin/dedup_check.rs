// src/bin/dedup_check.rs

use code_intelligence::{optimize::Deduplicator, Pipeline};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dedup_check <project_path>");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    if !path.is_dir() {
        eprintln!("Error: {} is not a directory", args[1]);
        std::process::exit(1);
    }

    println!("🔍 Analyzing project: {:?}\n", path);
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&path).await?;
    let dedup = Deduplicator::new();
    let result = dedup.find_duplicates(&analysis.call_graph, &analysis.files);

    println!("📊 Deduplication Report");
    println!("=======================\n");
    println!("Duplicate groups found: {}", result.duplicate_groups.len());
    println!("Total token savings: ~{}\n", result.total_saved_tokens);
    println!(
        "Confidence score: {:.1}%\n",
        result.accuracy_metrics.confidence_score * 100.0
    );

    if result.duplicate_groups.is_empty() {
        println!("✅ No duplicate code found! Great job!");
    } else {
        println!("🔍 Duplicate Groups:\n");
        for (i, group) in result.duplicate_groups.iter().enumerate() {
            println!(
                "Group {} ({} functions, similarity: {:.1}%):",
                i + 1,
                group.functions.len(),
                group.similarity_score * 100.0
            );
            println!("  Type: {:?}", group.duplicate_type);
            println!("  Suggestion: {}", group.refactoring_suggestion);
            println!("  Functions:");
            for func in &group.functions {
                println!("    - {} ({}:{})", func.name, func.file, func.line);
            }
            println!();
        }
    }

    Ok(())
}
