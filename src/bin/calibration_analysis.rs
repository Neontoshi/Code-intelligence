// src/bin/calibrate_with_metrics.rs

//! Model calibration with comprehensive metrics
//!
//! This tool calibrates the model and provides detailed calibration
//! metrics including reliability diagrams.

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::calibration::{CalibratedModel, CalibrationMethod};
use code_intelligence::ml::classifier::DeadCodeClassifier;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Calibrate model with metrics")]
struct Args {
    /// Model file path
    #[arg(short, long)]
    model: PathBuf,

    /// Validation data file
    #[arg(short, long, default_value = "data/val.json")]
    val_data: PathBuf,

    /// Test data file
    #[arg(short, long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Output model path
    #[arg(short, long, default_value = "model_calibrated.bin")]
    output: PathBuf,

    /// Calibration method: temperature, histogram, isotonic, none
    #[arg(long, default_value = "temperature")]
    method: String,

    /// Generate calibration report
    #[arg(long)]
    report: bool,

    /// Output directory for report
    #[arg(long, default_value = "calibration_results")]
    output_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub method: String,
    pub temperature: f64,
    pub num_samples: usize,
    pub before: CalibrationMetrics,
    pub after: CalibrationMetrics,
    pub bins: Vec<CalibrationBinDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationMetrics {
    pub expected_calibration_error: f64,
    pub max_calibration_error: f64,
    pub brier_score: f64,
    pub log_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBinDetail {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
    pub accuracy: f64,
    pub avg_confidence: f64,
    pub calibration_error: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔬 Model Calibration with Metrics");
    println!("=================================\n");

    // Load model
    println!("📊 Loading model from: {:?}", args.model);
    let classifier = DeadCodeClassifier::load(&args.model.to_string_lossy())?;
    let model = classifier.model.ok_or("No model found")?;
    println!("   Model loaded successfully");

    // Load validation data
    println!("📊 Loading validation data from: {:?}", args.val_data);
    let val_data = std::fs::read_to_string(&args.val_data)?;
    let val_examples: Vec<TrainingExample> = serde_json::from_str(&val_data)?;
    println!("   Validation examples: {}", val_examples.len());

    // Load test data
    println!("📊 Loading test data from: {:?}", args.test_data);
    let test_data = std::fs::read_to_string(&args.test_data)?;
    let test_examples: Vec<TrainingExample> = serde_json::from_str(&test_data)?;
    println!("   Test examples: {}", test_examples.len());

    // Parse method
    let method = match args.method.as_str() {
        "temperature" => CalibrationMethod::TemperatureScaling,
        "histogram" => CalibrationMethod::HistogramBinning,
        "none" => CalibrationMethod::None,
        _ => {
            eprintln!("Unknown method: {}", args.method);
            eprintln!("Available: temperature, histogram, none");
            std::process::exit(1);
        }
    };

    // Create output directory
    if args.report {
        std::fs::create_dir_all(&args.output_dir)?;
    }

    // Before calibration metrics
    println!("\n📊 Before Calibration:");
    let before_metrics = compute_calibration_metrics(&model, &test_examples);
    print_metrics(&before_metrics);

    // Calibrate
    println!("\n🧪 Calibrating with {:?}...", method);
    let calibrated = CalibratedModel::calibrate(&model, &val_examples, method);

    // After calibration metrics
    println!("\n📊 After Calibration:");
    let after_metrics = compute_calibration_metrics(&calibrated.classifier, &test_examples);
    print_metrics(&after_metrics);

    // Generate report
    if args.report {
        let report = CalibrationReport {
            method: args.method.clone(),
            temperature: calibrated.calibration.temperature,
            num_samples: calibrated.calibration.num_samples,
            before: before_metrics,
            after: after_metrics,
            bins: calibrated
                .calibration
                .bins
                .iter()
                .map(|bin| CalibrationBinDetail {
                    lower: bin.lower,
                    upper: bin.upper,
                    count: bin.count,
                    accuracy: bin.empirical_accuracy,
                    avg_confidence: (bin.lower + bin.upper) / 2.0,
                    calibration_error: (bin.empirical_accuracy - (bin.lower + bin.upper) / 2.0)
                        .abs(),
                })
                .collect(),
        };

        let report_path = args.output_dir.join("calibration_report.json");
        std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;

        // Generate markdown
        generate_markdown_report(&report, &args.output_dir)?;

        println!("\n📁 Calibration report saved to: {:?}", args.output_dir);
    }

    // Save calibrated model
    let mut new_classifier = DeadCodeClassifier::new();
    new_classifier.model = Some(calibrated.classifier);
    new_classifier.calibration = Some(calibrated.calibration);
    new_classifier.save(&args.output.to_string_lossy())?;

    println!("\n✅ Calibrated model saved to: {:?}", args.output);

    Ok(())
}

fn compute_calibration_metrics(
    classifier: &code_intelligence::ml::classifier::LinearClassifier,
    examples: &[TrainingExample],
) -> CalibrationMetrics {
    let labeled: Vec<_> = examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .collect();

    let mut predictions = Vec::new();
    let mut confidences = Vec::new();

    for example in &labeled {
        let features = example.features.to_feature_vector();
        let pred = classifier.predict(&features);
        let target = match example.label {
            TrainingLabel::Alive => 1.0,
            TrainingLabel::Dead => 0.0,
            _ => 0.5,
        };
        predictions.push(pred);
        confidences.push(target);
    }

    // ECE
    let num_bins = 10;
    let bin_width = 1.0 / num_bins as f64;
    let mut bin_accuracies = vec![0.0; num_bins];
    let mut bin_confidences = vec![0.0; num_bins];
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
        }
    }

    // Brier score
    let brier_score: f64 = predictions
        .iter()
        .zip(confidences.iter())
        .map(|(&p, &c)| (p - c).powi(2))
        .sum::<f64>()
        / predictions.len() as f64;

    // Log loss
    let log_loss: f64 = predictions
        .iter()
        .zip(confidences.iter())
        .map(|(&p, &c)| {
            let p_clamped = p.clamp(1e-15, 1.0 - 1e-15);
            let c_clamped = c.clamp(1e-15, 1.0 - 1e-15);
            -c_clamped * p_clamped.ln() - (1.0 - c_clamped) * (1.0 - p_clamped).ln()
        })
        .sum::<f64>()
        / predictions.len() as f64;

    CalibrationMetrics {
        expected_calibration_error: ece,
        max_calibration_error: max_ce,
        brier_score,
        log_loss,
    }
}

fn print_metrics(metrics: &CalibrationMetrics) {
    println!("   ECE: {:.3}", metrics.expected_calibration_error);
    println!("   Max CE: {:.3}", metrics.max_calibration_error);
    println!("   Brier Score: {:.3}", metrics.brier_score);
    println!("   Log Loss: {:.3}", metrics.log_loss);

    if metrics.expected_calibration_error < 0.05 {
        println!("   ✅ Well-calibrated");
    } else if metrics.expected_calibration_error < 0.10 {
        println!("   📌 Moderately calibrated");
    } else {
        println!("   🔴 Poorly calibrated");
    }
}

fn generate_markdown_report(
    report: &CalibrationReport,
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut markdown = String::new();

    markdown.push_str("# 🔬 Calibration Report\n\n");
    markdown.push_str(&format!(
        "*Generated on {}*\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    markdown.push_str("## Summary\n\n");
    markdown.push_str(&format!("- **Method**: {}\n", report.method));
    markdown.push_str(&format!("- **Temperature**: {:.3}\n", report.temperature));
    markdown.push_str(&format!("- **Samples**: {}\n\n", report.num_samples));

    markdown.push_str("## Metrics Comparison\n\n");
    markdown.push_str("| Metric | Before | After | Improvement |\n");
    markdown.push_str("|--------|--------|-------|-------------|\n");

    let ece_improvement =
        report.before.expected_calibration_error - report.after.expected_calibration_error;
    let brier_improvement = report.before.brier_score - report.after.brier_score;
    let log_loss_improvement = report.before.log_loss - report.after.log_loss;

    markdown.push_str(&format!(
        "| ECE | {:.3} | {:.3} | {:.3} ({:.1}%) |\n",
        report.before.expected_calibration_error,
        report.after.expected_calibration_error,
        ece_improvement,
        if report.before.expected_calibration_error > 0.0 {
            ece_improvement / report.before.expected_calibration_error * 100.0
        } else {
            0.0
        }
    ));
    markdown.push_str(&format!(
        "| Max CE | {:.3} | {:.3} | {:.3} |\n",
        report.before.max_calibration_error,
        report.after.max_calibration_error,
        report.before.max_calibration_error - report.after.max_calibration_error
    ));
    markdown.push_str(&format!(
        "| Brier | {:.3} | {:.3} | {:.3} ({:.1}%) |\n",
        report.before.brier_score,
        report.after.brier_score,
        brier_improvement,
        if report.before.brier_score > 0.0 {
            brier_improvement / report.before.brier_score * 100.0
        } else {
            0.0
        }
    ));
    markdown.push_str(&format!(
        "| Log Loss | {:.3} | {:.3} | {:.3} ({:.1}%) |\n",
        report.before.log_loss,
        report.after.log_loss,
        log_loss_improvement,
        if report.before.log_loss > 0.0 {
            log_loss_improvement / report.before.log_loss * 100.0
        } else {
            0.0
        }
    ));

    markdown.push_str("\n## Calibration Bins\n\n");
    markdown.push_str("| Bin | Count | Avg Confidence | Accuracy | Error |\n");
    markdown.push_str("|-----|-------|----------------|----------|-------|\n");

    for bin in &report.bins {
        markdown.push_str(&format!(
            "| {:.2}-{:.2} | {} | {:.3} | {:.3} | {:.3} |\n",
            bin.lower,
            bin.upper,
            bin.count,
            bin.avg_confidence,
            bin.accuracy,
            bin.calibration_error
        ));
    }

    let report_path = output_dir.join("calibration_report.md");
    std::fs::write(&report_path, markdown)?;

    Ok(())
}
