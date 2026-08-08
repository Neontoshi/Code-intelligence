// src/bin/model_comparison.rs

//! Model comparison - compare different ML algorithms
//!
//! This tool compares different ML algorithms on the dead code detection task.

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::LinearClassifier;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
struct Args {
    /// Training data file
    #[arg(short, long, default_value = "data/train.json")]
    train_data: PathBuf,

    /// Validation data file
    #[arg(short, long, default_value = "data/val.json")]
    val_data: PathBuf,

    /// Test data file
    #[arg(short, long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Output directory for results
    #[arg(short, long, default_value = "model_comparison")]
    output_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ModelResult {
    name: String,
    train_accuracy: f64,
    val_accuracy: f64,
    test_accuracy: f64,
    test_precision: f64,
    test_recall: f64,
    test_f1: f64,
    train_time_ms: u64,
    inference_time_ms: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🧠 Model Comparison");
    println!("==================\n");

    // Load data
    println!("📊 Loading training data from: {:?}", args.train_data);
    let train_data = std::fs::read_to_string(&args.train_data)?;
    let train_examples: Vec<TrainingExample> = serde_json::from_str(&train_data)?;

    println!("📊 Loading validation data from: {:?}", args.val_data);
    let val_data = std::fs::read_to_string(&args.val_data)?;
    let val_examples: Vec<TrainingExample> = serde_json::from_str(&val_data)?;

    println!("📊 Loading test data from: {:?}", args.test_data);
    let test_data = std::fs::read_to_string(&args.test_data)?;
    let test_examples: Vec<TrainingExample> = serde_json::from_str(&test_data)?;

    println!("   Train: {} examples", train_examples.len());
    println!("   Val: {} examples", val_examples.len());
    println!("   Test: {} examples\n", test_examples.len());

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    let mut results = Vec::new();

    // 1. Logistic Regression (our current model)
    println!("🧪 Training: Logistic Regression");
    let start = Instant::now();
    let mut lr = LinearClassifier::new_with_schema()
        .with_learning_rate(0.01)
        .with_epochs(50);
    let train_acc = lr.train(&train_examples);
    let train_time = start.elapsed().as_millis() as u64;

    // Evaluate on validation
    let val_acc = evaluate_classifier(&lr, &val_examples);

    // Evaluate on test
    let test_metrics = evaluate_classifier_full(&lr, &test_examples);

    results.push(ModelResult {
        name: "Logistic Regression".to_string(),
        train_accuracy: train_acc,
        val_accuracy: val_acc,
        test_accuracy: test_metrics.accuracy,
        test_precision: test_metrics.precision,
        test_recall: test_metrics.recall,
        test_f1: test_metrics.f1,
        train_time_ms: train_time,
        inference_time_ms: 0,
    });

    println!(
        "   Train Acc: {:.1}%, Val Acc: {:.1}%, Test Acc: {:.1}%",
        train_acc * 100.0,
        val_acc * 100.0,
        test_metrics.accuracy * 100.0
    );

    // Note: For Random Forest and Gradient Boosting, we'd need to add
    // dependencies. For now, we show what would be possible.
    println!("\n💡 To compare with Random Forest and Gradient Boosting:");
    println!("   Add linfa-trees and linfa-glm to dependencies");
    println!("   The current best model is: Logistic Regression");
    println!("   F1: {:.1}%", test_metrics.f1 * 100.0);

    // Save results
    save_results(&results, &args.output_dir)?;

    // Print summary
    print_summary_table(&results);

    Ok(())
}

#[derive(Debug, Clone)]
struct FullMetrics {
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
}

fn evaluate_classifier(classifier: &LinearClassifier, examples: &[TrainingExample]) -> f64 {
    let labeled: Vec<_> = examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .collect();

    if labeled.is_empty() {
        return 0.0;
    }

    let mut correct = 0;
    for example in &labeled {
        let features = example.features.to_feature_vector();
        let pred = classifier.predict_label(&features);
        if pred == example.label {
            correct += 1;
        }
    }

    correct as f64 / labeled.len() as f64
}

fn evaluate_classifier_full(
    classifier: &LinearClassifier,
    examples: &[TrainingExample],
) -> FullMetrics {
    let labeled: Vec<_> = examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .collect();

    if labeled.is_empty() {
        return FullMetrics {
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1: 0.0,
        };
    }

    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in &labeled {
        let features = example.features.to_feature_vector();
        let pred = classifier.predict_label(&features);

        match (pred, &example.label) {
            (TrainingLabel::Alive, TrainingLabel::Alive) => tp += 1,
            (TrainingLabel::Dead, TrainingLabel::Dead) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }
    }

    let total = tp + tn + fp + fn_;
    let accuracy = if total > 0 {
        (tp + tn) as f64 / total as f64
    } else {
        0.0
    };
    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    FullMetrics {
        accuracy,
        precision,
        recall,
        f1,
    }
}

fn save_results(
    results: &[ModelResult],
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_path = output_dir.join("model_comparison.json");
    std::fs::write(&json_path, serde_json::to_string_pretty(results)?)?;
    println!("\n📁 Results saved to: {:?}", json_path);

    let csv_path = output_dir.join("model_comparison.csv");
    let mut csv = String::new();
    csv.push_str("Name,TrainAcc,ValAcc,TestAcc,TestPrec,TestRecall,TestF1,TrainTimeMs\n");
    for r in results {
        csv.push_str(&format!(
            "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{}\n",
            r.name,
            r.train_accuracy,
            r.val_accuracy,
            r.test_accuracy,
            r.test_precision,
            r.test_recall,
            r.test_f1,
            r.train_time_ms
        ));
    }
    std::fs::write(&csv_path, csv)?;
    println!("📁 CSV saved to: {:?}", csv_path);

    Ok(())
}

fn print_summary_table(results: &[ModelResult]) {
    println!("\n📊 Model Comparison Summary:");
    println!(
        "   {:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8}",
        "Model", "Train", "Val", "Test", "Prec", "F1"
    );
    println!(
        "   {:-<25}-+-{:->8}-+-{:->8}-+-{:->8}-+-{:->8}-+-{:->8}",
        "", "", "", "", "", ""
    );

    for r in results {
        println!(
            "   {:<25} | {:>7.1}% | {:>7.1}% | {:>7.1}% | {:>7.1}% | {:>7.1}%",
            r.name,
            r.train_accuracy * 100.0,
            r.val_accuracy * 100.0,
            r.test_accuracy * 100.0,
            r.test_precision * 100.0,
            r.test_f1 * 100.0
        );
    }
}
