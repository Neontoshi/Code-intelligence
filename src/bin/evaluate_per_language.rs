// src/bin/evaluate_per_language.rs

mod common;

use clap::Parser;
use code_intelligence::analysis::training_data::TrainingExample;
use code_intelligence::DeadCodeClassifier;
use common::metrics::evaluate;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct EvalArgs {
    /// Model file path
    #[arg(short, long, default_value = "model.bin")]
    model: PathBuf,

    /// Test data file
    #[arg(short, long, default_value = "test.json")]
    test_data: PathBuf,

    /// Validation data file (optional)
    #[arg(short, long)]
    val_data: Option<PathBuf>,

    /// Output detailed metrics
    #[arg(long)]
    detailed: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = EvalArgs::parse();

    println!("🔬 Loading model from: {:?}", args.model);
    let classifier = DeadCodeClassifier::load(&args.model.to_string_lossy())?;

    // Load test data (should be from repositories NOT seen during training)
    println!("📊 Loading test data from: {:?}", args.test_data);
    let test_data = std::fs::read_to_string(&args.test_data)?;
    let test_examples: Vec<TrainingExample> = serde_json::from_str(&test_data)?;

    // Evaluate on test set
    let test_metrics = evaluate(&classifier, &test_examples);
    println!("\n📈 Test Set Metrics:");
    test_metrics.print();

    // Optionally evaluate on validation set
    if let Some(val_path) = args.val_data {
        println!("\n📊 Loading validation data from: {:?}", val_path);
        let val_data = std::fs::read_to_string(&val_path)?;
        let val_examples: Vec<TrainingExample> = serde_json::from_str(&val_data)?;
        let val_metrics = evaluate(&classifier, &val_examples);
        println!("\n📈 Validation Set Metrics:");
        val_metrics.print();
    }

    // Per-language breakdown
    println!("\n🌐 Per-Language Breakdown:");
    let by_language = group_by_language(&test_examples);
    for (lang, examples) in by_language {
        // Convert Vec<&TrainingExample> to Vec<TrainingExample>
        let owned_examples: Vec<TrainingExample> = examples.into_iter().cloned().collect();
        let metrics = evaluate(&classifier, &owned_examples);
        println!("\n   {}:", lang);
        println!("     Accuracy: {:.1}%", metrics.accuracy * 100.0);
        println!("     Precision: {:.1}%", metrics.precision * 100.0);
        println!("     Recall: {:.1}%", metrics.recall * 100.0);
        println!("     F1: {:.1}%", metrics.f1 * 100.0);
        println!("     FPR: {:.1}%", metrics.fpr * 100.0);
    }

    // ⭐ Per-repository breakdown
    println!("\n📁 Per-Repository Breakdown:");
    let by_repo = group_by_repository(&test_examples);
    for (repo, examples) in by_repo {
        let owned_examples: Vec<TrainingExample> = examples.into_iter().cloned().collect();
        let metrics = evaluate(&classifier, &owned_examples);
        println!("\n   {}:", repo);
        println!("     Precision: {:.1}%", metrics.precision * 100.0);
        println!("     Recall: {:.1}%", metrics.recall * 100.0);
        println!("     F1: {:.1}%", metrics.f1 * 100.0);
        println!("     Examples: {}", owned_examples.len());
    }

    Ok(())
}

fn group_by_language(examples: &[TrainingExample]) -> HashMap<String, Vec<&TrainingExample>> {
    let mut groups: HashMap<String, Vec<&TrainingExample>> = HashMap::new();
    for example in examples {
        groups
            .entry(example.language.clone())
            .or_default()
            .push(example);
    }
    groups
}

fn group_by_repository(examples: &[TrainingExample]) -> HashMap<String, Vec<&TrainingExample>> {
    let mut groups: HashMap<String, Vec<&TrainingExample>> = HashMap::new();
    for example in examples {
        if let Some(ref repo) = example.repository_id {
            groups.entry(repo.clone()).or_default().push(example);
        }
    }
    groups
}
