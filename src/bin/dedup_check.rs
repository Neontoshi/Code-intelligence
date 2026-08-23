// src/bin/dedup_check.rs

use code_intelligence::{
    error::{err, Result},
    ml::duplicate_classifier::DuplicateClassifier,
    optimize::Deduplicator,
    Pipeline,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: dedup_check <project_path> [--threshold <0.0-1.0>] [--duplicate-model <model.bin>]");
        eprintln!("");
        eprintln!("Examples:");
        eprintln!("  dedup_check ~/Documents/Kyma");
        eprintln!("  dedup_check . --threshold 0.80");
        eprintln!("  dedup_check . --duplicate-model models/duplicate_model.bin");
        return Err(err::config("Missing project path argument"));
    }

    let project_path = PathBuf::from(&args[1]);

    let mut model_path = None;
    let mut threshold = 0.85;
    let mut i = 2;

    while i < args.len() {
        match args[i].as_str() {
            "--threshold" | "-t" => {
                if i + 1 < args.len() {
                    threshold = args[i + 1]
                        .parse()
                        .map_err(|e| err::config(format!("Invalid threshold: {}", e)))?;
                    i += 2;
                } else {
                    return Err(err::config(
                        "--threshold requires a float value between 0.0 and 1.0",
                    ));
                }
            }
            "--duplicate-model" | "-m" => {
                if i + 1 < args.len() {
                    model_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    return Err(err::config("--duplicate-model requires a file path"));
                }
            }
            _ => {
                return Err(err::config(format!("Unknown argument: {}", args[i])));
            }
        }
    }

    if !project_path.is_dir() {
        return Err(err::analysis(format!(
            "{} is not a directory",
            project_path.display()
        )));
    }

    println!("🔍 Analyzing project: {:?}\n", project_path);

    let duplicate_model = if let Some(path) = model_path {
        if path.exists() {
            match DuplicateClassifier::load(&*path.to_string_lossy()) {
                Ok(model) => {
                    println!("✅ Loaded duplicate model from: {:?}\n", path);
                    Some(model)
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to load duplicate model: {}", e);
                    eprintln!("   Continuing without ML support.\n");
                    None
                }
            }
        } else {
            eprintln!("⚠️ Model file not found: {:?}", path);
            eprintln!("   Continuing without ML support.\n");
            None
        }
    } else {
        None
    };

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&project_path).await?;

    let mut dedup = if let Some(model) = duplicate_model {
        Deduplicator::new_with_ml(Some(model))
    } else {
        Deduplicator::new()
    };
    dedup = dedup.with_threshold(threshold);

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
            let ml_indicator = if group.confidence_score > 0.0 {
                format!(" (ML: {:.1}%)", group.confidence_score * 100.0)
            } else {
                String::new()
            };

            println!(
                "Group {} ({} functions, similarity: {:.1}%{}):",
                i + 1,
                group.functions.len(),
                group.similarity_score * 100.0,
                ml_indicator
            );
            println!("  Type: {:?}", group.duplicate_type);
            println!("  Suggestion: {}", group.refactoring_suggestion);
            println!("  Priority: {:.1}%", group.priority_score * 100.0);
            println!("  Token savings: ~{}", group.total_token_savings);
            println!("  Functions:");
            for func in &group.functions {
                println!("    - {} ({}:{})", func.name, func.file, func.line);
            }
            println!();
        }
    }

    Ok(())
}
