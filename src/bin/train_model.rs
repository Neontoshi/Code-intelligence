// src/bin/train_model.rs

use clap::Parser;
use code_intelligence::analysis::training_data::TrainingLabel;
use code_intelligence::ml::classifier::DeadCodeClassifier;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    /// Training data file (repository-level split)
    #[arg(short, long, default_value = "data/train.json")]
    train_data: PathBuf,

    /// Validation data file (repository-level split)
    #[arg(short, long, default_value = "data/val.json")]
    val_data: PathBuf,

    /// Test data file (repository-level split)
    #[arg(short, long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Model output path
    #[arg(short, long, default_value = "model.bin")]
    output: PathBuf,

    /// Target precision for threshold tuning
    #[arg(long, default_value = "0.99")]
    target_precision: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔬 Training Model with Repository-Level Split");
    println!("============================================\n");

    // Load training data
    println!("📊 Loading training data from: {:?}", args.train_data);
    let train_data = std::fs::read_to_string(&args.train_data)?;
    let train_examples: Vec<code_intelligence::analysis::training_data::TrainingExample> =
        serde_json::from_str(&train_data)?;

    // Load validation data
    println!("📊 Loading validation data from: {:?}", args.val_data);
    let val_data = std::fs::read_to_string(&args.val_data)?;
    let val_examples: Vec<code_intelligence::analysis::training_data::TrainingExample> =
        serde_json::from_str(&val_data)?;

    // Load test data
    println!("📊 Loading test data from: {:?}", args.test_data);
    let test_data = std::fs::read_to_string(&args.test_data)?;
    let test_examples: Vec<code_intelligence::analysis::training_data::TrainingExample> =
        serde_json::from_str(&test_data)?;

    println!("\n📊 Dataset Statistics:");
    println!("   Train: {} examples", train_examples.len());
    println!("   Validation: {} examples", val_examples.len());
    println!("   Test: {} examples", test_examples.len());

    // Training set stats
    let train_alive = train_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let train_dead = train_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();
    let train_unknown = train_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Unknown)
        .count();

    println!("\n   Train split:");
    println!(
        "      Alive: {}, Dead: {}, Unknown: {}",
        train_alive, train_dead, train_unknown
    );

    // Validation set stats
    let val_alive = val_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let val_dead = val_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();
    let val_unknown = val_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Unknown)
        .count();

    println!("   Validation split:");
    println!(
        "      Alive: {}, Dead: {}, Unknown: {}",
        val_alive, val_dead, val_unknown
    );

    // Test set stats
    let test_alive = test_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let test_dead = test_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();
    let test_unknown = test_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Unknown)
        .count();

    println!("   Test split:");
    println!(
        "      Alive: {}, Dead: {}, Unknown: {}",
        test_alive, test_dead, test_unknown
    );

    // Train model on TRAINING set only
    println!("\n🧠 Training model on training set...");
    let mut classifier = DeadCodeClassifier::new();
    classifier.train(&train_examples)?;

    // Print feature importance
    classifier.print_feature_importance();

    // Evaluate on validation set
    println!("\n📊 Validation Set Performance:");
    let val_accuracy = evaluate_classifier(&classifier, &val_examples);
    println!("   Accuracy: {:.1}%", val_accuracy * 100.0);

    // Evaluate on test set
    println!("\n📊 Test Set Performance:");
    let test_metrics = evaluate_classifier_full(&classifier, &test_examples);
    println!("   Accuracy: {:.1}%", test_metrics.accuracy * 100.0);
    println!("   Precision: {:.1}%", test_metrics.precision * 100.0);
    println!("   Recall: {:.1}%", test_metrics.recall * 100.0);
    println!("   F1: {:.1}%", test_metrics.f1 * 100.0);
    println!("   FPR: {:.1}%", test_metrics.fpr * 100.0);

    // Create versioned model
    use code_intelligence::ml::{ModelPerformance, TrainingMetadata, VersionedModel};

    let metadata = TrainingMetadata {
        training_repositories: vec!["repository_split".to_string()],
        examples_count: train_examples.len(),
        alive_count: train_alive,
        dead_count: train_dead,
        languages: vec![
            "rust".to_string(),
            "python".to_string(),
            "js".to_string(),
            "go".to_string(),
            "java".to_string(),
        ],
        training_date: chrono::Utc::now().to_rfc3339(),
        training_duration_secs: 0.0,
    };

    let performance = ModelPerformance {
        accuracy: test_metrics.accuracy,
        precision: test_metrics.precision,
        recall: test_metrics.recall,
        f1: test_metrics.f1,
        fpr: test_metrics.fpr,
        fnr: 1.0 - test_metrics.recall,
        threshold: args.target_precision,
    };

    if let Some(inner_model) = classifier.model.clone() {
        // Create versioned model with components
        let mut versioned = VersionedModel::new(inner_model, metadata, Some(performance));

        // Add threshold from args
        versioned.set_threshold(args.target_precision);

        // TODO: Add scaler if we start using feature scaling
        // versioned.set_scaler(scaler);

        // TODO: Add calibration after calibrate_model runs
        // versioned.set_calibration(calibration);

        let versioned_path = args.output.with_extension("v2.json");
        versioned.save(&versioned_path.to_string_lossy())?;
        println!("\n✅ Versioned model saved to: {:?}", versioned_path);
        println!("   Threshold: {:.2}", versioned.get_threshold());
        if let Some(perf) = versioned.get_performance() {
            println!("   Test F1: {:.1}%", perf.f1 * 100.0);
        }
    }

    // Also save legacy format for backward compatibility
    classifier.save(&args.output.to_string_lossy())?;
    println!("✅ Legacy model saved to: {:?}", args.output);

    // Show predictions on test set
    println!("\n🔮 Sample Predictions (Test Set):");
    for example in test_examples.iter().take(10) {
        let prediction = classifier.predict(example);
        let prob = classifier.predict_probability(example);
        let emoji = if prediction == TrainingLabel::Alive {
            "✅"
        } else {
            "❌"
        };
        let label = format!("{:?}", prediction);
        let actual = format!("{:?}", example.label);
        println!(
            "   {} {} → {} ({:.1}% confidence) [actual: {}]",
            emoji,
            example.function_name,
            label,
            prob * 100.0,
            actual
        );
    }

    println!(
        "\n📊 Model Test Accuracy: {:.1}%",
        test_metrics.accuracy * 100.0
    );

    Ok(())
}

// ============================================================================
// Evaluation Helpers
// ============================================================================

#[derive(Debug, Clone)]
struct FullMetrics {
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    fpr: f64,
}

fn evaluate_classifier(
    classifier: &DeadCodeClassifier,
    examples: &[code_intelligence::analysis::training_data::TrainingExample],
) -> f64 {
    let labeled: Vec<_> = examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .collect();

    if labeled.is_empty() {
        return 0.0;
    }

    let mut correct = 0;
    for example in &labeled {
        let prediction = classifier.predict(example);
        if prediction == example.label {
            correct += 1;
        }
    }

    correct as f64 / labeled.len() as f64
}

fn evaluate_classifier_full(
    classifier: &DeadCodeClassifier,
    examples: &[code_intelligence::analysis::training_data::TrainingExample],
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
            fpr: 0.0,
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
    let fpr = if fp + tn > 0 {
        fp as f64 / (fp + tn) as f64
    } else {
        0.0
    };

    FullMetrics {
        accuracy,
        precision,
        recall,
        f1,
        fpr,
    }
}
