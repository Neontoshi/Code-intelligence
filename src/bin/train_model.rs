// src/bin/train_model.rs

use code_intelligence::analysis::training_data::TrainingLabel;
use code_intelligence::ml::classifier::DeadCodeClassifier;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let data_file = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("combined_training.json")
    };

    let model_file = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("model.bin")
    };

    println!("🔬 Loading training data from: {:?}", data_file);

    let data = std::fs::read_to_string(&data_file)?;
    let examples: Vec<code_intelligence::analysis::training_data::TrainingExample> =
        serde_json::from_str(&data)?;

    println!("📊 Loaded {} examples", examples.len());

    let alive_count = examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let dead_count = examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();
    let unknown_count = examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Unknown)
        .count();

    println!("   Alive: {}", alive_count);
    println!("   Dead: {}", dead_count);
    println!("   Unknown: {}", unknown_count);

    // Train model
    let mut classifier = DeadCodeClassifier::new();
    classifier.train(&examples)?;

    // Print feature importance
    classifier.print_feature_importance();

    // Create versioned model
    use code_intelligence::ml::{ModelPerformance, TrainingMetadata, VersionedModel};

    let metadata = TrainingMetadata {
        training_repositories: vec!["combined".to_string()],
        examples_count: examples.len(),
        alive_count,
        dead_count,
        languages: vec!["rust".to_string(), "python".to_string(), "js".to_string()],
        training_date: chrono::Utc::now().to_rfc3339(),
        training_duration_secs: 0.0,
    };

    let performance = ModelPerformance {
        accuracy: classifier.accuracy,
        precision: 0.0,
        recall: 0.0,
        f1: 0.0,
        fpr: 0.0,
        fnr: 0.0,
        threshold: 0.92,
    };

    // Clone the model for versioned format (so we keep the original)
    if let Some(inner_model) = classifier.model.clone() {
        let versioned = VersionedModel::new(inner_model, metadata, Some(performance));
        let versioned_path = model_file.with_extension("v2.json");
        versioned.save(&versioned_path.to_string_lossy())?;
        println!("✅ Versioned model saved to: {:?}", versioned_path);
    }

    // Also save legacy format for backward compatibility
    classifier.save(&model_file.to_string_lossy())?;
    println!("✅ Legacy model saved to: {:?}", model_file);

    // Show predictions
    println!("\n🔮 Sample Predictions:");
    for example in examples.iter().take(10) {
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
        "\n📊 Model Accuracy: {:.1}%",
        classifier.get_accuracy() * 100.0
    );

    Ok(())
}
