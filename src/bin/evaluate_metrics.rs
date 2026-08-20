// src/bin/evaluate_metrics.rs

//! Comprehensive evaluation with Precision, Recall, F1, PR-AUC

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::DeadCodeClassifier;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Comprehensive model evaluation")]
struct Args {
    /// Model file path
    #[arg(short, long)]
    model: PathBuf,

    /// Test data file
    #[arg(short, long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Output file for results
    #[arg(short, long, default_value = "evaluation_results.json")]
    output: PathBuf,

    /// Generate detailed report
    #[arg(long)]
    detailed: bool,

    /// Top-K precision to compute (K values)
    #[arg(long, default_value = "10,25,50,100")]
    top_k: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    pub total: usize,
    pub correct: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub fpr: f64,
    pub fnr: f64,
    pub specificity: f64,
    pub confusion_matrix: ConfusionMatrix,
    pub top_k_precision: Vec<TopKPrecision>,
    pub auc_pr: f64,
    pub auc_roc: f64,
    pub threshold: f64,
    pub calibration: CalibrationStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    pub tp: usize,  // True Positive (predicted Dead, actual Dead)
    pub tn: usize,  // True Negative (predicted Alive, actual Alive)
    pub fp: usize,  // False Positive (predicted Dead, actual Alive)
    pub fn_: usize, // False Negative (predicted Alive, actual Dead)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopKPrecision {
    pub k: usize,
    pub precision: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationStats {
    pub expected_calibration_error: f64,
    pub max_calibration_error: f64,
    pub brier_score: f64,
    pub bins: Vec<CalibrationBin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBin {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
    pub accuracy: f64,
    pub avg_confidence: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("📊 Model Evaluation");
    println!("==================");

    // Load model
    println!("📊 Loading model from: {:?}", args.model);
    let classifier = DeadCodeClassifier::load(&args.model.to_string_lossy())?;
    println!("   Model loaded successfully");

    // Load test data
    println!("📊 Loading test data from: {:?}", args.test_data);
    let data = std::fs::read_to_string(&args.test_data)?;
    let test_examples: Vec<TrainingExample> = serde_json::from_str(&data)?;
    println!("   Test examples: {}", test_examples.len());

    // Parse top-K values
    let top_k_values: Vec<usize> = args
        .top_k
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // Run evaluation
    let metrics = evaluate(&classifier, &test_examples, &top_k_values);

    // Save results
    std::fs::write(&args.output, serde_json::to_string_pretty(&metrics)?)?;

    // Print summary
    print_summary(&metrics, args.detailed);

    println!("\n📁 Results saved to: {:?}", args.output);

    // Generate markdown report if detailed
    if args.detailed {
        generate_markdown_report(&metrics, &args.output)?;
    }

    Ok(())
}

fn evaluate(
    classifier: &DeadCodeClassifier,
    examples: &[TrainingExample],
    top_k_values: &[usize],
) -> EvaluationMetrics {
    let labeled: Vec<_> = examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .collect();

    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    // For calibration and PR-AUC
    let mut predictions = Vec::new();
    let mut confidences = Vec::new();

    for example in &labeled {
        let alive_prob = classifier.predict_probability(example);
        let dead_prob = 1.0 - alive_prob;
        let prediction = if dead_prob > 0.5 {
            TrainingLabel::Dead
        } else {
            TrainingLabel::Alive
        };
        let actual = &example.label;

        // DEAD is positive class
        match (prediction, actual) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }

        // Store for calibration
        predictions.push(dead_prob);
        confidences.push(match actual {
            TrainingLabel::Dead => 1.0,
            TrainingLabel::Alive => 0.0,
            _ => 0.5,
        });
    }

    let total = tp + tn + fp + fn_;
    let correct = tp + tn;

    let accuracy = if total > 0 {
        correct as f64 / total as f64
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
    let fpr = if fp + tn > 0 {
        fp as f64 / (fp + tn) as f64
    } else {
        0.0
    };
    let fnr = if fn_ + tp > 0 {
        fn_ as f64 / (fn_ + tp) as f64
    } else {
        0.0
    };
    let specificity = 1.0 - fpr;

    // Compute PR-AUC (simplified approximation)
    let auc_pr = compute_pr_auc(&predictions, &confidences);

    // Compute ROC-AUC (simplified)
    let auc_roc = compute_roc_auc(&predictions, &confidences);

    // Calibration
    let calibration = compute_calibration(&predictions, &confidences);

    // Top-K precision
    let mut top_k_precision = Vec::new();
    for &k in top_k_values {
        let precision_at_k = compute_precision_at_k(&predictions, &confidences, k);
        top_k_precision.push(TopKPrecision {
            k,
            precision: precision_at_k,
        });
    }

    EvaluationMetrics {
        total,
        correct,
        accuracy,
        precision,
        recall,
        f1,
        fpr,
        fnr,
        specificity,
        confusion_matrix: ConfusionMatrix { tp, tn, fp, fn_ },
        top_k_precision,
        auc_pr,
        auc_roc,
        threshold: 0.5,
        calibration,
    }
}

fn compute_pr_auc(predictions: &[f64], confidences: &[f64]) -> f64 {
    // Simple approximation using trapezoidal rule
    let mut points: Vec<(f64, f64)> = predictions
        .iter()
        .zip(confidences.iter())
        .map(|(&p, &c)| (p, c))
        .collect();
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut auc = 0.0;
    for i in 1..points.len() {
        let width = points[i].0 - points[i - 1].0;
        let height = (points[i].1 + points[i - 1].1) / 2.0;
        auc += width * height;
    }
    auc
}

fn compute_roc_auc(predictions: &[f64], confidences: &[f64]) -> f64 {
    // Simple approximation using trapezoidal rule
    let mut points: Vec<(f64, f64)> = predictions
        .iter()
        .zip(confidences.iter())
        .map(|(&p, &c)| (p, c))
        .collect();
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut auc = 0.0;
    for i in 1..points.len() {
        let width = points[i].0 - points[i - 1].0;
        let height = (points[i].1 + points[i - 1].1) / 2.0;
        auc += width * height;
    }
    auc
}

fn compute_calibration(predictions: &[f64], confidences: &[f64]) -> CalibrationStats {
    let num_bins = 10;
    let mut bins = vec![
        CalibrationBin {
            lower: 0.0,
            upper: 0.0,
            count: 0,
            accuracy: 0.0,
            avg_confidence: 0.0
        };
        num_bins
    ];

    let bin_width = 1.0 / num_bins as f64;
    let mut bin_confidences = vec![0.0; num_bins];
    let mut bin_accuracies = vec![0.0; num_bins];
    let mut bin_counts = vec![0; num_bins];

    for i in 0..predictions.len() {
        let pred = predictions[i];
        let conf = confidences[i];
        let bin_idx = ((pred / bin_width).floor() as usize).min(num_bins - 1);
        bin_confidences[bin_idx] += pred;
        bin_accuracies[bin_idx] += conf;
        bin_counts[bin_idx] += 1;
    }

    let mut ece = 0.0;
    let mut max_ce: f64 = 0.0;

    for i in 0..num_bins {
        let count = bin_counts[i];
        if count > 0 {
            let avg_conf = bin_confidences[i] / count as f64;
            let avg_acc = bin_accuracies[i] / count as f64;
            let ce = (avg_conf - avg_acc).abs();
            ece += ce * count as f64 / predictions.len() as f64;
            max_ce = max_ce.max(ce);

            bins[i] = CalibrationBin {
                lower: i as f64 * bin_width,
                upper: (i + 1) as f64 * bin_width,
                count,
                accuracy: avg_acc,
                avg_confidence: avg_conf,
            };
        }
    }

    // Brier score
    let brier_score: f64 = predictions
        .iter()
        .zip(confidences.iter())
        .map(|(&p, &c)| (p - c).powi(2))
        .sum::<f64>()
        / predictions.len() as f64;

    CalibrationStats {
        expected_calibration_error: ece,
        max_calibration_error: max_ce,
        brier_score,
        bins,
    }
}

fn compute_precision_at_k(predictions: &[f64], confidences: &[f64], k: usize) -> f64 {
    let mut pairs: Vec<(f64, f64)> = predictions
        .iter()
        .zip(confidences.iter())
        .map(|(&p, &c)| (p, c))
        .collect();
    pairs.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    });

    let top_k = pairs.iter().take(k).collect::<Vec<_>>();
    let positive = top_k.iter().filter(|&&(_, c)| *c == 1.0).count();

    if k > 0 {
        positive as f64 / k as f64
    } else {
        0.0
    }
}

fn print_summary(metrics: &EvaluationMetrics, _detailed: bool) {
    println!("\n📊 Evaluation Summary:");
    println!("   Total examples: {}", metrics.total);
    println!("   Correct: {}", metrics.correct);
    println!("\n   Accuracy: {:.1}%", metrics.accuracy * 100.0);
    println!("   Precision: {:.1}%", metrics.precision * 100.0);
    println!("   Recall: {:.1}%", metrics.recall * 100.0);
    println!("   F1: {:.1}%", metrics.f1 * 100.0);
    println!("   FPR: {:.1}%", metrics.fpr * 100.0);
    println!("   FNR: {:.1}%", metrics.fnr * 100.0);
    println!("   Specificity: {:.1}%", metrics.specificity * 100.0);

    println!("\n   Confusion Matrix (DEAD = Positive):");
    println!("              ACTUAL");
    println!("            Alive   Dead");
    println!(
        "   Pred Alive  {:>4}   {:>4}  ← False Negatives",
        metrics.confusion_matrix.tn, metrics.confusion_matrix.fn_
    );
    println!(
        "   Pred Dead   {:>4}   {:>4}  ← True Positives",
        metrics.confusion_matrix.fp, metrics.confusion_matrix.tp
    );

    println!("\n   PR-AUC: {:.3}", metrics.auc_pr);
    println!("   ROC-AUC: {:.3}", metrics.auc_roc);

    println!("\n   Calibration:");
    println!(
        "      ECE: {:.3}",
        metrics.calibration.expected_calibration_error
    );
    println!(
        "      Max CE: {:.3}",
        metrics.calibration.max_calibration_error
    );
    println!("      Brier Score: {:.3}", metrics.calibration.brier_score);

    if !metrics.top_k_precision.is_empty() {
        println!("\n   Top-K Precision:");
        for top_k in &metrics.top_k_precision {
            println!(
                "      Precision@{}: {:.1}%",
                top_k.k,
                top_k.precision * 100.0
            );
        }
    }
}

fn generate_markdown_report(
    metrics: &EvaluationMetrics,
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut markdown = String::new();

    markdown.push_str("# 📊 Model Evaluation Report\n\n");

    markdown.push_str("## Summary\n\n");
    markdown.push_str(&format!("- **Total examples**: {}\n", metrics.total));
    markdown.push_str(&format!("- **Correct predictions**: {}\n", metrics.correct));
    markdown.push_str(&format!(
        "- **Accuracy**: {:.1}%\n",
        metrics.accuracy * 100.0
    ));
    markdown.push_str(&format!(
        "- **Precision**: {:.1}%\n",
        metrics.precision * 100.0
    ));
    markdown.push_str(&format!("- **Recall**: {:.1}%\n", metrics.recall * 100.0));
    markdown.push_str(&format!("- **F1**: {:.1}%\n", metrics.f1 * 100.0));

    markdown.push_str("\n## Confusion Matrix\n\n");
    markdown.push_str("| | Alive (Actual) | Dead (Actual) |\n");
    markdown.push_str("|---|---|---|\n");
    markdown.push_str(&format!(
        "| **Alive (Pred)** | {} | {} |\n",
        metrics.confusion_matrix.tn, metrics.confusion_matrix.fn_
    ));
    markdown.push_str(&format!(
        "| **Dead (Pred)** | {} | {} |\n",
        metrics.confusion_matrix.fp, metrics.confusion_matrix.tp
    ));

    markdown.push_str("\n## Performance Metrics\n\n");
    markdown.push_str("| Metric | Value |\n");
    markdown.push_str("|--------|-------|\n");
    markdown.push_str(&format!(
        "| Accuracy | {:.1}% |\n",
        metrics.accuracy * 100.0
    ));
    markdown.push_str(&format!(
        "| Precision | {:.1}% |\n",
        metrics.precision * 100.0
    ));
    markdown.push_str(&format!("| Recall | {:.1}% |\n", metrics.recall * 100.0));
    markdown.push_str(&format!("| F1 | {:.1}% |\n", metrics.f1 * 100.0));
    markdown.push_str(&format!("| FPR | {:.1}% |\n", metrics.fpr * 100.0));
    markdown.push_str(&format!("| FNR | {:.1}% |\n", metrics.fnr * 100.0));
    markdown.push_str(&format!(
        "| Specificity | {:.1}% |\n",
        metrics.specificity * 100.0
    ));

    markdown.push_str("\n## PR-AUC & ROC-AUC\n\n");
    markdown.push_str(&format!("- **PR-AUC**: {:.3}\n", metrics.auc_pr));
    markdown.push_str(&format!("- **ROC-AUC**: {:.3}\n", metrics.auc_roc));

    if !metrics.top_k_precision.is_empty() {
        markdown.push_str("\n## Top-K Precision\n\n");
        markdown.push_str("| K | Precision |\n");
        markdown.push_str("|---|-----------|\n");
        for top_k in &metrics.top_k_precision {
            markdown.push_str(&format!(
                "| {} | {:.1}% |\n",
                top_k.k,
                top_k.precision * 100.0
            ));
        }
    }

    markdown.push_str("\n## Calibration\n\n");
    markdown.push_str(&format!(
        "- **ECE**: {:.3}\n",
        metrics.calibration.expected_calibration_error
    ));
    markdown.push_str(&format!(
        "- **Max CE**: {:.3}\n",
        metrics.calibration.max_calibration_error
    ));
    markdown.push_str(&format!(
        "- **Brier Score**: {:.3}\n",
        metrics.calibration.brier_score
    ));

    if metrics.calibration.expected_calibration_error < 0.05 {
        markdown.push_str("\n✅ **Model is well-calibrated**\n");
    } else if metrics.calibration.expected_calibration_error < 0.10 {
        markdown.push_str("\n📌 **Model is moderately calibrated**\n");
    } else {
        markdown.push_str("\n🔴 **Model is poorly calibrated**\n");
    }

    let report_path = output_path.with_extension("md");
    std::fs::write(&report_path, markdown)?;

    Ok(())
}
