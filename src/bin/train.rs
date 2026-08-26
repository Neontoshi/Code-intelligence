// src/bin/train.rs

use clap::{Parser, Subcommand};
use code_intelligence::error::{err, Result};
use code_intelligence::DeadCodeClassifier;
use code_intelligence::TrainingExample;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about = "ML training toolkit")]
struct Args {
    #[command(subcommand)]
    command: TrainCommand,
}

#[derive(Subcommand, Debug)]
enum TrainCommand {
    /// Train a dead code detection model
    Model {
        /// Training data path
        #[arg(long, default_value = "data/train.json")]
        data: PathBuf,
        /// Validation data path
        #[arg(long, default_value = "data/val.json")]
        val_data: Option<PathBuf>,
        /// Output model path
        #[arg(long, default_value = "model.bin")]
        output: PathBuf,
        /// Target precision (0.0-1.0)
        #[arg(long, default_value = "0.95")]
        precision: f64,
    },
    /// Train a duplicate detection model
    Duplicate {
        /// Input data path (JSON with training examples)
        #[arg(short, long)]
        input: PathBuf,
        /// Output model path
        #[arg(short, long, default_value = "duplicate_model.bin")]
        output: PathBuf,
    },
    /// Calibrate a trained model
    Calibrate {
        /// Model file path
        #[arg(long)]
        model: PathBuf,
        /// Validation data
        #[arg(long)]
        data: PathBuf,
        /// Output model path
        #[arg(long, default_value = "model_calibrated.bin")]
        output: PathBuf,
        /// Calibration method: temperature, histogram, none
        #[arg(long, default_value = "temperature")]
        method: String,
    },
    /// Tune confidence threshold
    Tune {
        /// Model file path
        #[arg(long)]
        model: PathBuf,
        /// Validation data
        #[arg(long)]
        data: PathBuf,
        /// Target precision (0.0-1.0)
        #[arg(long, default_value = "0.99")]
        precision: f64,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        TrainCommand::Model {
            data,
            val_data,
            output,
            precision,
        } => {
            run_train_model(&data, val_data.as_deref(), &output, precision)?;
        }
        TrainCommand::Duplicate { input, output } => {
            run_train_duplicate(&input, &output)?;
        }
        TrainCommand::Calibrate {
            model,
            data,
            output,
            method,
        } => {
            run_calibrate(&model, &data, &output, &method)?;
        }
        TrainCommand::Tune {
            model,
            data,
            precision,
        } => {
            run_tune(&model, &data, precision)?;
        }
    }

    Ok(())
}

fn run_train_model(
    data: &Path,
    val_data: Option<&Path>,
    output: &Path,
    precision: f64,
) -> Result<()> {
    use code_intelligence::analysis::training_data::TrainingExample;
    use code_intelligence::ml::classifier::DeadCodeClassifier;

    println!("🧠 Training model...");
    println!("   Training data: {:?}", data);

    let data_str = std::fs::read_to_string(data)?;
    let train_examples: Vec<TrainingExample> = serde_json::from_str(&data_str)?;
    println!("   Training examples: {}", train_examples.len());

    let mut classifier = DeadCodeClassifier::new();
    classifier
        .train(&train_examples)
        .map_err(|e| err::training(e))?;

    if let Some(vd) = val_data {
        println!("   Validation data: {:?}", vd);
        let val_str = std::fs::read_to_string(vd)?;
        let val_examples: Vec<TrainingExample> = serde_json::from_str(&val_str)?;
        println!("   Validation examples: {}", val_examples.len());

        // Split validation into calibration and tuning sets
        let split_idx = val_examples.len() / 2;
        let (calibration_examples, tuning_examples) = val_examples.split_at(split_idx);
        println!(
            "   Calibration examples: {}, Tuning examples: {}",
            calibration_examples.len(),
            tuning_examples.len()
        );

        // CALIBRATE first using calibration split
        classifier
            .calibrate(calibration_examples)
            .map_err(|e| err::training(e))?;
        println!("   Model calibrated on calibration split");

        // TUNE threshold using calibrated predictions on tuning split
        let optimal_threshold =
            tune_threshold_for_precision_calibrated(&classifier, tuning_examples, precision);
        println!(
            "   Optimal threshold for precision {:.2}: {:.2}",
            precision, optimal_threshold
        );

        // PERSIST the frozen threshold
        classifier.set_threshold(optimal_threshold);
        println!("   Threshold frozen at: {:.2}", optimal_threshold);
    } else {
        println!("   ⚠️  No validation data provided - using default threshold 0.92");
        classifier.set_threshold(0.92);
    }

    classifier.print_feature_importance();
    classifier
        .save(output)
        .map_err(|e| err::model(e.to_string()))?;

    println!("\n✅ Model saved to: {:?}", output);

    Ok(())
}

fn tune_threshold_for_precision_calibrated(
    classifier: &DeadCodeClassifier,
    val_examples: &[TrainingExample],
    target_precision: f64,
) -> f64 {
    use code_intelligence::analysis::training_data::TrainingLabel;

    let mut best_threshold = 0.92;
    let mut best_f1 = 0.0;

    for threshold in (50..=99).map(|t| t as f64 / 100.0) {
        let mut tp = 0;
        let mut fp = 0;
        let mut fn_ = 0;

        for example in val_examples {
            if example.label == TrainingLabel::Unknown {
                continue;
            }

            let features = example.features.to_feature_vector();
            let dead_prob = 1.0 - classifier.predict_calibrated(&features);
            let pred = if dead_prob >= threshold {
                TrainingLabel::Dead
            } else {
                TrainingLabel::Alive
            };

            match (pred, &example.label) {
                (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
                (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
                (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
                _ => {}
            }
        }

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

        if precision >= target_precision && f1 > best_f1 {
            best_f1 = f1;
            best_threshold = threshold;
        }
    }

    best_threshold
}

fn run_train_duplicate(input: &Path, output: &Path) -> Result<()> {
    use code_intelligence::analysis::training_data::TrainingExample;
    use code_intelligence::ml::duplicate_classifier::{
        DuplicateClassifier, DuplicateExample, DuplicateLabel,
    };

    println!("🧠 Training duplicate detection model...");

    let data_str = std::fs::read_to_string(input)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data_str)?;

    let mut duplicate_examples = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for i in 0..examples.len().min(100) {
        for j in (i + 1)..examples.len().min(100) {
            let a = &examples[i];
            let b = &examples[j];

            let key = (a.full_path.clone(), b.full_path.clone());
            if processed.contains(&key) {
                continue;
            }
            processed.insert(key);

            let similarity = a
                .features
                .to_feature_vector()
                .iter()
                .zip(b.features.to_feature_vector().iter())
                .map(|(x, y)| if x == y { 1.0 } else { 0.0 })
                .sum::<f64>()
                / a.features.to_feature_vector().len() as f64;

            let label = if similarity > 0.85 {
                DuplicateLabel::Duplicate
            } else if similarity < 0.3 {
                DuplicateLabel::NotDuplicate
            } else {
                continue;
            };

            duplicate_examples.push(DuplicateExample {
                func_a: a.features.clone(),
                func_b: b.features.clone(),
                label,
                confidence: similarity,
            });
        }
    }

    if duplicate_examples.is_empty() {
        return Err(err::training("No duplicate examples generated"));
    }

    let mut classifier = DuplicateClassifier::default();
    let accuracy = classifier.train(&duplicate_examples);
    println!("   Training accuracy: {:.1}%", accuracy * 100.0);

    classifier
        .save(output)
        .map_err(|e| err::model(e.to_string()))?;
    println!("✅ Model saved to: {:?}", output);

    Ok(())
}

fn run_calibrate(model: &Path, data: &Path, output: &Path, method: &str) -> Result<()> {
    use code_intelligence::analysis::training_data::TrainingExample;
    use code_intelligence::ml::calibration::{CalibratedModel, CalibrationMethod};
    use code_intelligence::ml::classifier::DeadCodeClassifier;

    println!("🔬 Calibrating model...");

    let mut classifier = DeadCodeClassifier::load(&*model.to_string_lossy())
        .map_err(|e| err::model(e.to_string()))?;
    let data_str = std::fs::read_to_string(data)?;
    let val_examples: Vec<TrainingExample> = serde_json::from_str(&data_str)?;

    let cal_method = match method {
        "temperature" => CalibrationMethod::TemperatureScaling,
        "histogram" => CalibrationMethod::HistogramBinning,
        _ => CalibrationMethod::None,
    };
    if let Some(model) = classifier.get_model_mut() {
        let calibrated = CalibratedModel::calibrate(model, &val_examples, cal_method);

        // Clone before moving
        let cal_classifier = calibrated.classifier.clone();
        let cal_params = calibrated.calibration.clone();

        classifier.model = Some(cal_classifier);
        classifier.calibration = Some(cal_params);

        let stats = calibrated.calibration_stats(&val_examples);
        stats.print();
        classifier
            .save(output)
            .map_err(|e| err::model(e.to_string()))?;
        println!("\n✅ Calibrated model saved to: {:?}", output);
    }

    Ok(())
}

fn run_tune(model: &Path, data: &Path, precision: f64) -> Result<()> {
    use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
    use code_intelligence::ml::classifier::DeadCodeClassifier;

    println!("🎯 Tuning threshold...");

    let classifier = DeadCodeClassifier::load(&*model.to_string_lossy())
        .map_err(|e| err::model(e.to_string()))?;
    let data_str = std::fs::read_to_string(data)?;
    let val_examples: Vec<TrainingExample> = serde_json::from_str(&data_str)?;

    let mut best_threshold = 0.92;
    let mut best_f1 = 0.0;

    for threshold in (50..=95).step_by(5).map(|t| t as f64 / 100.0) {
        let mut tp = 0;
        let mut fn_ = 0;
        let mut fp = 0;

        for example in &val_examples {
            let dead_prob = classifier.predict_dead_probability(example);
            let pred = if dead_prob >= threshold {
                TrainingLabel::Dead
            } else {
                TrainingLabel::Alive
            };
            let actual = &example.label;

            match (pred, actual) {
                (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
                (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
                (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
                _ => {}
            }
        }

        let p = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let r = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };
        let f1 = if p + r > 0.0 {
            2.0 * p * r / (p + r)
        } else {
            0.0
        };

        if f1 > best_f1 && p >= precision {
            best_f1 = f1;
            best_threshold = threshold;
        }
    }

    println!("\n📊 Optimal threshold: {:.2}", best_threshold);
    println!("   Best F1: {:.1}%", best_f1 * 100.0);

    Ok(())
}
