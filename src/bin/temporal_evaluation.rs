// src/bin/temporal_evaluation.rs

//! Temporal evaluation - test model on time-based splits
//!
//! This tool evaluates how well the model performs on future code
//! by splitting data by commit timestamp.

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::DeadCodeClassifier;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about = "Temporal evaluation of dead code detection")]
struct Args {
    /// Model file path
    #[arg(short = 'm', long)]
    model: PathBuf,

    /// Training data file (for reference)
    #[arg(short = 'r', long, default_value = "data/train.json")]
    train_data: PathBuf,

    /// Test data file (should contain commit timestamps)
    #[arg(short = 'e', long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Output directory for results
    #[arg(short = 'o', long, default_value = "temporal_results")]
    output_dir: PathBuf,

    /// Number of time windows to evaluate
    #[arg(short = 'w', long, default_value = "5")]
    windows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalResult {
    pub window: String,
    pub start_date: String,
    pub end_date: String,
    pub examples: usize,
    pub alive_count: usize,
    pub dead_count: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub fpr: f64,
    pub threshold: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("📊 Temporal Evaluation");
    println!("=====================");
    println!();

    // Check if model exists
    if !args.model.exists() {
        eprintln!("❌ Model file not found: {:?}", args.model);
        eprintln!("   Please train a model first: cargo run --bin train_model");
        std::process::exit(1);
    }

    // Load model
    println!("📊 Loading model from: {:?}", args.model);
    let classifier = DeadCodeClassifier::load(&args.model.to_string_lossy())?;
    println!("   Model loaded successfully");

    // Load training data (for reference)
    let train_data = std::fs::read_to_string(&args.train_data)?;
    let train_examples: Vec<TrainingExample> = serde_json::from_str(&train_data)?;
    println!("   Training examples: {}", train_examples.len());

    // Load test data with timestamps
    let test_data = std::fs::read_to_string(&args.test_data)?;
    let test_examples: Vec<TrainingExample> = serde_json::from_str(&test_data)?;
    println!("   Test examples: {}", test_examples.len());

    // Group test examples by time
    let mut with_time: Vec<(TrainingExample, i64)> = test_examples
        .into_iter()
        .filter_map(|e| {
            // Try to get timestamp from commit_hash or repository_id
            let time = if let Some(ref hash) = e.commit_hash {
                parse_commit_time(hash).or_else(|| {
                    // Try to parse as timestamp directly
                    hash.parse::<i64>().ok()
                })
            } else {
                // Try repository_id as timestamp
                e.repository_id
                    .as_ref()
                    .and_then(|id| id.parse::<i64>().ok())
            };
            time.map(|t| (e, t))
        })
        .collect();

    if with_time.is_empty() {
        eprintln!("❌ No examples with commit timestamps found.");
        eprintln!("   To fix, run training_data_exporter with --git flag");
        eprintln!("   Or add commit_hash timestamps to your training data");
        std::process::exit(1);
    }

    // Sort by time
    with_time.sort_by(|a, b| a.1.cmp(&b.1));

    println!("   Examples with timestamps: {}", with_time.len());

    let total = with_time.len();
    let window_size = total / args.windows;

    if window_size == 0 {
        eprintln!(
            "❌ Not enough examples for {} windows (need at least {})",
            args.windows, args.windows
        );
        std::process::exit(1);
    }

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    let mut results = Vec::new();

    // Evaluate on each time window
    for i in 0..args.windows {
        let start_idx = i * window_size;
        let end_idx = if i == args.windows - 1 {
            total
        } else {
            (i + 1) * window_size
        };

        let window_examples: Vec<TrainingExample> = with_time[start_idx..end_idx]
            .iter()
            .map(|(e, _)| e.clone())
            .collect();

        let start_time = with_time[start_idx].1;
        let end_time = with_time[end_idx - 1].1;

        let metrics = evaluate(&classifier, &window_examples);

        let window_name = format!("Window {}", i + 1);
        println!("\n📊 {}: {} examples", window_name, window_examples.len());
        println!(
            "   Time range: {} → {}",
            format_timestamp(start_time),
            format_timestamp(end_time)
        );
        println!("   Accuracy: {:.1}%", metrics.accuracy * 100.0);
        println!("   Precision: {:.1}%", metrics.precision * 100.0);
        println!("   Recall: {:.1}%", metrics.recall * 100.0);
        println!("   F1: {:.1}%", metrics.f1 * 100.0);

        let alive_count = window_examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Alive)
            .count();
        let dead_count = window_examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Dead)
            .count();

        results.push(TemporalResult {
            window: window_name,
            start_date: format_timestamp(start_time),
            end_date: format_timestamp(end_time),
            examples: window_examples.len(),
            alive_count,
            dead_count,
            accuracy: metrics.accuracy,
            precision: metrics.precision,
            recall: metrics.recall,
            f1: metrics.f1,
            fpr: metrics.fpr,
            threshold: 0.5,
        });
    }

    // Save results
    let results_path = args.output_dir.join("temporal_results.json");
    std::fs::write(&results_path, serde_json::to_string_pretty(&results)?)?;

    // Generate markdown report
    generate_markdown_report(&results, &args.output_dir)?;

    // Check for performance degradation
    println!("\n📊 Temporal Analysis:");
    if results.len() >= 2 {
        let first_f1 = results[0].f1;
        let last_f1 = results[results.len() - 1].f1;
        let degradation = (first_f1 - last_f1) * 100.0;

        println!("   First window F1: {:.1}%", first_f1 * 100.0);
        println!("   Last window F1: {:.1}%", last_f1 * 100.0);
        if degradation > 5.0 {
            println!(
                "   ⚠️ F1 dropped by {:.1}% over time - model may not generalize to future code",
                degradation
            );
        } else {
            println!("   ✅ F1 stable over time ({:.1}% change)", degradation);
        }
    } else {
        println!("   Not enough windows for temporal analysis");
    }

    println!("\n📁 Results saved to: {:?}", args.output_dir);
    println!("   - temporal_results.json");
    println!("   - temporal_report.md");

    Ok(())
}

/// Parse a commit hash to get timestamp
fn parse_commit_time(commit_hash: &str) -> Option<i64> {
    // Check if it's already a timestamp (digits only, 10+ digits)
    if commit_hash.chars().all(|c| c.is_ascii_digit()) && commit_hash.len() >= 10 {
        return commit_hash.parse::<i64>().ok();
    }

    // Try to get commit timestamp from git
    // Only if it looks like a git hash (40 chars hex)
    if commit_hash.chars().all(|c| c.is_ascii_hexdigit()) && commit_hash.len() == 40 {
        let output = Command::new("git")
            .args(["show", "-s", "--format=%ct", commit_hash])
            .output()
            .ok()?;

        if output.status.success() {
            let ts_str = String::from_utf8_lossy(&output.stdout);
            return ts_str.trim().parse::<i64>().ok();
        }
    }

    // Try to parse as a timestamp (milliseconds or seconds)
    if let Ok(ts) = commit_hash.parse::<i64>() {
        // If it's in milliseconds (13 digits), convert to seconds
        if ts > 1_000_000_000_000 {
            return Some(ts / 1000);
        }
        return Some(ts);
    }

    None
}

fn format_timestamp(ts: i64) -> String {
    if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        ts.to_string()
    }
}

/// Evaluation metrics
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct EvaluationMetrics {
    total: usize,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    fpr: f64,
    threshold: f64,
}

fn evaluate(classifier: &DeadCodeClassifier, examples: &[TrainingExample]) -> EvaluationMetrics {
    let labeled: Vec<_> = examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .collect();

    if labeled.is_empty() {
        return EvaluationMetrics {
            total: 0,
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1: 0.0,
            fpr: 0.0,
            threshold: 0.5,
        };
    }

    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in &labeled {
        let prediction = classifier.predict(example);
        let actual = &example.label;

        match (prediction, actual) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
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
        accuracy: (tp + tn) as f64 / total as f64,
        precision,
        recall,
        f1,
        fpr,
        threshold: 0.5,
    }
}

fn generate_markdown_report(
    results: &[TemporalResult],
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut markdown = String::new();

    markdown.push_str("# 📊 Temporal Evaluation Report\n\n");
    markdown.push_str("## Summary\n\n");

    markdown.push_str(
        "| Window | Date Range | Examples | Alive | Dead | Accuracy | Precision | Recall | F1 |\n",
    );
    markdown.push_str(
        "|--------|------------|----------|-------|------|----------|-----------|--------|-----|\n",
    );

    for r in results {
        markdown.push_str(&format!(
            "| {} | {} → {} | {} | {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
            r.window,
            r.start_date,
            r.end_date,
            r.examples,
            r.alive_count,
            r.dead_count,
            r.accuracy * 100.0,
            r.precision * 100.0,
            r.recall * 100.0,
            r.f1 * 100.0
        ));
    }

    markdown.push_str("\n## Analysis\n\n");

    if results.len() >= 2 {
        let first_f1 = results[0].f1;
        let last_f1 = results[results.len() - 1].f1;
        let degradation = (first_f1 - last_f1) * 100.0;

        if degradation > 5.0 {
            markdown.push_str(&format!(
                "⚠️ **Performance degraded by {:.1}% over time.**\n\n",
                degradation
            ));
            markdown.push_str("The model's performance drops on newer code. This suggests:\n");
            markdown.push_str("1. Code patterns are evolving\n");
            markdown.push_str("2. The model needs more recent training data\n");
            markdown.push_str("3. Consider retraining on more recent code\n");
        } else if degradation > 0.0 {
            markdown.push_str(&format!(
                "📉 **Performance slightly decreased by {:.1}% over time.**\n\n",
                degradation
            ));
        } else {
            markdown.push_str("✅ **Performance stable over time.**\n\n");
        }
    }

    markdown.push_str("\n## Recommendations\n\n");

    if let Some(last) = results.last() {
        if last.f1 > 0.85 {
            markdown.push_str("✅ Model shows strong performance on recent code.\n");
        } else if last.f1 > 0.70 {
            markdown
                .push_str("📌 Model performs adequately on recent code but could be improved.\n");
        } else {
            markdown
                .push_str("🔴 Model performs poorly on recent code. Retraining is recommended.\n");
        }
    }

    markdown.push_str("\n---\n");
    markdown.push_str(&format!(
        "*Report generated on {}*\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    let report_path = output_dir.join("temporal_report.md");
    std::fs::write(&report_path, markdown)?;

    Ok(())
}
