// src/bin/temporal_evaluation.rs

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::error::Result;
use code_intelligence::ml::classifier::DeadCodeClassifier;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Temporal evaluation - train on past, test on future"
)]
struct Args {
    /// Test data file (should contain commit timestamps)
    #[arg(short = 'e', long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Output directory for results
    #[arg(short = 'o', long, default_value = "temporal_results")]
    output_dir: PathBuf,

    /// Number of time windows to evaluate
    #[arg(short = 'w', long, default_value = "5")]
    windows: usize,

    /// Minimum examples per window
    #[arg(long, default_value = "100")]
    min_examples: usize,

    /// Seed for reproducibility
    #[arg(long, default_value = "42")]
    seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalResult {
    pub window: String,
    pub train_start: String,
    pub train_end: String,
    pub test_start: String,
    pub test_end: String,
    pub train_examples: usize,
    pub test_examples: usize,
    pub train_alive: usize,
    pub train_dead: usize,
    pub test_alive: usize,
    pub test_dead: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub fpr: f64,
    pub threshold: f64,
    pub train_accuracy: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("📊 Temporal Evaluation");
    println!("=====================");
    println!();
    println!("⏰ Train on PAST data, test on FUTURE data");
    println!("   This measures how well the model generalizes to new code.\n");

    // Load test data with timestamps
    let test_data = std::fs::read_to_string(&args.test_data)?;
    let all_examples: Vec<TrainingExample> = serde_json::from_str(&test_data)?;
    println!("📊 Loaded {} total examples", all_examples.len());

    // Get timestamps for all examples
    let mut with_time: Vec<(TrainingExample, i64)> = all_examples
        .into_iter()
        .filter_map(|e| {
            let time = if let Some(ref hash) = e.commit_hash {
                parse_commit_time(hash).or_else(|| hash.parse::<i64>().ok())
            } else {
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

    // Sort by time (oldest first)
    with_time.sort_by(|a, b| a.1.cmp(&b.1));

    println!("   Examples with timestamps: {}", with_time.len());
    println!(
        "   Oldest: {}",
        format_timestamp(with_time.first().unwrap().1)
    );
    println!(
        "   Newest: {}",
        format_timestamp(with_time.last().unwrap().1)
    );

    let total = with_time.len();
    let window_size = total / args.windows;

    if window_size < args.min_examples {
        eprintln!(
            "❌ Not enough examples per window. Need at least {}, got {}",
            args.min_examples, window_size
        );
        std::process::exit(1);
    }

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    let mut results = Vec::new();

    // For each window, train on ALL data before it, test on the window
    for i in 0..args.windows {
        let test_start_idx = i * window_size;
        let test_end_idx = if i == args.windows - 1 {
            total
        } else {
            (i + 1) * window_size
        };

        // Training data = ALL data before this window
        let train_examples: Vec<TrainingExample> = with_time[0..test_start_idx]
            .iter()
            .map(|(e, _)| e.clone())
            .collect();

        // Test data = this window
        let test_examples: Vec<TrainingExample> = with_time[test_start_idx..test_end_idx]
            .iter()
            .map(|(e, _)| e.clone())
            .collect();

        // Skip if not enough training data
        if train_examples.len() < args.min_examples {
            println!(
                "\n⚠️  Window {}: Not enough training data ({}), skipping",
                i + 1,
                train_examples.len()
            );
            continue;
        }

        let train_start = with_time[0].1;
        let train_end = with_time[test_start_idx - 1].1;
        let test_start = with_time[test_start_idx].1;
        let test_end = with_time[test_end_idx - 1].1;

        println!("\n📊 Window {}:", i + 1);
        println!(
            "   Training: {} examples ({})",
            train_examples.len(),
            format!(
                "{} → {}",
                format_timestamp(train_start),
                format_timestamp(train_end)
            )
        );
        println!(
            "   Testing:  {} examples ({})",
            test_examples.len(),
            format!(
                "{} → {}",
                format_timestamp(test_start),
                format_timestamp(test_end)
            )
        );

        // Train a fresh model on the training data
        let mut classifier = DeadCodeClassifier::new();
        let train_result = classifier.train(&train_examples);

        // Get training accuracy
        let train_accuracy = if let Ok(_) = train_result {
            classifier.get_accuracy()
        } else {
            0.0
        };

        // Evaluate on test data
        let test_metrics = evaluate(&classifier, &test_examples);

        // Count labels
        let train_alive = train_examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Alive)
            .count();
        let train_dead = train_examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Dead)
            .count();
        let test_alive = test_examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Alive)
            .count();
        let test_dead = test_examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Dead)
            .count();

        println!(
            "   Train Acc: {:.1}%, Test Acc: {:.1}%, F1: {:.1}%",
            train_accuracy * 100.0,
            test_metrics.accuracy * 100.0,
            test_metrics.f1 * 100.0
        );

        results.push(TemporalResult {
            window: format!("Window {}", i + 1),
            train_start: format_timestamp(train_start),
            train_end: format_timestamp(train_end),
            test_start: format_timestamp(test_start),
            test_end: format_timestamp(test_end),
            train_examples: train_examples.len(),
            test_examples: test_examples.len(),
            train_alive,
            train_dead,
            test_alive,
            test_dead,
            accuracy: test_metrics.accuracy,
            precision: test_metrics.precision,
            recall: test_metrics.recall,
            f1: test_metrics.f1,
            fpr: test_metrics.fpr,
            threshold: 0.5,
            train_accuracy,
        });
    }

    // Save results
    let results_path = args.output_dir.join("temporal_results.json");
    std::fs::write(&results_path, serde_json::to_string_pretty(&results)?)?;

    // Generate markdown report
    generate_markdown_report(&results, &args.output_dir)?;

    // Detailed temporal analysis
    println!("\n📊 Temporal Generalization Analysis:");
    if results.len() >= 2 {
        let first_f1 = results[0].f1;
        let last_f1 = results[results.len() - 1].f1;
        let degradation = (first_f1 - last_f1) * 100.0;

        let first_acc = results[0].accuracy;
        let last_acc = results[results.len() - 1].accuracy;
        let acc_degradation = (first_acc - last_acc) * 100.0;

        println!("   First window F1:  {:.1}%", first_f1 * 100.0);
        println!("   Last window F1:   {:.1}%", last_f1 * 100.0);
        println!("   F1 change:        {:.1}%", degradation);
        println!("   First window Acc: {:.1}%", first_acc * 100.0);
        println!("   Last window Acc:  {:.1}%", last_acc * 100.0);
        println!("   Acc change:       {:.1}%", acc_degradation);

        // Check if performance drop is significant
        if degradation > 5.0 {
            println!(
                "\n   ⚠️  WARNING: F1 dropped by {:.1}% over time!",
                degradation
            );
            println!("   The model may not generalize well to future code.");
            println!("   Consider:");
            println!("   - Retraining on more recent data");
            println!("   - Using time-based cross-validation");
            println!("   - Adding temporal features to the model");
        } else if degradation > 0.0 {
            println!("\n   📉 F1 decreased slightly by {:.1}%", degradation);
            println!("   Model shows mild temporal degradation.");
        } else {
            println!("\n   ✅ F1 stable or improved over time!");
            println!("   Model generalizes well to future code.");
        }
    } else {
        println!("   Not enough windows for temporal analysis (need at least 2)");
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

/// Format a timestamp to a readable string
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

/// Evaluate a classifier on examples
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

/// Generate a markdown report from results
fn generate_markdown_report(results: &[TemporalResult], output_dir: &PathBuf) -> Result<()> {
    let mut markdown = String::new();

    markdown.push_str("# 📊 Temporal Generalization Report\n\n");
    markdown.push_str("## Summary\n\n");
    markdown.push_str("This report measures how well the model generalizes to **future code**.\n");
    markdown
        .push_str("Each window: Train on data from **before** the window, test on the window.\n\n");

    markdown.push_str(
        "| Window | Train Period | Test Period | Train Examples | Test Examples | Train Acc | Test Acc | F1 |\n",
    );
    markdown.push_str(
        "|--------|--------------|-------------|----------------|---------------|-----------|----------|----|\n",
    );

    for r in results {
        markdown.push_str(&format!(
            "| {} | {} → {} | {} → {} | {} | {} | {:.1}% | {:.1}% | {:.1}% |\n",
            r.window,
            r.train_start,
            r.train_end,
            r.test_start,
            r.test_end,
            r.train_examples,
            r.test_examples,
            r.train_accuracy * 100.0,
            r.accuracy * 100.0,
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
                "⚠️ **F1 dropped by {:.1}% over time.**\n\n",
                degradation
            ));
            markdown.push_str("The model's performance degrades on newer code:\n");
            markdown.push_str("1. Code patterns are evolving over time\n");
            markdown.push_str("2. The model needs more recent training data\n");
            markdown.push_str("3. Consider retraining on a rolling window of data\n");
        } else if degradation > 0.0 {
            markdown.push_str(&format!(
                "📉 **F1 decreased slightly by {:.1}% over time.**\n\n",
                degradation
            ));
            markdown.push_str("The model shows mild temporal degradation.\n");
            markdown.push_str("Consider periodic retraining to maintain performance.\n");
        } else {
            markdown.push_str("✅ **F1 stable or improved over time.**\n\n");
            markdown.push_str("The model generalizes well to future code.\n");
        }
    }

    markdown.push_str("\n## Recommendations\n\n");

    if let Some(last) = results.last() {
        if last.f1 > 0.85 {
            markdown.push_str("✅ Model generalizes well to recent code.\n");
        } else if last.f1 > 0.70 {
            markdown
                .push_str("📌 Model performs adequately on recent code but could be improved.\n");
        } else {
            markdown.push_str(
                "🔴 Model performs poorly on recent code. Retraining is strongly recommended.\n",
            );
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
