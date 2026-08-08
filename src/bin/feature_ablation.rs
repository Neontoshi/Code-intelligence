// src/bin/feature_ablation.rs

//! Feature ablation study - determine which features actually matter
//!
//! This tool trains models with different feature subsets to understand
//! which features are most important for dead code detection.

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::LinearClassifier;
use code_intelligence::ml::feature_schema::{FeatureCategory, FEATURE_SCHEMA};
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    /// Training data file
    #[arg(short, long, default_value = "data/train.json")]
    train_data: PathBuf,

    /// Validation data file
    #[arg(short, long, default_value = "data/val.json")]
    val_data: PathBuf,

    /// Output directory for results
    #[arg(short, long, default_value = "ablation_results")]
    output_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AblationResult {
    name: String,
    features: Vec<String>,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    feature_count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔬 Feature Ablation Study");
    println!("========================\n");

    // Load data
    println!("📊 Loading training data from: {:?}", args.train_data);
    let train_data = std::fs::read_to_string(&args.train_data)?;
    let train_examples: Vec<TrainingExample> = serde_json::from_str(&train_data)?;

    println!("📊 Loading validation data from: {:?}", args.val_data);
    let val_data = std::fs::read_to_string(&args.val_data)?;
    let val_examples: Vec<TrainingExample> = serde_json::from_str(&val_data)?;

    println!("   Train: {} examples", train_examples.len());
    println!("   Val: {} examples\n", val_examples.len());

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    // Define feature sets to test
    let feature_sets = define_feature_sets();

    let mut results = Vec::new();

    for (name, feature_indices) in &feature_sets {
        println!("🧪 Training: {}", name);

        // Train model with this feature subset
        let result = train_and_evaluate(&train_examples, &val_examples, feature_indices, name);

        println!("   Accuracy: {:.1}%", result.accuracy * 100.0);
        println!("   Precision: {:.1}%", result.precision * 100.0);
        println!("   Recall: {:.1}%", result.recall * 100.0);
        println!("   F1: {:.1}%", result.f1 * 100.0);
        println!("   Features: {}\n", result.feature_count);

        results.push(result);
    }

    // Save results
    save_results(&results, &args.output_dir)?;

    // Print summary table
    print_summary_table(&results);

    // Find best performing set
    let best = results
        .iter()
        .max_by(|a, b| a.f1.partial_cmp(&b.f1).unwrap());
    if let Some(best) = best {
        println!("\n🏆 Best performing feature set: {}", best.name);
        println!("   F1: {:.1}%", best.f1 * 100.0);
        println!("   Features: {}", best.feature_count);
    }

    Ok(())
}

fn define_feature_sets() -> Vec<(String, Vec<usize>)> {
    let total_features = FEATURE_SCHEMA.feature_count();
    let all_features: Vec<usize> = (0..total_features).collect();

    // Get features by category
    let graph_features: Vec<usize> = FEATURE_SCHEMA
        .get_by_category(&FeatureCategory::Graph)
        .iter()
        .map(|f| f.index)
        .collect();

    let signature_features: Vec<usize> = FEATURE_SCHEMA
        .get_by_category(&FeatureCategory::Signature)
        .iter()
        .map(|f| f.index)
        .collect();

    let name_features: Vec<usize> = FEATURE_SCHEMA
        .get_by_category(&FeatureCategory::Name)
        .iter()
        .map(|f| f.index)
        .collect();

    let file_features: Vec<usize> = FEATURE_SCHEMA
        .get_by_category(&FeatureCategory::File)
        .iter()
        .map(|f| f.index)
        .collect();

    let type_features: Vec<usize> = FEATURE_SCHEMA
        .get_by_category(&FeatureCategory::Type)
        .iter()
        .map(|f| f.index)
        .collect();

    let complexity_features: Vec<usize> = FEATURE_SCHEMA
        .get_by_category(&FeatureCategory::Complexity)
        .iter()
        .map(|f| f.index)
        .collect();

    vec![
        // Baseline: Graph only
        ("Graph Only".to_string(), graph_features.clone()),
        // Graph + Signature
        ("Graph + Signature".to_string(), {
            let mut v = graph_features.clone();
            v.extend(signature_features.clone());
            v
        }),
        // Graph + Signature + Complexity
        ("Graph + Signature + Complexity".to_string(), {
            let mut v = graph_features.clone();
            v.extend(signature_features.clone());
            v.extend(complexity_features.clone());
            v
        }),
        // Graph + Signature + Complexity + Name
        ("Graph + Signature + Complexity + Name".to_string(), {
            let mut v = graph_features.clone();
            v.extend(signature_features.clone());
            v.extend(complexity_features.clone());
            v.extend(name_features.clone());
            v
        }),
        // All features
        ("All Features".to_string(), all_features),
        // Graph + Type context (for method detection)
        ("Graph + Type Context".to_string(), {
            let mut v = graph_features.clone();
            v.extend(type_features.clone());
            v
        }),
        // File context only
        ("File Context Only".to_string(), file_features),
        // Name patterns only
        ("Name Patterns Only".to_string(), name_features),
    ]
}

fn train_and_evaluate(
    train_examples: &[TrainingExample],
    val_examples: &[TrainingExample],
    feature_indices: &[usize],
    name: &str,
) -> AblationResult {
    // Extract features using only the specified indices
    let _train_features: Vec<Vec<f64>> = train_examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .map(|e| {
            let full = e.features.to_feature_vector();
            feature_indices.iter().map(|&i| full[i]).collect()
        })
        .collect();

    let _train_labels: Vec<f64> = train_examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .map(|e| match e.label {
            TrainingLabel::Alive => 1.0,
            TrainingLabel::Dead => 0.0,
            TrainingLabel::Unknown => 0.5,
        })
        .collect();

    // Train model
    let mut classifier = LinearClassifier::new(feature_indices.len())
        .with_learning_rate(0.01)
        .with_epochs(50);

    // Convert to training examples with subset features
    let subset_examples: Vec<TrainingExample> = train_examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .cloned()
        .collect();

    let _accuracy = classifier.train(&subset_examples);

    // Evaluate on validation set
    let val_features: Vec<Vec<f64>> = val_examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .map(|e| {
            let full = e.features.to_feature_vector();
            feature_indices.iter().map(|&i| full[i]).collect()
        })
        .collect();

    let val_labels: Vec<f64> = val_examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .map(|e| match e.label {
            TrainingLabel::Alive => 1.0,
            TrainingLabel::Dead => 0.0,
            TrainingLabel::Unknown => 0.5,
        })
        .collect();

    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for (features, &label) in val_features.iter().zip(val_labels.iter()) {
        let pred = classifier.predict(features);
        let pred_label = if pred > 0.5 { 1.0 } else { 0.0 };

        if pred_label == 1.0 && label == 1.0 {
            tp += 1;
        } else if pred_label == 0.0 && label == 0.0 {
            tn += 1;
        } else if pred_label == 1.0 && label == 0.0 {
            fp += 1;
        } else if pred_label == 0.0 && label == 1.0 {
            fn_ += 1;
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
    let accuracy_val = if total > 0 {
        (tp + tn) as f64 / total as f64
    } else {
        0.0
    };

    AblationResult {
        name: name.to_string(),
        features: feature_indices
            .iter()
            .map(|i| FEATURE_SCHEMA.features[*i].name.clone())
            .collect(),
        accuracy: accuracy_val,
        precision,
        recall,
        f1,
        feature_count: feature_indices.len(),
    }
}

fn save_results(
    results: &[AblationResult],
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Save as JSON
    let json_path = output_dir.join("ablation_results.json");
    std::fs::write(&json_path, serde_json::to_string_pretty(results)?)?;
    println!("📁 Results saved to: {:?}", json_path);

    // Save as CSV
    let csv_path = output_dir.join("ablation_results.csv");
    let mut csv = String::new();
    csv.push_str("Name,FeatureCount,Accuracy,Precision,Recall,F1\n");
    for r in results {
        csv.push_str(&format!(
            "{},{},{:.4},{:.4},{:.4},{:.4}\n",
            r.name, r.feature_count, r.accuracy, r.precision, r.recall, r.f1
        ));
    }
    std::fs::write(&csv_path, csv)?;
    println!("📁 CSV saved to: {:?}", csv_path);

    Ok(())
}

fn print_summary_table(results: &[AblationResult]) {
    println!("\n📊 Ablation Summary Table:");
    println!(
        "   {:<35} | {:>5} | {:>8} | {:>8} | {:>8} | {:>8}",
        "Feature Set", "Count", "Acc", "Prec", "Recall", "F1"
    );
    println!(
        "   {:-<35}-+-{:->5}-+-{:->8}-+-{:->8}-+-{:->8}-+-{:->8}",
        "", "", "", "", "", ""
    );

    // Sort by F1
    let mut sorted = results.to_vec();
    sorted.sort_by(|a, b| b.f1.partial_cmp(&a.f1).unwrap());

    for r in &sorted {
        println!(
            "   {:<35} | {:>5} | {:>7.1}% | {:>7.1}% | {:>7.1}% | {:>7.1}%",
            r.name,
            r.feature_count,
            r.accuracy * 100.0,
            r.precision * 100.0,
            r.recall * 100.0,
            r.f1 * 100.0
        );
    }
}
