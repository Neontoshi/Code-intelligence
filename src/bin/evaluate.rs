// src/bin/evaluate.rs

use clap::{Parser, Subcommand};
use code_intelligence::error::{err, Result};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about = "Model evaluation toolkit")]
struct Args {
    #[command(subcommand)]
    command: EvalCommand,
}

#[derive(Subcommand, Debug)]
enum EvalCommand {
    /// Basic metrics evaluation
    Metrics {
        /// Model file path
        #[arg(short, long)]
        model: PathBuf,
        /// Test data file
        #[arg(short, long, default_value = "data/test.json")]
        test_data: PathBuf,
        /// Output file
        #[arg(short, long, default_value = "evaluation_results.json")]
        output: PathBuf,
    },
    /// Per-language evaluation
    PerLanguage {
        /// Model file path
        #[arg(short, long)]
        model: PathBuf,
        /// Test data file
        #[arg(short, long, default_value = "data/test.json")]
        test_data: PathBuf,
        /// Output file
        #[arg(short, long, default_value = "per_language_results.json")]
        output: PathBuf,
        /// Show detailed metrics
        #[arg(long)]
        detailed: bool,
    },
    /// Detailed evaluation with report
    Detailed {
        /// Model file path
        #[arg(short, long)]
        model: PathBuf,
        /// Test data file
        #[arg(short, long, default_value = "data/test.json")]
        test_data: PathBuf,
        /// Validation data file
        #[arg(short, long)]
        val_data: Option<PathBuf>,
        /// Output directory
        #[arg(short, long, default_value = "evaluation_results")]
        output_dir: PathBuf,
        /// Generate markdown report
        #[arg(long)]
        report: bool,
        /// Top-K values for precision@K
        #[arg(long, default_value = "10,25,50,100")]
        top_k: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        EvalCommand::Metrics {
            model,
            test_data,
            output,
        } => {
            run_metrics(&model, &test_data, &output)?;
        }
        EvalCommand::PerLanguage {
            model,
            test_data,
            output,
            detailed,
        } => {
            run_per_language(&model, &test_data, &output, detailed)?;
        }
        EvalCommand::Detailed {
            model,
            test_data,
            val_data,
            output_dir,
            report,
            top_k,
        } => {
            run_detailed(
                &model,
                &test_data,
                val_data.as_deref(),
                &output_dir,
                report,
                &top_k,
            )?;
        }
    }

    Ok(())
}

fn run_metrics(model: &Path, test_data: &Path, output: &Path) -> Result<()> {
    use code_intelligence::analysis::training_data::TrainingExample;
    use code_intelligence::ml::classifier::DeadCodeClassifier;

    println!("📊 Running metrics evaluation...");

    let classifier = DeadCodeClassifier::load(&*model.to_string_lossy())
        .map_err(|e| err::model(e.to_string()))?;
    let data = std::fs::read_to_string(test_data)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    let metrics = compute_metrics(&classifier, &examples);
    std::fs::write(output, serde_json::to_string_pretty(&metrics)?)?;

    println!("✅ Results saved to: {:?}", output);
    println!(
        "   Accuracy: {:.1}%, Precision: {:.1}%, Recall: {:.1}%, F1: {:.1}%",
        metrics.accuracy * 100.0,
        metrics.precision * 100.0,
        metrics.recall * 100.0,
        metrics.f1 * 100.0
    );

    Ok(())
}

fn run_per_language(model: &Path, test_data: &Path, output: &Path, detailed: bool) -> Result<()> {
    use code_intelligence::analysis::training_data::TrainingExample;
    use code_intelligence::ml::classifier::DeadCodeClassifier;

    println!("📊 Running per-language evaluation...");

    let classifier = DeadCodeClassifier::load(&*model.to_string_lossy())
        .map_err(|e| err::model(e.to_string()))?;
    let data = std::fs::read_to_string(test_data)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    let mut by_language: std::collections::HashMap<String, Vec<TrainingExample>> =
        std::collections::HashMap::new();
    for example in examples {
        by_language
            .entry(example.language.clone())
            .or_default()
            .push(example);
    }

    let mut results = Vec::new();
    for (language, examples) in &by_language {
        let metrics = compute_metrics(&classifier, examples);
        results.push(serde_json::json!({
            "language": language,
            "examples": examples.len(),
            "accuracy": metrics.accuracy,
            "precision": metrics.precision,
            "recall": metrics.recall,
            "f1": metrics.f1,
            "fpr": metrics.fpr,
        }));
    }

    results.sort_by(|a, b| {
        b["f1"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["f1"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    std::fs::write(output, serde_json::to_string_pretty(&results)?)?;
    println!("✅ Results saved to: {:?}", output);

    if detailed {
        println!("\n📊 Per-Language Results:");
        for result in &results {
            println!(
                "   {:<12} F1={:>5.1}%, Prec={:>5.1}%, Rec={:>5.1}%, Accuracy={:>5.1}%, Examples={}",
                result["language"].as_str().unwrap_or("unknown"),
                result["f1"].as_f64().unwrap_or(0.0) * 100.0,
                result["precision"].as_f64().unwrap_or(0.0) * 100.0,
                result["recall"].as_f64().unwrap_or(0.0) * 100.0,
                result["accuracy"].as_f64().unwrap_or(0.0) * 100.0,
                result["examples"].as_u64().unwrap_or(0)
            );
        }
    }

    Ok(())
}

fn run_detailed(
    model: &Path,
    test_data: &Path,
    val_data: Option<&Path>,
    output_dir: &Path,
    report: bool,
    top_k: &str,
) -> Result<()> {
    use code_intelligence::analysis::training_data::TrainingExample;
    use code_intelligence::ml::classifier::DeadCodeClassifier;

    println!("📊 Running detailed evaluation...");

    std::fs::create_dir_all(output_dir)?;

    let classifier = DeadCodeClassifier::load(&*model.to_string_lossy())
        .map_err(|e| err::model(e.to_string()))?;
    let test_data_str = std::fs::read_to_string(test_data)?;
    let test_examples: Vec<TrainingExample> = serde_json::from_str(&test_data_str)?;

    let top_k_values: Vec<usize> = top_k.split(',').filter_map(|s| s.parse().ok()).collect();
    let metrics = compute_detailed_metrics(&classifier, &test_examples, &top_k_values);

    let json_path = output_dir.join("detailed_results.json");
    std::fs::write(&json_path, serde_json::to_string_pretty(&metrics)?)?;

    if report {
        generate_report(&metrics, output_dir)?;
    }

    if let Some(vd) = val_data {
        let val_data_str = std::fs::read_to_string(vd)?;
        let val_examples: Vec<TrainingExample> = serde_json::from_str(&val_data_str)?;
        let val_metrics = compute_detailed_metrics(&classifier, &val_examples, &top_k_values);
        let val_path = output_dir.join("validation_metrics.json");
        std::fs::write(&val_path, serde_json::to_string_pretty(&val_metrics)?)?;
    }

    println!("✅ Detailed evaluation complete!");
    println!("   Results: {:?}", output_dir);
    println!(
        "   Accuracy: {:.1}%, Precision: {:.1}%, Recall: {:.1}%, F1: {:.1}%",
        metrics.accuracy * 100.0,
        metrics.precision * 100.0,
        metrics.recall * 100.0,
        metrics.f1 * 100.0
    );

    Ok(())
}

// Common evaluation functions

#[derive(Debug, Clone, serde::Serialize)]
struct EvaluationMetrics {
    total: usize,
    correct: usize,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    fpr: f64,
    fnr: f64,
    specificity: f64,
    confusion_matrix: ConfusionMatrix,
    top_k_precision: Vec<TopKPrecision>,
    auc_pr: f64,
    auc_roc: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ConfusionMatrix {
    tp: usize,
    tn: usize,
    fp: usize,
    #[serde(rename = "fn")]
    fn_: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TopKPrecision {
    k: usize,
    precision: f64,
}

fn compute_metrics(
    classifier: &code_intelligence::ml::classifier::DeadCodeClassifier,
    examples: &[code_intelligence::analysis::training_data::TrainingExample],
) -> EvaluationMetrics {
    use code_intelligence::analysis::training_data::TrainingLabel;

    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in examples {
        if example.label == TrainingLabel::Unknown {
            continue;
        }

        let dead_prob = classifier.predict_dead_probability(example);
        let pred = if dead_prob >= 0.5 {
            TrainingLabel::Dead
        } else {
            TrainingLabel::Alive
        };
        let actual = &example.label;

        match (pred, actual) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }
    }

    let total = tp + tn + fp + fn_;
    let correct = tp + tn;

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

    EvaluationMetrics {
        total,
        correct,
        accuracy: if total > 0 {
            correct as f64 / total as f64
        } else {
            0.0
        },
        precision,
        recall,
        f1,
        fpr,
        fnr,
        specificity: 1.0 - fpr,
        confusion_matrix: ConfusionMatrix {
            tp,
            tn,
            fp,
            fn_: fn_,
        },
        top_k_precision: Vec::new(),
        auc_pr: 0.0,
        auc_roc: 0.0,
    }
}

fn compute_detailed_metrics(
    classifier: &code_intelligence::ml::classifier::DeadCodeClassifier,
    examples: &[code_intelligence::analysis::training_data::TrainingExample],
    top_k_values: &[usize],
) -> EvaluationMetrics {
    let mut metrics = compute_metrics(classifier, examples);

    let mut preds: Vec<(f64, f64)> = Vec::new();
    for example in examples {
        if example.label != code_intelligence::analysis::training_data::TrainingLabel::Unknown {
            let dead_prob = classifier.predict_dead_probability(example);
            let label = match example.label {
                code_intelligence::analysis::training_data::TrainingLabel::Dead => 1.0,
                code_intelligence::analysis::training_data::TrainingLabel::Alive => 0.0,
                _ => 0.5,
            };
            preds.push((dead_prob, label));
        }
    }
    preds.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    for &k in top_k_values {
        if k > 0 && k <= preds.len() {
            let positive = preds.iter().take(k).filter(|(_, l)| *l == 1.0).count();
            metrics.top_k_precision.push(TopKPrecision {
                k,
                precision: positive as f64 / k as f64,
            });
        }
    }

    let mut tp_count = 0;
    let mut fp_count = 0;
    let total_pos = preds.iter().filter(|(_, l)| *l == 1.0).count();
    let mut precisions = Vec::new();
    let mut recalls = Vec::new();

    if total_pos > 0 {
        for (_, label) in &preds {
            if *label == 1.0 {
                tp_count += 1
            } else {
                fp_count += 1
            }
            let prec = if tp_count + fp_count > 0 {
                tp_count as f64 / (tp_count + fp_count) as f64
            } else {
                0.0
            };
            precisions.push(prec);
            recalls.push(tp_count as f64 / total_pos as f64);
        }
    }

    let mut auc = 0.0;
    for i in 1..precisions.len() {
        let rec_diff = recalls[i] - recalls[i - 1];
        let prec_avg = (precisions[i] + precisions[i - 1]) / 2.0;
        auc += rec_diff * prec_avg;
    }
    metrics.auc_pr = auc;

    let total_neg = preds.iter().filter(|(_, l)| *l == 0.0).count();
    if total_pos > 0 && total_neg > 0 {
        let mut tp_count2 = 0;
        let mut fp_count2 = 0;
        let mut tprs = Vec::new();
        let mut fprs = Vec::new();

        for (_, label) in &preds {
            if *label == 1.0 {
                tp_count2 += 1
            } else {
                fp_count2 += 1
            }
            tprs.push(tp_count2 as f64 / total_pos as f64);
            fprs.push(fp_count2 as f64 / total_neg as f64);
        }

        let mut roc_auc = 0.0;
        for i in 1..tprs.len() {
            let fpr_diff = fprs[i] - fprs[i - 1];
            let tpr_avg = (tprs[i] + tprs[i - 1]) / 2.0;
            roc_auc += fpr_diff * tpr_avg;
        }
        metrics.auc_roc = roc_auc;
    }

    metrics
}

fn generate_report(metrics: &EvaluationMetrics, output_dir: &Path) -> Result<()> {
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
    markdown.push_str(&format!("- **FPR**: {:.1}%\n", metrics.fpr * 100.0));

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

    markdown.push_str("\n## AUC Metrics\n\n");
    markdown.push_str(&format!("- **PR-AUC**: {:.3}\n", metrics.auc_pr));
    markdown.push_str(&format!("- **ROC-AUC**: {:.3}\n", metrics.auc_roc));

    let report_path = output_dir.join("evaluation_report.md");
    std::fs::write(&report_path, markdown)?;

    Ok(())
}
