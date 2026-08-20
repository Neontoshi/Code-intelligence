// src/bin/evaluate_per_language_detailed.rs

//! Detailed per-language evaluation with breakdown by language

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::DeadCodeClassifier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Evaluate model performance per language")]
struct Args {
    /// Model file path
    #[arg(short, long)]
    model: PathBuf,

    /// Test data file
    #[arg(short, long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Output file for results
    #[arg(short, long, default_value = "per_language_results.json")]
    output: PathBuf,

    /// Generate markdown report
    #[arg(long)]
    report: bool,

    /// Show detailed metrics per language
    #[arg(long)]
    detailed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageMetrics {
    pub language: String,
    pub examples: usize,
    pub alive_count: usize,
    pub dead_count: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub fpr: f64,
    pub fnr: f64,
    pub confusion: ConfusionMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    pub tp: usize,
    pub tn: usize,
    pub fp: usize,
    pub fn_: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerLanguageResults {
    pub overall: LanguageMetrics,
    pub by_language: Vec<LanguageMetrics>,
    pub summary: LanguageSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSummary {
    pub best_language: String,
    pub worst_language: String,
    pub avg_f1: f64,
    pub f1_variance: f64,
    pub languages_with_good_performance: Vec<String>,
    pub languages_needing_improvement: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("📊 Per-Language Evaluation");
    println!("=========================");

    // Load model
    println!("📊 Loading model from: {:?}", args.model);
    let classifier = DeadCodeClassifier::load(&args.model.to_string_lossy())?;
    println!("   Model loaded successfully");

    // Load test data
    println!("📊 Loading test data from: {:?}", args.test_data);
    let data = std::fs::read_to_string(&args.test_data)?;
    let test_examples: Vec<TrainingExample> = serde_json::from_str(&data)?;
    println!("   Test examples: {}", test_examples.len());

    // Group by language
    let mut by_language: HashMap<String, Vec<TrainingExample>> = HashMap::new();
    for example in test_examples {
        by_language
            .entry(example.language.clone())
            .or_default()
            .push(example);
    }

    println!("   Languages found: {}", by_language.len());

    // Evaluate per language
    let mut language_metrics = Vec::new();

    for (language, examples) in &by_language {
        let metrics = evaluate_language(&classifier, examples, language);
        language_metrics.push(metrics);
    }

    // Sort by F1 (best first)
    language_metrics.sort_by(|a, b| b.f1.partial_cmp(&a.f1).unwrap_or(std::cmp::Ordering::Equal));

    // Compute overall metrics
    let all_examples: Vec<TrainingExample> = by_language.values().flatten().cloned().collect();
    let overall = evaluate_language(&classifier, &all_examples, "Overall");

    // Generate summary
    let summary = generate_summary(&language_metrics);

    let results = PerLanguageResults {
        overall,
        by_language: language_metrics,
        summary,
    };

    // Save results
    std::fs::write(&args.output, serde_json::to_string_pretty(&results)?)?;

    // Print summary
    print_summary(&results, args.detailed);

    if args.report {
        generate_markdown_report(&results, &args.output)?;
    }

    println!("\n📁 Results saved to: {:?}", args.output);

    Ok(())
}

fn evaluate_language(
    classifier: &DeadCodeClassifier,
    examples: &[TrainingExample],
    language: &str,
) -> LanguageMetrics {
    let labeled: Vec<_> = examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .collect();

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

    let alive_count = labeled
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let dead_count = labeled
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();

    LanguageMetrics {
        language: language.to_string(),
        examples: labeled.len(),
        alive_count,
        dead_count,
        accuracy,
        precision,
        recall,
        f1,
        fpr,
        fnr,
        confusion: ConfusionMatrix { tp, tn, fp, fn_ },
    }
}

fn generate_summary(metrics: &[LanguageMetrics]) -> LanguageSummary {
    if metrics.is_empty() {
        return LanguageSummary {
            best_language: "none".to_string(),
            worst_language: "none".to_string(),
            avg_f1: 0.0,
            f1_variance: 0.0,
            languages_with_good_performance: Vec::new(),
            languages_needing_improvement: Vec::new(),
        };
    }

    let mut sorted = metrics.to_vec();
    sorted.sort_by(|a, b| b.f1.partial_cmp(&a.f1).unwrap_or(std::cmp::Ordering::Equal));

    let best_language = sorted.first().unwrap().language.clone();
    let worst_language = sorted.last().unwrap().language.clone();

    let avg_f1: f64 = metrics.iter().map(|m| m.f1).sum::<f64>() / metrics.len() as f64;
    let f1_variance: f64 =
        metrics.iter().map(|m| (m.f1 - avg_f1).powi(2)).sum::<f64>() / metrics.len() as f64;

    let mut good = Vec::new();
    let mut needs_improvement = Vec::new();

    for m in metrics {
        if m.f1 > 0.80 {
            good.push(m.language.clone());
        } else if m.f1 < 0.60 {
            needs_improvement.push(m.language.clone());
        }
    }

    LanguageSummary {
        best_language,
        worst_language,
        avg_f1,
        f1_variance,
        languages_with_good_performance: good,
        languages_needing_improvement: needs_improvement,
    }
}

fn print_summary(results: &PerLanguageResults, detailed: bool) {
    println!("\n📊 Per-Language Summary:");

    // Overall
    println!("\n   Overall:");
    println!("      Accuracy: {:.1}%", results.overall.accuracy * 100.0);
    println!("      Precision: {:.1}%", results.overall.precision * 100.0);
    println!("      Recall: {:.1}%", results.overall.recall * 100.0);
    println!("      F1: {:.1}%", results.overall.f1 * 100.0);

    // By language
    println!("\n   By Language:");
    println!(
        "   {:<12} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8}",
        "Language", "Examples", "Precision", "Recall", "F1", "FPR"
    );
    println!(
        "   {:-<12}-+-{:-<8}-+-{:-<8}-+-{:-<8}-+-{:-<8}-+-{:-<8}",
        "", "", "", "", "", ""
    );

    for m in &results.by_language {
        let emoji = if m.f1 > 0.85 {
            "✅"
        } else if m.f1 > 0.70 {
            "📌"
        } else {
            "🔴"
        };
        println!(
            "   {:<12} | {:>8} | {:>7.1}% | {:>7.1}% | {:>7.1}% | {:>7.1}%",
            format!("{} {}", emoji, m.language),
            m.examples,
            m.precision * 100.0,
            m.recall * 100.0,
            m.f1 * 100.0,
            m.fpr * 100.0
        );
    }

    // Summary
    println!("\n   Summary:");
    println!(
        "      Best language: {} (F1: {:.1}%)",
        results.summary.best_language,
        results
            .by_language
            .first()
            .map(|m| m.f1 * 100.0)
            .unwrap_or(0.0)
    );
    println!(
        "      Worst language: {} (F1: {:.1}%)",
        results.summary.worst_language,
        results
            .by_language
            .last()
            .map(|m| m.f1 * 100.0)
            .unwrap_or(0.0)
    );
    println!("      Avg F1: {:.1}%", results.summary.avg_f1 * 100.0);
    println!("      F1 variance: {:.3}", results.summary.f1_variance);

    if !results.summary.languages_with_good_performance.is_empty() {
        println!(
            "      ✅ Good performance: {}",
            results.summary.languages_with_good_performance.join(", ")
        );
    }
    if !results.summary.languages_needing_improvement.is_empty() {
        println!(
            "      🔴 Needs improvement: {}",
            results.summary.languages_needing_improvement.join(", ")
        );
    }

    if detailed {
        println!("\n   Detailed Confusion Matrices:");
        for m in &results.by_language {
            println!("\n      {}:", m.language);
            println!(
                "         TP: {}, TN: {}, FP: {}, FN: {}",
                m.confusion.tp, m.confusion.tn, m.confusion.fp, m.confusion.fn_
            );
        }
    }
}

fn generate_markdown_report(
    results: &PerLanguageResults,
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut markdown = String::new();

    markdown.push_str("# 📊 Per-Language Evaluation Report\n\n");

    markdown.push_str("## Overall Performance\n\n");
    markdown.push_str(&format!(
        "- **Accuracy**: {:.1}%\n",
        results.overall.accuracy * 100.0
    ));
    markdown.push_str(&format!(
        "- **Precision**: {:.1}%\n",
        results.overall.precision * 100.0
    ));
    markdown.push_str(&format!(
        "- **Recall**: {:.1}%\n",
        results.overall.recall * 100.0
    ));
    markdown.push_str(&format!("- **F1**: {:.1}%\n", results.overall.f1 * 100.0));

    markdown.push_str("\n## Per-Language Performance\n\n");
    markdown.push_str("| Language | Examples | Precision | Recall | F1 | FPR |\n");
    markdown.push_str("|----------|----------|-----------|--------|----|-----|\n");

    for m in &results.by_language {
        markdown.push_str(&format!(
            "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
            m.language,
            m.examples,
            m.precision * 100.0,
            m.recall * 100.0,
            m.f1 * 100.0,
            m.fpr * 100.0
        ));
    }

    markdown.push_str("\n## Summary\n\n");
    markdown.push_str(&format!(
        "- **Best language**: {} (F1: {:.1}%)\n",
        results.summary.best_language,
        results
            .by_language
            .first()
            .map(|m| m.f1 * 100.0)
            .unwrap_or(0.0)
    ));
    markdown.push_str(&format!(
        "- **Worst language**: {} (F1: {:.1}%)\n",
        results.summary.worst_language,
        results
            .by_language
            .last()
            .map(|m| m.f1 * 100.0)
            .unwrap_or(0.0)
    ));
    markdown.push_str(&format!(
        "- **Average F1**: {:.1}%\n",
        results.summary.avg_f1 * 100.0
    ));

    if !results.summary.languages_with_good_performance.is_empty() {
        markdown.push_str(&format!(
            "- ✅ **Good performance**: {}\n",
            results.summary.languages_with_good_performance.join(", ")
        ));
    }
    if !results.summary.languages_needing_improvement.is_empty() {
        markdown.push_str(&format!(
            "- 🔴 **Needs improvement**: {}\n",
            results.summary.languages_needing_improvement.join(", ")
        ));
    }

    let report_path = output_path.with_extension("md");
    std::fs::write(&report_path, markdown)?;

    Ok(())
}
