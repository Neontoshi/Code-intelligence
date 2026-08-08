// src/bin/tune_threshold.rs

//! Find optimal threshold on validation set

mod common;

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::DeadCodeClassifier;
use common::metrics::EvaluationMetrics;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    /// Model file path
    #[arg(short, long, default_value = "model.bin")]
    model: PathBuf,

    /// Validation data file
    #[arg(short, long, default_value = "val.json")]
    val_data: PathBuf,

    /// Target precision (0.0-1.0)
    #[arg(long, default_value = "0.99")]
    target_precision: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔬 Loading model from: {:?}", args.model);
    let classifier = DeadCodeClassifier::load(&args.model.to_string_lossy())?;

    println!("📊 Loading validation data from: {:?}", args.val_data);
    let data = std::fs::read_to_string(&args.val_data)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    // Evaluate at different thresholds
    let thresholds: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();

    let mut results = Vec::new();
    for threshold in thresholds {
        let metrics = evaluate_at_threshold(&classifier, &examples, threshold);
        results.push((threshold, metrics));
    }

    // Find best threshold for target precision
    let best = results
        .iter()
        .filter(|(_, m)| m.precision >= args.target_precision)
        .max_by(|a, b| a.1.recall.partial_cmp(&b.1.recall).unwrap());

    if let Some((threshold, metrics)) = best {
        println!(
            "\n🎯 Optimal threshold for {:.1}% precision:",
            args.target_precision * 100.0
        );
        println!("   Threshold: {:.2}", threshold);
        println!("   Precision: {:.1}%", metrics.precision * 100.0);
        println!("   Recall: {:.1}%", metrics.recall * 100.0);
        println!("   F1: {:.1}%", metrics.f1 * 100.0);
        println!("   FPR: {:.1}%", metrics.fpr * 100.0);
    } else {
        println!(
            "\n⚠️  No threshold achieves {:.1}% precision on validation set.",
            args.target_precision * 100.0
        );
        println!(
            "   Best precision: {:.1}%",
            results.iter().map(|(_, m)| m.precision).fold(0.0, f64::max) * 100.0
        );
    }

    // Show full table
    println!("\n📊 Threshold vs Metrics:");
    println!("   Threshold | Precision | Recall | F1 | FPR");
    println!("   ----------|-----------|--------|----|-----");
    for (threshold, metrics) in results.iter().step_by(5) {
        println!(
            "   {:.2}       | {:.1}%     | {:.1}%   | {:.2} | {:.1}%",
            threshold,
            metrics.precision * 100.0,
            metrics.recall * 100.0,
            metrics.f1,
            metrics.fpr * 100.0
        );
    }

    Ok(())
}

fn evaluate_at_threshold(
    classifier: &DeadCodeClassifier,
    examples: &[TrainingExample],
    threshold: f64,
) -> EvaluationMetrics {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in examples {
        let prob = classifier.predict_probability(example);
        let prediction = if prob >= threshold {
            TrainingLabel::Dead
        } else {
            TrainingLabel::Alive
        };
        let actual = &example.label;

        match (prediction, actual) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            _ => {}
        }
    }

    let total = tp + tn + fp + fn_;
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
    let fpr = if fp + tn > 0 {
        fp as f64 / (fp + tn) as f64
    } else {
        0.0
    };

    EvaluationMetrics {
        total,
        correct: tp + tn,
        true_positives: tp,
        true_negatives: tn,
        false_positives: fp,
        false_negatives: fn_,
        accuracy: (tp + tn) as f64 / total as f64,
        precision,
        recall,
        f1,
        fpr,
        fnr: fn_ as f64 / (fn_ + tp) as f64,
        specificity: 1.0 - fpr,
    }
}
